// Button behavior-flag tests: default, cancel, toggle, activate-on-press, dismiss dialog.
// Everything runs headless through the WidgetCache; nothing opens a window.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{EngineEvent, Event};
use pushrod::geometry::point;
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::{Behavior, Widget};
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;

fn down(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn up(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseUp {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn motion(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseMove {
        position: point(x, y),
    }
}

fn key(key: Key) -> InputEvent {
    InputEvent::KeyDown {
        key,
        modifiers: Modifiers::empty(),
        repeat: false,
    }
}

/// A cache with a 100x20 button at (10, 10) inside a root panel.
fn with_button(button: Button) -> (WidgetCache, u32) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 200)));

    let id = cache.add_child(0, Box::new(button.at(10, 10).size(100, 20)));

    (cache, id)
}

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

fn count(events: &[Event], wanted: fn(&Event) -> bool) -> usize {
    events.iter().filter(|e| wanted(e)).count()
}

fn is_click(e: &Event) -> bool {
    matches!(e, Event::Clicked { .. })
}

/// A full click: move in, press, release; returns every event produced.
fn click(cache: &mut WidgetCache) -> Vec<Event> {
    let mut events = cache.handle_event(motion(20, 20));

    events.extend(cache.handle_event(down(20, 20)));
    events.extend(cache.handle_event(up(20, 20)));
    events
}

#[test]
fn a_plain_button_clicks_on_release_and_is_unchanged() {
    let (mut cache, _) = with_button(Button::new("OK"));
    let mut events = cache.handle_event(motion(20, 20));

    events.extend(cache.handle_event(down(20, 20)));
    assert_eq!(count(&events, is_click), 0);
    assert_eq!(count(&cache.handle_event(up(20, 20)), is_click), 1);
}

#[test]
fn default_button_draws_a_two_pixel_ring() {
    let ring = |button: Button| {
        painted(&button.size(100, 20))
            .iter()
            .filter(|c| matches!(c, PaintCall::Rect(..)))
            .count()
    };

    assert_eq!(ring(Button::new("OK")), 1);
    assert_eq!(ring(Button::new("OK").default_action()), 2);
}

#[test]
fn return_activates_the_default_button_and_escape_the_cancel_button() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 200)));

    let ok = cache.add_child(0, Box::new(Button::new("OK").default_action().at(10, 10)));
    let cancel = cache.add_child(
        0,
        Box::new(Button::new("Cancel").cancel_action().at(10, 50)),
    );

    let clicked: Vec<u32> = cache
        .handle_event(key(Key::Return))
        .iter()
        .filter_map(|e| match e {
            Event::Clicked { id, .. } => Some(*id),
            _ => None,
        })
        .collect();

    assert_eq!(clicked, vec![ok]);

    let clicked: Vec<u32> = cache
        .handle_event(key(Key::Escape))
        .iter()
        .filter_map(|e| match e {
            Event::Clicked { id, .. } => Some(*id),
            _ => None,
        })
        .collect();

    assert_eq!(clicked, vec![cancel]);
}

#[test]
fn a_disabled_default_button_ignores_return() {
    let (mut cache, id) = with_button(Button::new("OK").default_action());

    cache.get_mut(id).unwrap().set_enabled(false);

    assert_eq!(count(&cache.handle_event(key(Key::Return)), is_click), 0);
}

#[test]
fn toggle_button_stays_pressed_and_emits_toggled() {
    let (mut cache, id) = with_button(Button::new("Bold").toggle());
    let toggles = |events: &[Event]| -> Vec<bool> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Toggled { checked, .. } => Some(*checked),
                _ => None,
            })
            .collect()
    };

    let events = click(&mut cache);
    assert_eq!(toggles(&events), vec![true]);
    assert_eq!(
        count(&events, is_click),
        0,
        "a toggle emits Toggled, not Clicked"
    );

    let button = cache.get_as::<Button>(id).unwrap();
    assert!(button.is_checked());
    assert!(!button.is_selected(), "released, but still drawn inverted");
    assert_eq!(
        painted(button).first(),
        Some(&PaintCall::Fill(Color::BLACK))
    );

    assert_eq!(toggles(&click(&mut cache)), vec![false]);
    assert!(!cache.get_as::<Button>(id).unwrap().is_checked());
}

#[test]
fn set_checked_changes_the_look_without_an_event() {
    let mut button = Button::new("Bold").toggle().size(100, 20);

    button.set_invalidated(false);
    button.set_checked(true);
    assert!(button.is_checked() && button.is_invalidated());
    assert_eq!(
        painted(&button).first(),
        Some(&PaintCall::Fill(Color::BLACK))
    );

    button.set_invalidated(false);
    button.set_checked(true);
    assert!(!button.is_invalidated());
}

#[test]
fn activate_on_press_clicks_on_press_and_not_on_release() {
    let (mut cache, _) = with_button(Button::new("Go").activate_on_press());
    let mut pressed = cache.handle_event(motion(20, 20));

    pressed.extend(cache.handle_event(down(20, 20)));
    assert_eq!(count(&pressed, is_click), 1);
    assert_eq!(count(&cache.handle_event(up(20, 20)), is_click), 0);
}

#[test]
fn a_toggle_that_activates_on_press_flips_on_press_only() {
    let (mut cache, id) = with_button(Button::new("Go").toggle().activate_on_press());

    cache.handle_event(motion(20, 20));
    cache.handle_event(down(20, 20));
    assert!(cache.get_as::<Button>(id).unwrap().is_checked());
    cache.handle_event(up(20, 20));
    assert!(cache.get_as::<Button>(id).unwrap().is_checked());
}

#[test]
fn dismiss_button_asks_for_the_dialog_to_close_once_per_activation() {
    let (mut cache, id) = with_button(Button::new("OK").dismisses_dialog());
    let events = click(&mut cache);
    let dismissals: Vec<u32> = events
        .iter()
        .filter_map(|e| match e {
            Event::Engine(EngineEvent::DialogDismissRequested { id }) => Some(*id),
            _ => None,
        })
        .collect();

    assert_eq!(dismissals, vec![id]);
    assert_eq!(count(&events, is_click), 1, "it still reports its click");
}

#[test]
fn a_button_that_is_default_and_dismisses_reports_through_return() {
    let (mut cache, id) = with_button(Button::new("OK").default_action().dismisses_dialog());
    let events = cache.handle_event(key(Key::Return));

    assert!(events.iter().any(|e| matches!(
        e,
        Event::Engine(EngineEvent::DialogDismissRequested { id: dismissed }) if *dismissed == id
    )));
}

#[test]
fn builders_set_the_matching_flags() {
    let button = Button::new("x")
        .default_action()
        .cancel_action()
        .toggle()
        .activate_on_press()
        .dismisses_dialog();

    for flag in [
        Behavior::DEFAULT_ACTION,
        Behavior::CANCEL_ACTION,
        Behavior::TOGGLE,
        Behavior::ACTIVATE_ON_PRESS,
        Behavior::DISMISSES_DIALOG,
    ] {
        assert!(button.behavior().contains(flag));
    }

    assert!(!Button::new("x").is_checked());
}

/// Counts the `DialogDismissRequested` events among `events` and the cache's queued events.
fn dismiss_requests(cache: &mut WidgetCache, events: &[Event]) -> usize {
    let queued = cache.take_pending_events();

    events
        .iter()
        .chain(queued.iter())
        .filter(|e| matches!(e, Event::Engine(EngineEvent::DialogDismissRequested { .. })))
        .count()
}

#[test]
fn a_dismiss_button_closes_its_modal_dialog_with_one_request() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 200)));

    let dialog = cache.add_child(0, Box::new(Panel::new().at(0, 0).size(200, 100)));
    let ok = cache.add_child(
        dialog,
        Box::new(
            Button::new("OK")
                .at(10, 10)
                .size(100, 20)
                .default_action()
                .dismisses_dialog(),
        ),
    );

    // By mouse.
    cache.open_modal(dialog).unwrap();
    // Buttons do not take focus, so nothing has it; Return still reaches the dialog's default.
    assert_eq!(cache.focused(), None);
    assert!(!cache.is_inert(ok));

    let events = click(&mut cache);

    assert_eq!(dismiss_requests(&mut cache, &events), 1);
    assert_eq!(cache.top_modal(), None);
    assert!(cache.get(dialog).unwrap().is_hidden());

    // By Return: `Activated`, `Clicked`, and the button's request close the dialog once.
    cache.open_modal(dialog).unwrap();
    cache.take_pending_events();

    let events = cache.handle_event(key(Key::Return));
    let queued = cache.take_pending_events();
    let closed = queued
        .iter()
        .filter(|e| matches!(e, Event::Engine(EngineEvent::ModalClosed { .. })))
        .count();
    let requests = events
        .iter()
        .chain(queued.iter())
        .filter(|e| matches!(e, Event::Engine(EngineEvent::DialogDismissRequested { .. })))
        .count();

    assert_eq!((requests, closed), (1, 1));
    assert_eq!(cache.top_modal(), None);
}
