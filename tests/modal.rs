// Modal dialog tests: input blocked outside the top dialog, focus, Return/Escape, dismissal,
// nesting, and the engine events.
//
// Headless: raw input events are built by hand and given to the engine.

use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::event::{EngineEvent, Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, ROOT_WIDGET_ID, VisualState, Widget, WidgetCore, WidgetId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// A widget that logs the input it receives and emits `Clicked` when released on itself.
struct Probe {
    core: WidgetCore,
    log: Rc<RefCell<Vec<String>>>,
}

impl Widget for Probe {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        let id = self.get_id();

        match event {
            Event::PointerPressed { id: target, .. } if *target == id => {
                self.log.borrow_mut().push(format!("press {id}"));
            }
            Event::PointerReleased {
                id: target,
                button,
                count,
                ..
            } if *target == id => {
                self.log.borrow_mut().push(format!("release {id}"));
                ctx.emit(Event::Clicked {
                    id,
                    button: *button,
                    count: *count,
                });
            }
            Event::KeyPressed { id: target, .. } if *target == id => {
                self.log.borrow_mut().push(format!("key {id}"));
            }
            _ => {}
        }

        Propagation::Continue
    }
}

/// The application state: what each dismissal callback saw, and every engine event.
#[derive(Default)]
struct State {
    dismissed: Vec<(WidgetId, WidgetId)>,
    engine: Vec<EngineEvent>,
}

/// A window with a main-screen button and a hidden dialog of two buttons:
///
/// ```text
///   0 root
///   ├── 1 main button     (0, 0)   focusable, default action
///   └── 2 dialog panel    (100, 0) hidden, 100x100
///        ├── 3 OK         (100, 0)   focusable, default action, dismisses
///        └── 4 Cancel     (130, 0)   focusable, cancel action, dismisses
/// ```
struct App {
    engine: Engine<State>,
    state: State,
    log: Rc<RefCell<Vec<String>>>,
    main: WidgetId,
    dialog: WidgetId,
    ok: WidgetId,
    cancel: WidgetId,
}

impl App {
    fn new() -> Self {
        let mut engine: Engine<State> = Engine::new(size(300, 300), 30);
        let log = Rc::new(RefCell::new(Vec::new()));
        let main = add_probe(
            &mut engine,
            &log,
            ROOT_WIDGET_ID,
            0,
            Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION,
        );
        let dialog = add_panel(&mut engine, 100);
        let ok = add_probe(
            &mut engine,
            &log,
            dialog,
            0,
            Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION | Behavior::DISMISSES_DIALOG,
        );
        let cancel = add_probe(
            &mut engine,
            &log,
            dialog,
            30,
            Behavior::FOCUSABLE | Behavior::CANCEL_ACTION | Behavior::DISMISSES_DIALOG,
        );

        engine.on_engine_event(|state: &mut State, _ui, event| state.engine.push(event.clone()));

        Self {
            engine,
            state: State::default(),
            log,
            main,
            dialog,
            ok,
            cancel,
        }
    }

    /// Opens `dialog` from a callback, recording the dismissal as `(root, dismissed_by)`.
    fn open(&mut self, dialog: WidgetId) {
        let pending = Rc::new(Cell::new(true));

        // Idle callbacks stay registered, so each one opens its dialog only once.
        self.engine.on_idle(move |_, ui| {
            if pending.replace(false) {
                ui.open_modal(dialog, move |state: &mut State, _ui, by| {
                    state.dismissed.push((dialog, by));
                })
                .unwrap();
            }
        });
        self.engine.run_idle(&mut self.state);
    }

    fn click(&mut self, x: i32) {
        let position = point(x, 5);

        self.engine.dispatch(
            InputEvent::MouseDown {
                position,
                button: MouseButton::Left,
                clicks: 1,
            },
            &mut self.state,
        );
        self.engine.dispatch(
            InputEvent::MouseUp {
                position,
                button: MouseButton::Left,
                clicks: 1,
            },
            &mut self.state,
        );
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

    fn take_log(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }
}

/// Adds a 20x20 probe at `(x, 0)` relative to `parent`.
fn add_probe(
    engine: &mut Engine<State>,
    log: &Rc<RefCell<Vec<String>>>,
    parent: WidgetId,
    x: i32,
    behavior: Behavior,
) -> WidgetId {
    let mut core = WidgetCore::new(point(x, 0), size(20, 20));

    core.set_event_mask(EventMask::ALL);

    let id = engine.add_child_widget(
        parent,
        Probe {
            core,
            log: log.clone(),
        },
    );

    engine.widgets().get_mut(id).unwrap().set_behavior(behavior);

    id
}

/// Adds a hidden 100x100 panel at `(x, 0)`, to hold a dialog.
fn add_panel(engine: &mut Engine<State>, x: i32) -> WidgetId {
    let id = engine.add_widget(RootWidget::new(size(100, 100)));
    let widget = engine.widgets().get_mut(id).unwrap();

    widget.set_origin(point(x, 0));
    widget.set_hidden(true);

    id
}

#[test]
fn opening_shows_the_dialog_and_focuses_its_first_control() {
    let mut app = App::new();

    app.open(app.dialog);

    let widgets = app.engine.widgets();

    assert!(!widgets.get(app.dialog).unwrap().is_hidden());
    assert_eq!(widgets.top_modal(), Some(app.dialog));
    assert_eq!(widgets.focused(), Some(app.ok));
    assert!(
        app.state
            .engine
            .contains(&EngineEvent::ModalOpened { root: app.dialog })
    );
}

#[test]
fn input_is_blocked_outside_the_top_modal() {
    let mut app = App::new();

    app.open(app.dialog);
    app.take_log();

    // A click on the main screen reaches nothing and takes no focus or pressed look.
    app.click(5);
    assert!(app.take_log().is_empty());
    assert_eq!(app.engine.widgets().focused(), Some(app.ok));
    assert!(app.engine.widgets().is_inert(app.main));
    assert!(!app.engine.widgets().set_focus(app.main));

    let state = app.engine.widgets().get(app.main).unwrap().state();
    assert!(!state.contains(VisualState::PRESSED));

    // Hovering the main screen does not mark it hovered.
    app.engine.dispatch(
        InputEvent::MouseMove {
            position: point(5, 5),
        },
        &mut app.state,
    );
    let state = app.engine.widgets().get(app.main).unwrap().state();
    assert!(!state.contains(VisualState::HOVERED));

    // Tab stays inside the dialog.
    app.key(Key::Tab);
    assert_eq!(app.engine.widgets().focused(), Some(app.cancel));
    app.key(Key::Tab);
    assert_eq!(app.engine.widgets().focused(), Some(app.ok));

    // Keys reach the focused control inside the dialog.
    app.take_log();
    app.key(Key::Char('a'));
    assert_eq!(app.take_log(), vec![format!("key {}", app.ok)]);
}

#[test]
fn clicking_a_dismissing_control_closes_the_dialog_and_reports_it() {
    let mut app = App::new();
    let clicked = Rc::new(RefCell::new(Vec::new()));
    let seen = clicked.clone();

    app.engine.on_click(app.cancel, move |state, _ui| {
        // The control's own callback runs before the dialog's dismissal callback.
        seen.borrow_mut().push(state.dismissed.len());
    });
    app.engine.widgets().set_focus(app.main);
    app.open(app.dialog);
    app.click(135);

    assert_eq!(app.state.dismissed, vec![(app.dialog, app.cancel)]);
    assert_eq!(*clicked.borrow(), vec![0]);

    let widgets = app.engine.widgets();

    assert_eq!(widgets.top_modal(), None);
    assert!(widgets.get(app.dialog).unwrap().is_hidden());
    assert!(!widgets.is_inert(app.main));
    // Focus goes back to where it was before the dialog opened.
    assert_eq!(widgets.focused(), Some(app.main));

    let dismiss = app
        .state
        .engine
        .iter()
        .position(|e| *e == EngineEvent::DialogDismissRequested { id: app.cancel });
    let closed = app.state.engine.iter().position(|e| {
        *e == EngineEvent::ModalClosed {
            root: app.dialog,
            dismissed_by: app.cancel,
        }
    });

    assert!(dismiss.is_some() && closed.is_some() && dismiss < closed);
}

#[test]
fn return_and_escape_use_the_dialogs_default_and_cancel_controls() {
    let mut app = App::new();

    app.open(app.dialog);
    app.key(Key::Return);
    assert_eq!(app.state.dismissed, vec![(app.dialog, app.ok)]);

    // Once closed, Return is the main screen's default again, and dismisses nothing.
    app.state.engine.clear();
    app.key(Key::Return);
    assert!(app.state.engine.contains(&EngineEvent::ActionTriggered {
        id: app.main,
        cancel: false
    }));
    assert_eq!(app.state.dismissed.len(), 1);

    app.open(app.dialog);
    app.key(Key::Escape);
    assert_eq!(
        app.state.dismissed,
        vec![(app.dialog, app.ok), (app.dialog, app.cancel)]
    );
}

#[test]
fn return_never_reaches_the_main_screen_while_a_dialog_is_open() {
    let mut app = App::new();

    // A dialog with no default control: Return does nothing at all.
    app.engine
        .widgets()
        .get_mut(app.ok)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE | Behavior::DISMISSES_DIALOG);
    app.open(app.dialog);
    app.state.engine.clear();
    app.key(Key::Return);

    assert!(
        !app.state
            .engine
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionTriggered { .. }))
    );
    assert_eq!(app.engine.widgets().top_modal(), Some(app.dialog));
}

#[test]
fn modals_nest_and_only_the_top_one_receives_input() {
    let mut app = App::new();
    let alert = add_panel(&mut app.engine, 200);
    let alert_ok = add_probe(
        &mut app.engine,
        &app.log,
        alert,
        0,
        Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION | Behavior::DISMISSES_DIALOG,
    );

    app.open(app.dialog);
    app.engine.widgets().set_focus(app.cancel);
    app.open(alert);

    assert_eq!(app.engine.widgets().open_modals(), vec![app.dialog, alert]);
    assert_eq!(app.engine.widgets().focused(), Some(alert_ok));
    assert!(app.engine.widgets().is_inert(app.ok));

    // The settings dialog under the alert is inert.
    app.click(105);
    assert!(app.state.dismissed.is_empty());

    // Return closes only the alert; focus returns to the dialog's Cancel.
    app.key(Key::Return);
    assert_eq!(app.state.dismissed, vec![(alert, alert_ok)]);
    assert_eq!(app.engine.widgets().top_modal(), Some(app.dialog));
    assert_eq!(app.engine.widgets().focused(), Some(app.cancel));

    app.key(Key::Return);
    assert_eq!(
        app.state.dismissed,
        vec![(alert, alert_ok), (app.dialog, app.ok)]
    );
    assert_eq!(app.engine.widgets().top_modal(), None);
}

#[test]
fn a_dismissal_callback_can_open_another_dialog() {
    let mut app = App::new();
    let alert = add_panel(&mut app.engine, 200);
    let alert_ok = add_probe(
        &mut app.engine,
        &app.log,
        alert,
        0,
        Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION | Behavior::DISMISSES_DIALOG,
    );
    let dialog = app.dialog;

    app.engine.on_idle(move |_, ui| {
        if ui.widgets().top_modal().is_none() {
            ui.open_modal(dialog, move |state: &mut State, ui, by| {
                state.dismissed.push((dialog, by));
                ui.open_modal(alert, move |state: &mut State, _ui, by| {
                    state.dismissed.push((alert, by));
                })
                .unwrap();
            })
            .unwrap();
        }
    });
    app.engine.run_idle(&mut app.state);
    app.key(Key::Return);

    assert_eq!(app.engine.widgets().top_modal(), Some(alert));
    assert_eq!(app.engine.widgets().focused(), Some(alert_ok));

    app.key(Key::Return);
    assert_eq!(
        app.state.dismissed,
        vec![(dialog, app.ok), (alert, alert_ok)]
    );
}

#[test]
fn a_dismissing_control_outside_any_dialog_does_nothing() {
    let mut app = App::new();

    app.engine
        .widgets()
        .get_mut(app.main)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE | Behavior::DISMISSES_DIALOG);
    app.click(5);

    assert!(app.state.dismissed.is_empty());
    assert!(
        !app.state
            .engine
            .iter()
            .any(|e| matches!(e, EngineEvent::DialogDismissRequested { .. }))
    );
}

#[test]
fn opening_an_invalid_root_is_an_error() {
    let mut app = App::new();
    let dialog = app.dialog;
    let results = Rc::new(RefCell::new(Vec::new()));
    let seen = results.clone();

    app.engine.on_idle(move |_, ui| {
        let mut seen = seen.borrow_mut();

        seen.push(ui.open_modal(ROOT_WIDGET_ID, |_, _, _| {}));
        seen.push(ui.open_modal(99, |_, _, _| {}));
        seen.push(ui.open_modal(dialog, |_, _, _| {}));
        seen.push(ui.open_modal(dialog, |_, _, _| {}));
    });
    app.engine.run_idle(&mut app.state);

    let results = results.borrow();

    assert!(matches!(results[0], Err(Error::Tree { .. })));
    assert!(matches!(results[1], Err(Error::UnknownWidget { id: 99 })));
    assert!(results[2].is_ok());
    assert!(matches!(results[3], Err(Error::Tree { .. })));
    assert_eq!(app.engine.widgets().open_modals(), vec![dialog]);
}

#[test]
fn removing_an_open_dialog_releases_the_window() {
    let mut app = App::new();

    app.open(app.dialog);
    app.engine.widgets().remove(app.dialog).unwrap();
    app.engine.run_idle(&mut app.state);

    assert_eq!(app.engine.widgets().top_modal(), None);
    assert!(!app.engine.widgets().is_inert(app.main));
    assert!(app.engine.widgets().set_focus(app.main));
    assert!(app.state.dismissed.is_empty());
}

#[test]
fn opening_repaints_a_dialog_that_was_hidden_since_it_was_built() {
    let mut app = App::new();
    let mut backend = pushrod::backend::headless::HeadlessBackend::new(size(300, 300));

    // Hidden `Widget`s are skipped by the first frame, so the dialog has never been painted.
    app.engine.widgets().draw_loop(&mut backend);
    assert!(
        !app.engine
            .widgets()
            .get(app.cancel)
            .unwrap()
            .is_invalidated()
    );

    app.open(app.dialog);

    for id in [app.dialog, app.ok, app.cancel] {
        assert!(app.engine.widgets().get(id).unwrap().is_invalidated());
    }
}
