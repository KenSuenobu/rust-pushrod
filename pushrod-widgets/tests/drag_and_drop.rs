// Drag and drop tests: input goes through `Engine::dispatch`, so no window is opened.

use pushrod::dnd::{DragPayload, DragSource, DropTarget};
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::VisualState;
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
struct Row(u32);

#[derive(Default)]
struct App {
    dropped: Vec<(u32, Option<u32>)>,
    files: Vec<PathBuf>,
    clicks: u32,
    log: Vec<EngineEvent>,
}

/// Two "lists" side by side: `from` at (0,0)-(100,100), `to` at (150,0)-(250,100).
struct Fixture {
    engine: Engine<App>,
    from: u32,
    to: u32,
    image: u32,
}

fn fixture() -> Fixture {
    let mut engine: Engine<App> = Engine::new(size(400, 200), 30);
    let from = engine.add_widget(Panel::new().at(0, 0).size(100, 100));
    let to = engine.add_widget(Panel::new().at(150, 0).size(100, 100));
    let image = engine.add_widget(Panel::new().at(0, 150).size(20, 20));

    engine.widgets().get_mut(image).unwrap().set_hidden(true);
    engine.set_drag_source(
        from,
        DragSource::new(|| DragPayload::new(Row(3))).image(image),
    );
    engine.set_drop_target(to, DropTarget::new().accepts::<Row>());
    engine.on_drop(to, |app, _ui, payload| {
        if let Some(row) = payload.get::<Row>() {
            app.dropped.push((row.0, None));
        }
    });
    engine.on_drop(to, |app, _ui, payload| {
        if let Some(files) = payload.get::<Vec<PathBuf>>() {
            app.files.extend(files.iter().cloned());
        }
    });
    engine.on_engine_event(|app, _ui, event| app.log.push(event.clone()));

    Fixture {
        engine,
        from,
        to,
        image,
    }
}

fn mouse(f: &mut Fixture, app: &mut App, event: InputEvent) {
    f.engine.dispatch(event, app);
}

fn press(f: &mut Fixture, app: &mut App, x: i32, y: i32) {
    mouse(
        f,
        app,
        InputEvent::MouseMove {
            position: point(x, y),
        },
    );
    mouse(
        f,
        app,
        InputEvent::MouseDown {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
    );
}

fn move_to(f: &mut Fixture, app: &mut App, x: i32, y: i32) {
    mouse(
        f,
        app,
        InputEvent::MouseMove {
            position: point(x, y),
        },
    );
}

fn release(f: &mut Fixture, app: &mut App, x: i32, y: i32) {
    mouse(
        f,
        app,
        InputEvent::MouseUp {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
    );
}

fn drag_over_state(f: &mut Fixture) -> bool {
    let id = f.to;

    f.engine
        .widgets()
        .get(id)
        .unwrap()
        .state()
        .contains(VisualState::DRAG_OVER)
}

#[test]
fn dragging_from_one_list_to_another_delivers_the_payload() {
    let mut f = fixture();
    let mut app = App::default();

    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 100, 50);
    assert!(f.engine.widgets().is_dragging());
    move_to(&mut f, &mut app, 200, 50);
    release(&mut f, &mut app, 200, 50);

    assert_eq!(app.dropped, vec![(3, None)]);
    assert!(!f.engine.widgets().is_dragging());
    assert!(
        app.log
            .contains(&EngineEvent::DragStarted { source: f.from })
    );
    assert!(app.log.contains(&EngineEvent::DragEnded {
        source: f.from,
        target: Some(f.to),
        dropped: true,
    }));
}

#[test]
fn the_target_highlights_while_the_payload_is_over_it() {
    let mut f = fixture();
    let mut app = App::default();

    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 120, 50);
    assert!(!drag_over_state(&mut f));
    move_to(&mut f, &mut app, 200, 50);
    assert!(drag_over_state(&mut f));
    move_to(&mut f, &mut app, 120, 50);
    assert!(!drag_over_state(&mut f));
    move_to(&mut f, &mut app, 200, 50);
    release(&mut f, &mut app, 200, 50);
    assert!(!drag_over_state(&mut f));
}

#[test]
fn escape_cancels_the_drag() {
    let mut f = fixture();
    let mut app = App::default();

    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 200, 50);
    assert!(drag_over_state(&mut f));
    mouse(
        &mut f,
        &mut app,
        InputEvent::KeyDown {
            key: Key::Escape,
            modifiers: Modifiers::empty(),
            repeat: false,
        },
    );
    assert!(!f.engine.widgets().is_dragging());
    assert!(!drag_over_state(&mut f));
    release(&mut f, &mut app, 200, 50);
    assert!(app.dropped.is_empty());
    assert!(app.log.contains(&EngineEvent::DragEnded {
        source: f.from,
        target: None,
        dropped: false,
    }));
}

#[test]
fn releasing_outside_a_target_drops_nothing() {
    let mut f = fixture();
    let mut app = App::default();

    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 300, 150);
    release(&mut f, &mut app, 300, 150);
    assert!(app.dropped.is_empty());
    assert!(
        app.log
            .iter()
            .any(|e| matches!(e, EngineEvent::DragEnded { dropped: false, .. }))
    );
}

#[test]
fn a_target_that_does_not_accept_the_type_or_is_disabled_is_skipped() {
    let mut f = fixture();
    let mut app = App::default();

    f.engine
        .set_drop_target(f.to, DropTarget::new().accepts::<String>());
    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 200, 50);
    assert!(!drag_over_state(&mut f));
    release(&mut f, &mut app, 200, 50);
    assert!(app.dropped.is_empty());

    f.engine
        .set_drop_target(f.to, DropTarget::new().accepts::<Row>());
    f.engine.widgets().get_mut(f.to).unwrap().set_enabled(false);
    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 200, 50);
    release(&mut f, &mut app, 200, 50);
    assert!(app.dropped.is_empty());
}

#[test]
fn a_child_of_a_target_drops_on_the_target() {
    let mut f = fixture();
    let mut app = App::default();

    f.engine
        .add_child_widget(f.to, Panel::new().at(160, 10).size(30, 30));
    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 170, 20);
    release(&mut f, &mut app, 170, 20);
    assert_eq!(app.dropped.len(), 1);
}

#[test]
fn the_drag_image_follows_the_pointer_and_hides_afterwards() {
    let mut f = fixture();
    let mut app = App::default();

    assert!(f.engine.widgets().get(f.image).unwrap().is_hidden());
    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 60, 60);

    let widget = f.engine.widgets().get(f.image).unwrap();

    assert!(!widget.is_hidden());
    assert_eq!(widget.get_origin(), point(72, 72));
    move_to(&mut f, &mut app, 200, 50);
    assert_eq!(
        f.engine.widgets().get(f.image).unwrap().get_origin(),
        point(212, 62)
    );
    release(&mut f, &mut app, 200, 50);
    assert!(f.engine.widgets().get(f.image).unwrap().is_hidden());
}

#[test]
fn a_small_movement_is_not_a_drag_and_a_drag_is_not_a_click() {
    let mut f = fixture();
    let mut app = App::default();
    let button = f
        .engine
        .add_widget(Button::new("b").at(0, 120).size(60, 24));

    f.engine.on_click(button, |app, _ui| app.clicks += 1);
    f.engine
        .set_drag_source(button, DragSource::new(|| DragPayload::new(Row(1))));

    // A press and release with a 2 pixel wiggle is a click.
    press(&mut f, &mut app, 10, 130);
    move_to(&mut f, &mut app, 12, 130);
    assert!(!f.engine.widgets().is_dragging());
    release(&mut f, &mut app, 12, 130);
    assert_eq!(app.clicks, 1);

    // A real drag over the button's own area is not a click.
    press(&mut f, &mut app, 10, 130);
    move_to(&mut f, &mut app, 40, 130);
    move_to(&mut f, &mut app, 10, 130);
    release(&mut f, &mut app, 10, 130);
    assert_eq!(app.clicks, 1);
}

#[test]
fn pointer_events_do_not_reach_other_widgets_during_a_drag() {
    let mut f = fixture();
    let mut app = App::default();
    let button = f
        .engine
        .add_widget(Button::new("b").at(300, 100).size(60, 24));

    f.engine.on_click(button, |app, _ui| app.clicks += 1);
    press(&mut f, &mut app, 50, 50);
    move_to(&mut f, &mut app, 310, 110);
    release(&mut f, &mut app, 310, 110);
    assert_eq!(app.clicks, 0);
}

#[test]
fn files_dropped_from_the_os_reach_an_accepting_target() {
    let mut f = fixture();
    let mut app = App::default();

    f.engine
        .set_drop_target(f.to, DropTarget::new().accepts::<Vec<PathBuf>>());
    f.engine.dispatch(
        InputEvent::FilesDropped {
            position: point(200, 50),
            paths: vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")],
        },
        &mut app,
    );
    assert_eq!(app.files.len(), 2);
    assert!(app.log.contains(&EngineEvent::FilesDropped {
        target: Some(f.to),
        count: 2,
    }));

    f.engine.dispatch(
        InputEvent::FilesDropped {
            position: point(20, 20),
            paths: vec![PathBuf::from("/tmp/c.txt")],
        },
        &mut app,
    );
    assert_eq!(app.files.len(), 2);
    assert!(app.log.contains(&EngineEvent::FilesDropped {
        target: None,
        count: 1,
    }));
}

#[test]
fn unknown_widgets_cannot_be_registered() {
    let mut f = fixture();

    assert!(
        !f.engine
            .set_drag_source(999, DragSource::new(|| DragPayload::new(1u8)))
    );
    assert!(!f.engine.set_drop_target(999, DropTarget::new()));
}
