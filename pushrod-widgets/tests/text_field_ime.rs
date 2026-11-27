// TextField input-method (IME) tests: composition (pre-edit) text is drawn underlined at the caret,
// stays out of the field's text, and is committed by the typed text that follows.  Sample text is
// fictional.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::event::{EngineEvent, Event};
use pushrod::geometry::{Rect, point};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text_field::TextField;

/// The calls one paint of the field makes.
fn painted(cache: &WidgetCache, id: u32) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();

    cache.get(id).unwrap().paint(&mut p);
    p.calls
}

/// A cache with the field at window position (10, 10), painted once and focused by a click.
fn focused(field: TextField) -> (WidgetCache, u32) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(400, 200)));

    let id = cache.add_child(0, Box::new(field.at(10, 10)));

    painted(&cache, id);
    cache.handle_event(InputEvent::MouseDown {
        position: point(12, 20),
        button: MouseButton::Left,
        clicks: 1,
    });
    cache.handle_event(InputEvent::MouseUp {
        position: point(12, 20),
        button: MouseButton::Left,
        clicks: 1,
    });
    cache.handle_event(InputEvent::KeyDown {
        key: Key::End,
        modifiers: Modifiers::empty(),
        repeat: false,
    });
    painted(&cache, id);
    (cache, id)
}

/// Sends the input method's composition text and repaints.
fn compose(cache: &mut WidgetCache, id: u32, text: &str, start: i32, length: i32) -> Vec<Event> {
    let events = cache.handle_event(InputEvent::Composition {
        text: text.to_string(),
        start,
        length,
    });

    painted(cache, id);
    events
}

/// Commits text, as the input method does when the user accepts a candidate.
fn commit(cache: &mut WidgetCache, id: u32, text: &str) -> Vec<Event> {
    let events = cache.handle_event(InputEvent::Text(text.to_string()));

    painted(cache, id);
    events
}

fn field(cache: &WidgetCache, id: u32) -> &TextField {
    cache.get_as::<TextField>(id).unwrap()
}

fn shows(calls: &[PaintCall], text: &str) -> bool {
    calls
        .iter()
        .any(|call| matches!(call, PaintCall::Text(_, shown, _) if shown == text))
}

fn underlined(calls: &[PaintCall]) -> bool {
    calls.iter().any(|call| matches!(call, PaintCall::Line(..)))
}

fn engine_events(events: &mut WidgetCache) -> Vec<EngineEvent> {
    events
        .take_pending_events()
        .into_iter()
        .filter_map(|event| match event {
            Event::Engine(engine) => Some(engine),
            _ => None,
        })
        .collect()
}

#[test]
fn composition_text_is_drawn_underlined_but_is_not_the_fields_text() {
    let (mut cache, id) = focused(TextField::new());
    let events = compose(&mut cache, id, "にほん", 3, 0);

    assert_eq!(field(&cache, id).text(), "");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TextChanged { .. }))
    );
    assert!(shows(&painted(&cache, id), "にほん"));
    assert!(underlined(&painted(&cache, id)));
}

#[test]
fn each_update_replaces_the_composition() {
    let (mut cache, id) = focused(TextField::new());

    compose(&mut cache, id, "に", 1, 0);
    compose(&mut cache, id, "にほ", 2, 0);

    let calls = painted(&cache, id);

    assert!(shows(&calls, "にほ"));
    assert!(!shows(&calls, "に"));
}

#[test]
fn committing_inserts_the_text_once_and_reports_one_change() {
    let (mut cache, id) = focused(TextField::new());

    compose(&mut cache, id, "にほん", 3, 0);
    compose(&mut cache, id, "", 0, 0);

    let events = commit(&mut cache, id, "日");
    let changes = events
        .iter()
        .filter(|event| matches!(event, Event::TextChanged { .. }))
        .count();

    assert_eq!(field(&cache, id).text(), "日");
    assert_eq!(changes, 1);
    assert!(!underlined(&painted(&cache, id)), "the composition is gone");
}

#[test]
fn an_empty_update_cancels_the_composition() {
    let (mut cache, id) = focused(TextField::new().with_text("ab"));

    compose(&mut cache, id, "に", 1, 0);
    compose(&mut cache, id, "", 0, 0);

    assert_eq!(field(&cache, id).text(), "ab");
    assert!(!underlined(&painted(&cache, id)));
}

#[test]
fn the_composition_appears_at_the_caret_inside_the_text() {
    let (mut cache, id) = focused(TextField::new().with_text("ab"));

    cache.handle_event(InputEvent::KeyDown {
        key: Key::Left,
        modifiers: Modifiers::empty(),
        repeat: false,
    });
    compose(&mut cache, id, "x", 1, 0);
    assert!(shows(&painted(&cache, id), "axb"));

    commit(&mut cache, id, "X");
    assert_eq!(field(&cache, id).text(), "aXb");
}

#[test]
fn a_committed_composition_replaces_the_selection() {
    let (mut cache, id) = focused(TextField::new().with_text("abc"));

    cache.handle_event(InputEvent::KeyDown {
        key: Key::Char('a'),
        modifiers: Modifiers::CTRL,
        repeat: false,
    });
    compose(&mut cache, id, "に", 1, 0);
    commit(&mut cache, id, "日");

    assert_eq!(field(&cache, id).text(), "日");
}

#[test]
fn the_input_methods_selection_is_underlined_more_thickly() {
    let (mut cache, id) = focused(TextField::new());

    compose(&mut cache, id, "にほん", 1, 2);

    let thick = painted(&cache, id)
        .into_iter()
        .filter(|call| matches!(call, PaintCall::FillRect(rect, _) if rect.h == 2))
        .count();

    assert_eq!(thick, 1);
}

#[test]
fn the_caret_rectangle_follows_the_input_methods_cursor() {
    let (mut cache, id) = focused(TextField::new());
    let before: Rect = cache.focused_text_input_rect().unwrap();

    compose(&mut cache, id, "にほん", 3, 0);

    let after = cache.focused_text_input_rect().unwrap();

    assert!(after.x > before.x, "the caret moved past the composition");
    assert_eq!(after.y, before.y);
    assert_eq!(after.w, 1);
}

#[test]
fn a_password_field_never_shows_the_composition() {
    let (mut cache, id) = focused(TextField::new().password());

    compose(&mut cache, id, "にほん", 3, 0);

    let calls = painted(&cache, id);

    assert!(!shows(&calls, "にほん"));
    assert!(!underlined(&calls));
}

#[test]
fn the_max_length_still_limits_the_committed_text() {
    let (mut cache, id) = focused(TextField::new().max_length(2));

    compose(&mut cache, id, "にほん", 3, 0);
    commit(&mut cache, id, "日本語");

    assert_eq!(field(&cache, id).text(), "日本");
}

#[test]
fn losing_the_focus_drops_the_composition() {
    let (mut cache, id) = focused(TextField::new());

    compose(&mut cache, id, "にほん", 3, 0);
    cache.clear_focus();
    painted(&cache, id);

    assert_eq!(field(&cache, id).text(), "");
    assert!(!underlined(&painted(&cache, id)));
}

#[test]
fn the_engine_reports_start_update_and_commit() {
    let (mut cache, id) = focused(TextField::new());

    engine_events(&mut cache);
    compose(&mut cache, id, "に", 1, 0);
    compose(&mut cache, id, "にほ", 2, 0);
    compose(&mut cache, id, "", 0, 0);
    commit(&mut cache, id, "日");

    let kinds = engine_events(&mut cache);

    assert_eq!(
        kinds,
        vec![
            EngineEvent::CompositionStarted { id },
            EngineEvent::CompositionUpdated {
                id,
                text: "に".to_string(),
                cursor: Some((1, 0))
            },
            EngineEvent::CompositionUpdated {
                id,
                text: "にほ".to_string(),
                cursor: Some((2, 0))
            },
            EngineEvent::CompositionEnded {
                id,
                committed: true
            },
        ]
    );
}
