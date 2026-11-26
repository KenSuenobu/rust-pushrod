// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the alert: modality, Return and Escape, the completion index, and its parts.

use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::event::EngineEvent;
use pushrod::geometry::{Point, point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetId};
use pushrod_widgets::alert::{
    Alert, AlertExt, AlertHandle, AlertKind, BUTTON_HEIGHT, BUTTON_WIDTH, HEIGHT, MIN_WIDTH,
};
use pushrod_widgets::button::Button;
use pushrod_widgets::text::Text;
use std::cell::RefCell;
use std::rc::Rc;

const WINDOW_W: u32 = 640;
const WINDOW_H: u32 = 400;

#[derive(Default)]
struct State {
    choices: Vec<usize>,
    main_clicks: u32,
    engine: Vec<EngineEvent>,
}

/// A window with one main button, and a slot that receives the handle of the next alert.
struct Harness {
    engine: Engine<State>,
    state: State,
    main: WidgetId,
    handle: Rc<RefCell<Option<AlertHandle>>>,
}

impl Harness {
    fn new() -> Self {
        let mut engine: Engine<State> = Engine::new(size(WINDOW_W, WINDOW_H), 30);
        let main = engine.add_widget(Button::new("Main").at(0, 0).size(100, 24).default_action());

        engine.on_click(main, |state, _ui| state.main_clicks += 1);
        engine.on_engine_event(|state, _ui, event| state.engine.push(event.clone()));

        Self {
            engine,
            state: State::default(),
            main,
            handle: Rc::new(RefCell::new(None)),
        }
    }

    /// Opens a Delete / Cancel warning from a callback; `customize` runs on the new handle.
    fn open_with<F>(&mut self, buttons: &'static [&'static str], default: usize, customize: F)
    where
        F: FnOnce(&mut AlertHandle, &mut pushrod::engine::Ui<State>) + 'static,
    {
        let slot = self.handle.clone();
        let mut customize = Some(customize);

        self.engine.on_idle(move |_, ui| {
            if let Some(customize) = customize.take() {
                let mut handle = ui.alert(
                    AlertKind::Warning,
                    "Delete \"report.txt\"?",
                    "This cannot be undone.",
                    buttons,
                    default,
                    |state: &mut State, _ui, choice| state.choices.push(choice),
                );

                customize(&mut handle, ui);
                *slot.borrow_mut() = Some(handle);
            }
        });
        self.engine.run_idle(&mut self.state);
    }

    fn open(&mut self) {
        self.open_with(&["Delete", "Cancel"], 0, |_, _| {});
    }

    fn handle(&self) -> AlertHandle {
        self.handle.borrow().clone().unwrap()
    }

    fn center_of(&mut self, id: WidgetId) -> Point {
        let origin = self.engine.widgets().absolute_origin(id);
        let size = self.engine.widgets().get(id).unwrap().get_size();

        point(origin.x + size.w as i32 / 2, origin.y + size.h as i32 / 2)
    }

    fn click_at(&mut self, position: Point) {
        for down in [true, false] {
            let event = if down {
                InputEvent::MouseDown {
                    position,
                    button: MouseButton::Left,
                    clicks: 1,
                }
            } else {
                InputEvent::MouseUp {
                    position,
                    button: MouseButton::Left,
                    clicks: 1,
                }
            };

            self.engine.dispatch(event, &mut self.state);
        }
    }

    fn click(&mut self, id: WidgetId) {
        let position = self.center_of(id);

        self.click_at(position);
    }

    fn key(&mut self, key: Key) {
        self.engine.dispatch(
            InputEvent::KeyDown {
                key,
                modifiers: Modifiers::empty(),
                repeat: false,
            },
            &mut self.state,
        );
    }
}

#[test]
fn the_alert_opens_centered_and_modal() {
    let mut h = Harness::new();

    h.open();

    let alert = h.handle().id();
    let widgets = h.engine.widgets();
    let bounds = widgets.get(alert).unwrap();

    assert_eq!(widgets.top_modal(), Some(alert));
    assert_eq!(bounds.get_size(), size(MIN_WIDTH, HEIGHT));
    assert_eq!(
        bounds.get_origin(),
        point(
            ((WINDOW_W - MIN_WIDTH) / 2) as i32,
            ((WINDOW_H - HEIGHT) / 2) as i32
        )
    );
    assert!(widgets.is_inert(h.main));
    assert!(
        h.state
            .engine
            .contains(&EngineEvent::ModalOpened { root: alert })
    );
}

#[test]
fn nothing_else_receives_input_while_open() {
    let mut h = Harness::new();

    h.open();
    h.click_at(point(10, 10));

    assert_eq!(h.state.main_clicks, 0);
    assert!(h.state.choices.is_empty());
    assert_eq!(h.engine.widgets().top_modal(), Some(h.handle().id()));
}

#[test]
fn return_chooses_the_default_button() {
    let mut h = Harness::new();

    h.open();
    h.key(Key::Return);

    // The main window's default button is never pressed.
    assert_eq!(h.state.choices, vec![0]);
    assert_eq!(h.state.main_clicks, 0);
}

#[test]
fn escape_chooses_the_last_button() {
    let mut h = Harness::new();

    h.open_with(&["Save", "Don't Save", "Cancel"], 0, |_, _| {});
    h.key(Key::Escape);

    assert_eq!(h.state.choices, vec![2]);
}

#[test]
fn clicking_a_button_passes_its_index_and_removes_the_alert() {
    let mut h = Harness::new();

    h.open();

    let handle = h.handle();
    let cancel = handle.parts().buttons[1];

    h.click(cancel);

    assert_eq!(h.state.choices, vec![1]);
    assert_eq!(h.engine.widgets().top_modal(), None);
    assert!(h.engine.widgets().get(handle.id()).is_none());
    assert!(h.engine.widgets().get(handle.parts().message).is_none());
    assert!(h.state.engine.contains(&EngineEvent::ModalClosed {
        root: handle.id(),
        dismissed_by: cancel,
    }));

    // The window takes input again.
    h.click(h.main);
    assert_eq!(h.state.main_clicks, 1);
}

#[test]
fn the_completion_runs_once() {
    let mut h = Harness::new();

    h.open();
    h.key(Key::Return);
    h.key(Key::Return);

    // The second Return goes to the main window's default button.
    assert_eq!(h.state.choices, vec![0]);
    assert_eq!(h.state.main_clicks, 1);
}

#[test]
fn parts_can_be_retrieved_and_restyled() {
    let mut h = Harness::new();

    h.open_with(&["Delete", "Cancel"], 0, |handle, ui| {
        let message = handle.parts().message;
        let text = ui.get_mut::<Text>(message).unwrap();

        text.set_font_color(Color::RED);
        text.set_text("Gone for good.".to_string());
    });

    let handle = h.handle();
    let widgets = h.engine.widgets();
    let message = widgets.get_as::<Text>(handle.parts().message).unwrap();

    assert_eq!(message.get_font_color(), Color::RED);
    assert_eq!(message.get_text(), "Gone for good.");
    assert_eq!(
        widgets
            .get_as::<Text>(handle.parts().title)
            .unwrap()
            .get_text(),
        "Delete \"report.txt\"?"
    );
    assert_eq!(widgets.part(handle.id(), "icon"), Some(handle.parts().icon));
    assert_eq!(
        widgets
            .get_as::<Button>(handle.parts().buttons[0])
            .unwrap()
            .get_text(),
        "Delete"
    );
}

#[test]
fn an_added_button_goes_left_and_reports_its_index() {
    let mut h = Harness::new();

    h.open_with(&["Delete", "Cancel"], 0, |handle, ui| {
        assert!(handle.add_button(ui, "Help").is_some());
    });

    let handle = h.handle();
    let buttons = handle.parts().buttons.clone();
    let alert = handle.id();

    assert_eq!(buttons.len(), 3);

    let widgets = h.engine.widgets();
    let x = |widgets: &mut WidgetCache, id| widgets.get(id).unwrap().get_origin().x;
    let (help, delete, cancel) = (
        x(widgets, buttons[2]),
        x(widgets, buttons[0]),
        x(widgets, buttons[1]),
    );

    assert!(help < delete && delete < cancel);
    assert!(widgets.get(alert).unwrap().get_size().w >= 3 * BUTTON_WIDTH);
    assert_eq!(
        widgets.get(buttons[2]).unwrap().get_size(),
        size(BUTTON_WIDTH, BUTTON_HEIGHT)
    );

    // Escape still chooses Cancel; clicking Help reports index 2.
    assert!(
        !widgets
            .get(buttons[2])
            .unwrap()
            .behavior()
            .contains(Behavior::CANCEL_ACTION)
    );

    h.click(buttons[2]);
    assert_eq!(h.state.choices, vec![2]);
}

#[test]
fn the_alert_widens_to_fit_many_buttons() {
    let mut h = Harness::new();

    h.open_with(&["One", "Two", "Three", "Four", "Five"], 0, |_, _| {});

    let alert = h.handle().id();
    let widgets = h.engine.widgets();
    let width = widgets.get(alert).unwrap().get_size().w;

    assert!(width > MIN_WIDTH);
    assert_eq!(
        widgets.get(alert).unwrap().get_origin().x,
        ((WINDOW_W - width) / 2) as i32
    );
}

#[test]
fn set_default_button_moves_return() {
    let mut h = Harness::new();

    h.open_with(&["Delete", "Cancel"], 0, |handle, ui| {
        assert!(handle.set_default_button(ui, 1));
        assert!(!handle.set_default_button(ui, 5));
    });

    let buttons = h.handle().parts().buttons.clone();
    let flags = |h: &mut Harness, i: usize| h.engine.widgets().get(buttons[i]).unwrap().behavior();

    assert!(!flags(&mut h, 0).contains(Behavior::DEFAULT_ACTION));
    assert!(flags(&mut h, 1).contains(Behavior::DEFAULT_ACTION));

    h.key(Key::Return);
    assert_eq!(h.state.choices, vec![1]);
}

#[test]
fn removing_the_escape_button_moves_escape_to_the_new_last_button() {
    let mut h = Harness::new();

    h.open_with(&["Save", "Don't Save", "Cancel"], 0, |handle, ui| {
        assert!(handle.add_button(ui, "Help").is_some());
        assert!(handle.remove_button(ui, 2));
        assert!(!handle.remove_button(ui, 9));
    });

    let handle = h.handle();

    // Save, Don't Save, Help: the buttons after the removed one moved down an index.
    assert_eq!(handle.parts().buttons.len(), 3);

    h.key(Key::Escape);
    assert_eq!(h.state.choices, vec![1]);
}

#[test]
fn the_only_button_cannot_be_removed() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(WINDOW_W, WINDOW_H))));

    let alert = cache.add_child(
        0,
        Box::new(Alert::new(AlertKind::Info, "Saved", "", &["OK"], 0)),
    );

    assert!(matches!(
        Alert::remove_button(&mut cache, alert, 0),
        Err(Error::Composite { .. })
    ));
    assert!(matches!(
        Alert::remove_button(&mut cache, alert, 3),
        Err(Error::Composite { .. })
    ));
    assert!(matches!(
        Alert::set_default_button(&mut cache, alert, 3),
        Err(Error::Composite { .. })
    ));
    assert!(matches!(
        Alert::add_button(&mut cache, 99, "Help"),
        Err(Error::UnknownWidget { id: 99 })
    ));
    assert!(matches!(
        Alert::add_button(&mut cache, 0, "Help"),
        Err(Error::Composite { .. })
    ));
}

#[test]
fn an_empty_button_list_gets_ok_and_a_wild_default_is_clamped() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(WINDOW_W, WINDOW_H))));

    let alert = cache.add_child(
        0,
        Box::new(Alert::new(AlertKind::Error, "Oops", "", &[], 0)),
    );
    let parts = cache.get_as::<Alert>(alert).unwrap().typed_parts();
    let ok = cache.get_as::<Button>(parts.buttons[0]).unwrap();
    let flags = ok.behavior();

    assert_eq!(parts.buttons.len(), 1);
    assert_eq!(ok.get_text(), "OK");
    assert!(flags.contains(Behavior::DEFAULT_ACTION | Behavior::CANCEL_ACTION));
    assert!(flags.contains(Behavior::DISMISSES_DIALOG));

    let alert = cache.add_child(
        0,
        Box::new(Alert::new(
            AlertKind::Question,
            "Go?",
            "",
            &["Yes", "No"],
            7,
        )),
    );
    let parts = cache.get_as::<Alert>(alert).unwrap().typed_parts();

    assert!(
        cache
            .get(parts.buttons[1])
            .unwrap()
            .behavior()
            .contains(Behavior::DEFAULT_ACTION)
    );
}

#[test]
fn each_kind_has_its_own_icon() {
    let kinds = [
        AlertKind::Info,
        AlertKind::Warning,
        AlertKind::Error,
        AlertKind::Question,
    ];
    let symbols: Vec<_> = kinds.iter().map(|k| k.symbol()).collect();

    assert_eq!(symbols, vec!["i", "!", "\u{00d7}", "?"]);
    assert_eq!(AlertKind::default(), AlertKind::Info);
    assert_ne!(AlertKind::Warning.colors(), AlertKind::Error.colors());
}

#[test]
fn a_closed_handle_does_nothing() {
    let mut h = Harness::new();
    let late = Rc::new(RefCell::new(Vec::new()));
    let seen = late.clone();

    h.open();
    h.key(Key::Escape);

    let mut handle = h.handle();

    h.engine.on_idle(move |_, ui| {
        let added = handle.add_button(ui, "Late");
        let removed = handle.remove_button(ui, 0);
        let default = handle.set_default_button(ui, 0);

        seen.borrow_mut().push((added, removed, default));
    });
    h.engine.run_idle(&mut h.state);

    assert_eq!(late.borrow()[0], (None, false, false));
    assert_eq!(h.state.choices, vec![1]);
    assert!(
        h.state
            .engine
            .iter()
            .any(|e| matches!(e, EngineEvent::WidgetRemoved { .. }))
    );
}
