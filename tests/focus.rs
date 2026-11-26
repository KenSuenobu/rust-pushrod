// Keyboard focus tests: click, Tab, arrows, Return/Escape, events, and Ui access.
//
// Headless: raw SDL events are built by hand and given to the cache.

use pushrod::cache::WidgetCache;
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{point, size};
use pushrod::input::InputEvent;
use pushrod::input::MouseButton;
use pushrod::input::{Key, Modifiers};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore, WidgetId};
use std::cell::RefCell;
use std::rc::Rc;

/// A widget that records the events it receives, and can stop key presses.
struct Probe {
    core: WidgetCore,
    log: Rc<RefCell<Vec<String>>>,
    stop_keys: bool,
    text: bool,
}

impl Widget for Probe {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn accepts_text_input(&self) -> bool {
        self.text
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventCtx) -> Propagation {
        let id = self.get_id();

        match event {
            Event::FocusGained { .. } => self.log.borrow_mut().push(format!("gained {id}")),
            Event::FocusLost { .. } => self.log.borrow_mut().push(format!("lost {id}")),
            Event::TextEntered { text, .. } => {
                self.log.borrow_mut().push(format!("text {id} {text}"))
            }
            Event::KeyPressed { .. } if self.stop_keys => return Propagation::Stop,
            _ => {}
        }

        Propagation::Continue
    }
}

struct Form {
    cache: WidgetCache,
    log: Rc<RefCell<Vec<String>>>,
}

impl Form {
    fn new() -> Self {
        let mut cache = WidgetCache::new();

        cache.add(Box::new(RootWidget::new(size(300, 300))));

        Self {
            cache,
            log: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Adds a 20x20 probe at `x`, in the row at y = 0, with a full event mask.
    fn add(&mut self, parent: WidgetId, x: i32, behavior: Behavior) -> WidgetId {
        let mut core = WidgetCore::new(point(x, 0), size(20, 20));

        core.set_event_mask(EventMask::ALL);

        let probe = Probe {
            core,
            log: self.log.clone(),
            stop_keys: false,
            text: false,
        };
        let id = self.cache.add_child(parent, Box::new(probe));

        self.cache.get_mut(id).unwrap().set_behavior(behavior);

        id
    }

    fn key(&mut self, key: Key, modifiers: Modifiers) -> Vec<Event> {
        self.cache.handle_event(InputEvent::KeyDown {
            key,
            modifiers,
            repeat: false,
        })
    }

    fn click(&mut self, x: i32) {
        for down in [true, false] {
            let event = if down {
                InputEvent::MouseDown {
                    position: point(x, 5),
                    button: MouseButton::Left,
                    clicks: 1,
                }
            } else {
                InputEvent::MouseUp {
                    position: point(x, 5),
                    button: MouseButton::Left,
                    clicks: 1,
                }
            };

            self.cache.handle_event(event);
        }
    }
}

/// Events do not implement `PartialEq`, so tests compare their `Debug` text.
fn text(events: impl IntoIterator<Item = Event>) -> Vec<String> {
    events.into_iter().map(|e| format!("{e:?}")).collect()
}

const FOCUSABLE: Behavior = Behavior::FOCUSABLE;

#[test]
fn clicking_a_focusable_widget_focuses_it_and_clicking_another_moves_focus() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);
    let b = form.add(0, 30, FOCUSABLE);

    form.click(5);
    assert_eq!(form.cache.focused(), Some(a));

    form.click(35);
    assert_eq!(form.cache.focused(), Some(b));
    assert_eq!(*form.log.borrow(), ["gained 1", "lost 1", "gained 2"]);
}

#[test]
fn clicking_something_unfocusable_keeps_focus() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);

    form.add(0, 30, Behavior::empty());
    form.click(5);
    form.click(35);

    assert_eq!(form.cache.focused(), Some(a));
}

#[test]
fn clicking_a_child_focuses_the_focusable_parent() {
    let mut form = Form::new();
    let parent = form.add(0, 0, FOCUSABLE);

    form.cache.get_mut(parent).unwrap().set_size(size(100, 100));
    form.add(parent, 0, Behavior::empty());
    form.click(5);

    assert_eq!(form.cache.focused(), Some(parent));
}

#[test]
fn focus_events_are_queued_for_the_application_lost_before_gained() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);
    let b = form.add(0, 30, FOCUSABLE);

    form.cache.take_pending_events();
    form.cache.set_focus(a);
    form.cache.take_pending_events();
    form.cache.set_focus(b);

    let events = form.cache.take_pending_events();

    assert_eq!(
        text(events),
        text([
            Event::FocusLost { id: a },
            Event::FocusGained { id: b },
            Event::Engine(EngineEvent::FocusChanged {
                old: Some(a),
                new: Some(b)
            })
        ])
    );
}

#[test]
fn focus_sets_and_clears_the_focused_visual_state() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);
    let b = form.add(0, 30, FOCUSABLE);

    form.cache.set_focus(a);
    assert!(
        form.cache
            .get(a)
            .unwrap()
            .state()
            .contains(VisualState::FOCUSED)
    );

    form.cache.set_focus(b);
    assert!(
        !form
            .cache
            .get(a)
            .unwrap()
            .state()
            .contains(VisualState::FOCUSED)
    );
    assert!(
        form.cache
            .get(b)
            .unwrap()
            .state()
            .contains(VisualState::FOCUSED)
    );
}

#[test]
fn set_focus_refuses_widgets_that_cannot_take_it() {
    let mut form = Form::new();
    let plain = form.add(0, 0, Behavior::empty());
    let disabled = form.add(0, 30, FOCUSABLE);
    let hidden = form.add(0, 60, FOCUSABLE);
    let child = form.add(hidden, 0, FOCUSABLE);

    form.cache.get_mut(disabled).unwrap().set_enabled(false);
    form.cache.get_mut(hidden).unwrap().set_hidden(true);

    for id in [plain, disabled, hidden, child, 99] {
        assert!(!form.cache.set_focus(id), "{id}");
    }

    assert_eq!(form.cache.focused(), None);
}

#[test]
fn focusing_the_focused_widget_does_nothing() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);

    form.cache.set_focus(a);
    form.cache.take_pending_events();

    assert!(form.cache.set_focus(a));
    assert!(form.cache.take_pending_events().is_empty());
}

#[test]
fn tab_and_shift_tab_walk_in_id_order_and_wrap() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);

    form.add(0, 30, Behavior::empty());

    let c = form.add(0, 60, FOCUSABLE);

    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(a));
    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(c));
    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(a));
    form.key(Key::Tab, Modifiers::SHIFT);
    assert_eq!(form.cache.focused(), Some(c));
}

#[test]
fn shift_tab_with_no_focus_goes_to_the_last_widget() {
    let mut form = Form::new();

    form.add(0, 0, FOCUSABLE);

    let last = form.add(0, 30, FOCUSABLE);

    form.key(Key::Tab, Modifiers::SHIFT);

    assert_eq!(form.cache.focused(), Some(last));
}

#[test]
fn tab_skips_disabled_and_hidden_widgets() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);
    let b = form.add(0, 30, FOCUSABLE);
    let c = form.add(0, 60, FOCUSABLE);

    form.cache.get_mut(b).unwrap().set_enabled(false);
    form.cache.set_focus(a);
    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(c));

    form.cache.get_mut(a).unwrap().set_hidden(true);
    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(c));
}

#[test]
fn a_widget_that_stops_the_key_keeps_tab_from_moving_focus() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);

    form.add(0, 30, FOCUSABLE);
    form.cache.get_as_mut::<Probe>(a).unwrap().stop_keys = true;
    form.cache.set_focus(a);
    form.key(Key::Tab, Modifiers::empty());

    assert_eq!(form.cache.focused(), Some(a));
}

#[test]
fn control_tab_does_not_move_focus() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);

    form.add(0, 30, FOCUSABLE);
    form.cache.set_focus(a);
    form.key(Key::Tab, Modifiers::CTRL);

    assert_eq!(form.cache.focused(), Some(a));
}

/// Builds: a field, a radio group of three (the middle one checked), and a final field.
fn radio_form() -> (Form, WidgetId, [WidgetId; 3], WidgetId) {
    let mut form = Form::new();
    let field = form.add(0, 0, FOCUSABLE);
    let group = form.add(0, 30, Behavior::empty());
    let radios = [
        form.add(group, 0, FOCUSABLE | Behavior::EXCLUSIVE),
        form.add(group, 30, FOCUSABLE | Behavior::EXCLUSIVE),
        form.add(group, 60, FOCUSABLE | Behavior::EXCLUSIVE),
    ];
    let last = form.add(0, 200, FOCUSABLE);

    form.cache
        .get_mut(radios[1])
        .unwrap()
        .set_state(VisualState::CHECKED);

    (form, field, radios, last)
}

#[test]
fn tab_visits_a_radio_group_once_at_its_checked_member() {
    let (mut form, field, radios, last) = radio_form();

    form.cache.set_focus(field);
    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(radios[1]));
    form.key(Key::Tab, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(last));
    form.key(Key::Tab, Modifiers::SHIFT);
    assert_eq!(form.cache.focused(), Some(radios[1]));
}

#[test]
fn tab_from_an_unchecked_radio_leaves_the_group() {
    let (mut form, _, radios, last) = radio_form();

    form.cache.set_focus(radios[2]);
    form.key(Key::Tab, Modifiers::empty());

    assert_eq!(form.cache.focused(), Some(last));
}

#[test]
fn arrows_move_within_a_group_and_wrap() {
    let (mut form, _, radios, _) = radio_form();

    form.cache.set_focus(radios[1]);
    form.key(Key::Down, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(radios[2]));
    form.key(Key::Right, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(radios[0]));
    form.key(Key::Up, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(radios[2]));
    form.key(Key::Left, Modifiers::empty());
    assert_eq!(form.cache.focused(), Some(radios[1]));
}

#[test]
fn arrows_do_nothing_outside_a_group() {
    let (mut form, field, _, _) = radio_form();

    form.cache.set_focus(field);
    form.key(Key::Down, Modifiers::empty());

    assert_eq!(form.cache.focused(), Some(field));
}

#[test]
fn return_activates_the_default_widget_and_escape_the_cancel_widget() {
    let mut form = Form::new();
    let field = form.add(0, 0, FOCUSABLE);
    let ok = form.add(0, 30, FOCUSABLE | Behavior::DEFAULT_ACTION);
    let cancel = form.add(0, 60, FOCUSABLE | Behavior::CANCEL_ACTION);

    form.cache.set_focus(field);
    form.cache.take_pending_events();

    assert_eq!(
        text(form.key(Key::Return, Modifiers::empty())),
        text([Event::Activated { id: ok }])
    );
    assert_eq!(
        text(form.cache.take_pending_events()),
        text([Event::Engine(EngineEvent::ActionTriggered {
            id: ok,
            cancel: false
        })])
    );
    assert_eq!(
        text(form.key(Key::KpEnter, Modifiers::empty())),
        text([Event::Activated { id: ok }])
    );
    assert_eq!(
        text(form.key(Key::Escape, Modifiers::empty())),
        text([Event::Activated { id: cancel }])
    );
    assert_eq!(
        text(form.cache.take_pending_events().into_iter().last()),
        text([Event::Engine(EngineEvent::ActionTriggered {
            id: cancel,
            cancel: true
        })])
    );
}

#[cfg(target_os = "macos")]
#[test]
fn command_period_cancels_on_macos() {
    let mut form = Form::new();
    let cancel = form.add(0, 0, FOCUSABLE | Behavior::CANCEL_ACTION);

    assert_eq!(
        text(form.key(Key::Char('.'), Modifiers::GUI)),
        text([Event::Activated { id: cancel }])
    );
}

#[test]
fn return_works_with_no_focus_and_skips_a_disabled_default() {
    let mut form = Form::new();
    let ok = form.add(0, 0, Behavior::DEFAULT_ACTION);

    assert_eq!(
        text(form.key(Key::Return, Modifiers::empty())),
        text([Event::Activated { id: ok }])
    );

    form.cache.get_mut(ok).unwrap().set_enabled(false);

    assert!(form.key(Key::Return, Modifiers::empty()).is_empty());
}

#[test]
fn the_nearest_form_wins_when_two_forms_have_default_buttons() {
    let mut form = Form::new();
    let outer_ok = form.add(0, 0, Behavior::DEFAULT_ACTION);
    let dialog = form.add(0, 30, Behavior::empty());
    let field = form.add(dialog, 0, FOCUSABLE);
    let inner_ok = form.add(dialog, 30, Behavior::DEFAULT_ACTION);

    form.cache.set_focus(field);

    assert_eq!(
        text(form.key(Key::Return, Modifiers::empty())),
        text([Event::Activated { id: inner_ok }])
    );

    form.cache.get_mut(inner_ok).unwrap().set_hidden(true);

    assert_eq!(
        text(form.key(Key::Return, Modifiers::empty())),
        text([Event::Activated { id: outer_ok }])
    );
}

#[test]
fn a_widget_that_stops_return_keeps_the_default_from_firing() {
    let mut form = Form::new();
    let field = form.add(0, 0, FOCUSABLE);

    form.add(0, 30, Behavior::DEFAULT_ACTION);
    form.cache.get_as_mut::<Probe>(field).unwrap().stop_keys = true;
    form.cache.set_focus(field);

    assert!(form.key(Key::Return, Modifiers::empty()).is_empty());
}

#[test]
fn text_input_is_wanted_only_while_a_text_widget_has_focus() {
    let mut form = Form::new();
    let plain = form.add(0, 0, FOCUSABLE);
    let text = form.add(0, 30, FOCUSABLE);

    form.cache.get_as_mut::<Probe>(text).unwrap().text = true;

    form.cache.set_focus(plain);
    assert!(!form.cache.text_input_wanted());
    form.cache.set_focus(text);
    assert!(form.cache.text_input_wanted());
    form.cache.clear_focus();
    assert!(!form.cache.text_input_wanted());
    assert_eq!(form.cache.focused(), None);
}

#[test]
fn typed_text_goes_to_the_focused_widget_only() {
    let mut form = Form::new();
    let a = form.add(0, 0, FOCUSABLE);

    form.add(0, 30, FOCUSABLE);

    let typed = |form: &mut Form| {
        form.cache.handle_event(InputEvent::Text("hi".into()));
    };

    typed(&mut form);
    assert!(
        form.log.borrow().is_empty(),
        "nothing focused, nothing typed"
    );

    form.cache.set_focus(a);
    form.log.borrow_mut().clear();
    typed(&mut form);

    assert_eq!(*form.log.borrow(), ["text 1 hi"]);
}

#[test]
fn ui_set_focus_and_focused_work_and_callbacks_fire() {
    let mut engine: Engine<Vec<String>> = Engine::new(size(300, 300), 60);
    let mut ids = Vec::new();

    for x in [0, 30] {
        let mut core = WidgetCore::new(point(x, 0), size(20, 20));

        core.set_event_mask(EventMask::ALL);

        let id = engine.add_widget(Probe {
            core,
            log: Rc::new(RefCell::new(Vec::new())),
            stop_keys: false,
            text: false,
        });

        engine
            .widgets()
            .get_mut(id)
            .unwrap()
            .set_behavior(Behavior::FOCUSABLE);
        ids.push(id);
    }

    let (first, second) = (ids[0], ids[1]);

    // Gaining focus in the first widget hands it to the second, through `Ui`.
    engine.on_focus_change(first, move |log, ui, gained| {
        log.push(format!("first {gained}"));

        if gained {
            log.push(format!("handed over: {}", ui.set_focus(second)));
        }
    });
    engine.on_engine_event(|log, ui, event| {
        if let EngineEvent::FocusChanged { old, new } = event {
            log.push(format!(
                "changed {old:?} -> {new:?}, now {:?}",
                ui.focused()
            ));
        }
    });

    assert!(engine.widgets().set_focus(first));

    let mut log = Vec::new();

    // Any event flushes the queued focus events.
    engine.dispatch(
        InputEvent::KeyUp {
            key: Key::Char('a'),
            modifiers: Modifiers::empty(),
        },
        &mut log,
    );

    assert_eq!(
        log,
        [
            "first true".to_string(),
            "handed over: true".to_string(),
            format!("changed None -> Some({first}), now Some({second})"),
            "first false".to_string(),
            format!("changed Some({first}) -> Some({second}), now Some({second})"),
        ]
    );
    assert_eq!(engine.widgets().focused(), Some(second));
}
