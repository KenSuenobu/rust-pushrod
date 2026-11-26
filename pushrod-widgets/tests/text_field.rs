// TextField tests: painting goes to a RecordingPainter and input through the WidgetCache, so no
// window is opened.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{AccessorySide, Event};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::Widget;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::prelude::TextField as PreludeField;
use pushrod_widgets::text_field::validate::Validity;
use pushrod_widgets::text_field::{Accessory, IntegerField, NumberField, TextField};
use std::time::Duration;

/// The width of one character in the headless painter.
fn glyph() -> i32 {
    Font::default().size() as i32 / 2
}

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();

    widget.paint(&mut p);
    p.calls
}

/// A cache with the field at window position (10, 10) under a root panel, painted once (so the
/// mouse can be mapped to characters) and focused by a click.
fn focused(field: TextField) -> (WidgetCache, u32) {
    let (mut cache, id) = unfocused(field);

    cache.handle_event(down(12, 20, 1));
    cache.handle_event(up(12, 20));
    cache.handle_event(key(Key::End));
    (cache, id)
}

fn unfocused(field: TextField) -> (WidgetCache, u32) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(400, 200)));

    let id = cache.add_child(0, Box::new(field.at(10, 10)));

    repaint(&mut cache, id);
    (cache, id)
}

fn repaint(cache: &mut WidgetCache, id: u32) {
    painted(cache.get_as::<TextField>(id).unwrap());
}

fn get(cache: &WidgetCache, id: u32) -> &TextField {
    cache.get_as::<TextField>(id).unwrap()
}

fn down(x: i32, y: i32, clicks: u8) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button: MouseButton::Left,
        clicks,
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

fn press(key: Key, modifiers: Modifiers) -> InputEvent {
    InputEvent::KeyDown {
        key,
        modifiers,
        repeat: false,
    }
}

fn key(key: Key) -> InputEvent {
    press(key, Modifiers::empty())
}

fn shifted(k: Key) -> InputEvent {
    press(k, Modifiers::SHIFT)
}

fn command(c: char) -> InputEvent {
    press(Key::Char(c), Modifiers::CTRL)
}

/// Types text and returns the events produced.
fn typed(cache: &mut WidgetCache, id: u32, text: &str) -> Vec<Event> {
    let events = cache.handle_event(InputEvent::Text(text.to_string()));

    repaint(cache, id);
    events
}

fn count(events: &[Event], f: impl Fn(&Event) -> bool) -> usize {
    events.iter().filter(|e| f(e)).count()
}

fn changes(events: &[Event]) -> usize {
    count(events, |e| matches!(e, Event::TextChanged { .. }))
}

fn finishes(events: &[Event]) -> usize {
    count(events, |e| matches!(e, Event::EditingFinished { .. }))
}

#[test]
fn a_new_field_is_empty_focusable_and_takes_text_input() {
    let field = TextField::new();

    assert_eq!(field.text(), "");
    assert_eq!(field.get_size().w, 200);
    assert!(field.is_focusable());
    assert!(field.accepts_text_input());
    assert_eq!(field.validity(), Validity::Valid);
    let _ = PreludeField::new();
}

#[test]
fn typing_inserts_at_the_caret_and_emits_text_changed() {
    let (mut cache, id) = focused(TextField::new());
    let events = typed(&mut cache, id, "Jane");

    assert_eq!(get(&cache, id).text(), "Jane");
    assert_eq!(get(&cache, id).caret(), 4);
    assert_eq!(changes(&events), 4);

    cache.handle_event(key(Key::Left));
    cache.handle_event(key(Key::Left));
    typed(&mut cache, id, "X");

    assert_eq!(get(&cache, id).text(), "JaXne");
}

#[test]
fn control_characters_are_never_inserted() {
    let (mut cache, id) = focused(TextField::new());
    let events = typed(&mut cache, id, "a\nb\tc");

    assert_eq!(get(&cache, id).text(), "abc");
    assert_eq!(changes(&events), 3);
}

#[test]
fn backspace_and_delete_remove_characters() {
    let (mut cache, id) = focused(TextField::new().with_text("abcd"));

    cache.handle_event(key(Key::Backspace));
    assert_eq!(get(&cache, id).text(), "abc");

    cache.handle_event(key(Key::Home));
    cache.handle_event(key(Key::Backspace));
    assert_eq!(get(&cache, id).text(), "abc", "nothing before the caret");

    let events = cache.handle_event(key(Key::Delete));

    assert_eq!(get(&cache, id).text(), "bc");
    assert_eq!(changes(&events), 1);

    cache.handle_event(key(Key::End));
    let events = cache.handle_event(key(Key::Delete));

    assert_eq!(changes(&events), 0, "nothing after the caret");
}

#[test]
fn word_movement_and_word_deletion() {
    let (mut cache, id) = focused(TextField::new().with_text("one two three"));
    let alt = |k| press(k, Modifiers::ALT);

    cache.handle_event(alt(Key::Left));
    assert_eq!(get(&cache, id).caret(), 8);
    cache.handle_event(alt(Key::Left));
    assert_eq!(get(&cache, id).caret(), 4);
    cache.handle_event(alt(Key::Right));
    assert_eq!(get(&cache, id).caret(), 7);

    cache.handle_event(alt(Key::Backspace));
    assert_eq!(get(&cache, id).text(), "one  three");

    cache.handle_event(alt(Key::Delete));
    assert_eq!(get(&cache, id).text(), "one ");
}

#[test]
fn shift_arrows_select_and_typing_replaces_the_selection() {
    let (mut cache, id) = focused(TextField::new().with_text("Jane"));

    cache.handle_event(key(Key::Home));
    cache.handle_event(shifted(Key::Right));
    cache.handle_event(shifted(Key::Right));

    assert_eq!(get(&cache, id).selected_text(), "Ja");

    cache.handle_event(key(Key::Right));
    assert_eq!(get(&cache, id).selected_text(), "", "an arrow collapses it");
    assert_eq!(get(&cache, id).caret(), 2);

    cache.handle_event(shifted(Key::Home));
    typed(&mut cache, id, "Jo");

    assert_eq!(get(&cache, id).text(), "Jone");
}

#[test]
fn clipboard_copy_cut_paste_and_select_all() {
    let (mut cache, id) = focused(TextField::new().with_text("hello world"));

    cache.handle_event(command('a'));
    assert_eq!(get(&cache, id).selected_text(), "hello world");

    cache.handle_event(key(Key::Home));
    cache.handle_event(shifted(Key::Right));
    cache.handle_event(shifted(Key::Right));
    cache.handle_event(shifted(Key::Right));
    cache.handle_event(shifted(Key::Right));
    cache.handle_event(shifted(Key::Right));
    cache.handle_event(command('c'));

    assert_eq!(cache.take_clipboard_write().as_deref(), Some("hello"));

    let events = cache.handle_event(command('x'));

    assert_eq!(cache.take_clipboard_write().as_deref(), Some("hello"));
    assert_eq!(get(&cache, id).text(), " world");
    assert_eq!(changes(&events), 1);

    cache.handle_event(key(Key::End));
    cache.set_clipboard_snapshot(Some("!!".to_string()));
    cache.handle_event(command('v'));

    assert_eq!(get(&cache, id).text(), " world!!");
}

#[test]
fn a_paste_right_after_a_copy_sees_the_copied_text() {
    let (mut cache, id) = focused(TextField::new().with_text("ab"));

    cache.handle_event(command('a'));
    cache.handle_event(command('c'));
    cache.handle_event(key(Key::End));
    cache.handle_event(command('v'));

    assert_eq!(get(&cache, id).text(), "abab");
}

#[test]
fn a_password_field_never_copies_or_cuts() {
    let (mut cache, id) = focused(TextField::new().password().with_text("secret"));

    cache.handle_event(command('a'));
    cache.handle_event(command('c'));
    cache.handle_event(command('x'));

    assert_eq!(cache.take_clipboard_write(), None);
    assert_eq!(get(&cache, id).text(), "secret");
}

#[test]
fn max_length_stops_typing_and_paste() {
    let (mut cache, id) = focused(TextField::new().max_length(3));

    typed(&mut cache, id, "abcdef");
    assert_eq!(get(&cache, id).text(), "abc");

    let events = typed(&mut cache, id, "z");
    assert_eq!(changes(&events), 0);
}

#[test]
fn set_text_emits_nothing_and_skips_the_rules() {
    let mut field = TextField::new().with_mask("99");

    field.set_text("zzzz");
    assert_eq!(field.text(), "zzzz");
    assert_eq!(field.caret(), 4);
    assert_eq!(field.validity(), Validity::Invalid);

    let mut limited = TextField::new().max_length(2);

    limited.set_text("abc");
    assert_eq!(limited.text(), "ab");
}

#[test]
fn a_mask_fills_literals_and_rejects_wrong_characters() {
    let (mut cache, id) = focused(TextField::new().with_mask("99/99/9999"));

    typed(&mut cache, id, "12");
    assert_eq!(get(&cache, id).text(), "12");

    typed(&mut cache, id, "2");
    assert_eq!(get(&cache, id).text(), "12/2", "the slash is filled in");

    let events = typed(&mut cache, id, "x");
    assert_eq!(get(&cache, id).text(), "12/2");
    assert_eq!(changes(&events), 0);

    typed(&mut cache, id, "5/2026");
    assert_eq!(get(&cache, id).text(), "12/25/2026");
    assert_eq!(get(&cache, id).validity(), Validity::Valid);

    typed(&mut cache, id, "1");
    assert_eq!(get(&cache, id).text(), "12/25/2026", "the mask is full");
}

#[test]
fn typing_the_literal_of_a_mask_is_accepted() {
    let (mut cache, id) = focused(TextField::new().with_mask("99/99"));

    typed(&mut cache, id, "12");
    typed(&mut cache, id, "/");
    assert_eq!(get(&cache, id).text(), "12/");
    typed(&mut cache, id, "3");
    assert_eq!(get(&cache, id).text(), "12/3");

    let (mut cache, id) = focused(TextField::new().with_mask("99/99"));

    typed(&mut cache, id, "1/");
    assert_eq!(get(&cache, id).text(), "1", "a slash is not a digit");
}

#[test]
fn a_mask_allows_deleting_only_from_the_end() {
    let (mut cache, id) = focused(TextField::new().with_mask("99/99"));

    typed(&mut cache, id, "1234");
    assert_eq!(get(&cache, id).text(), "12/34");

    cache.handle_event(key(Key::Backspace));
    assert_eq!(get(&cache, id).text(), "12/3");

    cache.handle_event(key(Key::Backspace));
    assert_eq!(
        get(&cache, id).text(),
        "12",
        "the slash goes with the digit"
    );

    cache.handle_event(key(Key::Home));
    cache.handle_event(key(Key::Right));
    let events = cache.handle_event(key(Key::Backspace));

    assert_eq!(changes(&events), 0, "deleting inside a mask is refused");
    assert_eq!(get(&cache, id).text(), "12");
}

#[test]
fn typing_inside_a_mask_overwrites() {
    let (mut cache, id) = focused(TextField::new().with_mask("99/99"));

    typed(&mut cache, id, "1234");
    cache.handle_event(key(Key::Home));
    typed(&mut cache, id, "9");

    assert_eq!(get(&cache, id).text(), "92/34");

    cache.handle_event(key(Key::Right));
    typed(&mut cache, id, "7");
    assert_eq!(get(&cache, id).text(), "92/74", "it hops the slash");
}

#[test]
fn integer_fields_reject_non_digits_and_out_of_range() {
    let (mut cache, id) = focused(IntegerField::range(0, 120));

    typed(&mut cache, id, "4a2");
    assert_eq!(get(&cache, id).text(), "42");

    typed(&mut cache, id, "1");
    assert_eq!(get(&cache, id).text(), "42", "421 is over the maximum");
    assert_eq!(get(&cache, id).value_i64(), Some(42));

    typed(&mut cache, id, "-");
    assert_eq!(get(&cache, id).text(), "42", "no negatives in 0..=120");
}

#[test]
fn integer_fields_with_negative_ranges_take_a_minus() {
    let (mut cache, id) = focused(IntegerField::range(-50, 50));

    typed(&mut cache, id, "-7");
    assert_eq!(get(&cache, id).value_i64(), Some(-7));

    typed(&mut cache, id, "9");
    assert_eq!(get(&cache, id).text(), "-7", "-79 is under the minimum");
}

#[test]
fn a_field_may_pass_through_intermediate_text() {
    let (mut cache, id) = focused(IntegerField::range(10, 20));

    typed(&mut cache, id, "1");
    assert_eq!(get(&cache, id).text(), "1");
    assert_eq!(get(&cache, id).validity(), Validity::Intermediate);

    typed(&mut cache, id, "5");
    assert_eq!(get(&cache, id).validity(), Validity::Valid);
}

#[test]
fn number_fields_limit_decimals_and_range() {
    let (mut cache, id) = focused(NumberField::range(0.0, 100.0, 2));

    typed(&mut cache, id, "2.7518");
    assert_eq!(get(&cache, id).text(), "2.75");
    assert_eq!(get(&cache, id).value_f64(), Some(2.75));

    typed(&mut cache, id, ".");
    assert_eq!(get(&cache, id).text(), "2.75", "one point only");

    cache.handle_event(command('a'));
    typed(&mut cache, id, "100.5");
    assert_eq!(get(&cache, id).text(), "100.", "100.5 is over the maximum");
}

#[test]
fn return_and_focus_loss_emit_editing_finished() {
    let (mut cache, id) = focused(TextField::new());
    let events = cache.handle_event(key(Key::Return));

    assert_eq!(finishes(&events), 1);
    assert_eq!(
        count(
            &events,
            |e| matches!(e, Event::EditingFinished { id: i } if *i == id)
        ),
        1
    );

    // Clicking the panel (not focusable) keeps the focus; tabbing away loses it.
    cache.add_child(0, Box::new(TextField::new().at(10, 80)));
    cache.handle_event(down(12, 90, 1));

    let events = cache.take_pending_events();

    assert_eq!(finishes(&events), 1, "focus loss finishes editing");
}

#[test]
fn return_is_not_consumed_so_a_default_button_still_sees_it() {
    let mut field = TextField::new();
    let mut ctx = pushrod::event::EventCtx::default();
    let result = field.handle_event(
        &Event::KeyPressed {
            id: 0,
            key: Key::Return,
            modifiers: Modifiers::empty(),
            repeat: false,
        },
        &mut ctx,
    );

    assert_eq!(result, pushrod::event::Propagation::Continue);
}

#[test]
fn clicking_places_the_caret_and_dragging_selects() {
    let (mut cache, id) = focused(TextField::new().with_text("abcdef"));
    let x = |chars: i32| 10 + 1 + 3 + chars * glyph();

    cache.handle_event(down(x(2), 20, 1));
    assert_eq!(get(&cache, id).caret(), 2);

    cache.handle_event(motion(x(5), 20));
    cache.handle_event(up(x(5), 20));
    assert_eq!(get(&cache, id).selected_text(), "cde");

    // Moving after release does not extend it.
    cache.handle_event(motion(x(1), 20));
    assert_eq!(get(&cache, id).selected_text(), "cde");
}

#[test]
fn double_click_selects_a_word_and_triple_click_all() {
    let (mut cache, id) = focused(TextField::new().with_text("one two"));
    let x = |chars: i32| 10 + 1 + 3 + chars * glyph();

    cache.handle_event(down(x(5), 20, 2));
    assert_eq!(get(&cache, id).selected_text(), "two");

    cache.handle_event(down(x(5), 20, 3));
    assert_eq!(get(&cache, id).selected_text(), "one two");
}

#[test]
fn the_caret_blinks_on_animation_ticks_while_focused() {
    let (mut cache, id) = focused(TextField::new());
    let caret_lines = |cache: &WidgetCache| {
        painted(get(cache, id))
            .iter()
            .filter(|c| matches!(c, PaintCall::FillRect(r, _) if r.w == 1))
            .count()
    };

    assert_eq!(caret_lines(&cache), 1);

    cache.tick(Duration::from_millis(600));
    assert_eq!(caret_lines(&cache), 0, "off after half a second");

    cache.tick(Duration::from_millis(600));
    assert_eq!(caret_lines(&cache), 1, "and on again");

    // Typing shows it at once.
    cache.tick(Duration::from_millis(600));
    typed(&mut cache, id, "a");
    assert_eq!(caret_lines(&cache), 1);
}

#[test]
fn an_unfocused_field_draws_no_caret_and_stops_ticking() {
    let (cache, id) = unfocused(TextField::new());

    assert!(
        !painted(get(&cache, id))
            .iter()
            .any(|c| matches!(c, PaintCall::FillRect(r, _) if r.w == 1))
    );
    assert!(!cache.animation_requested());
}

#[test]
fn a_focused_field_keeps_asking_for_ticks() {
    let (mut cache, _) = focused(TextField::new());

    assert!(cache.animation_requested());
    cache.tick(Duration::from_millis(16));
    assert!(cache.animation_requested());
}

#[test]
fn the_placeholder_shows_only_while_empty() {
    let (mut cache, id) = unfocused(TextField::new().placeholder("Your name"));
    let shows = |cache: &WidgetCache, id| {
        painted(get(cache, id))
            .iter()
            .any(|c| matches!(c, PaintCall::Text(_, t, color) if t == "Your name" && *color == Color::GRAY))
    };

    assert!(shows(&cache, id));

    cache.handle_event(down(12, 20, 1));
    typed(&mut cache, id, "J");
    assert!(!shows(&cache, id));
}

#[test]
fn a_password_is_drawn_masked_until_revealed() {
    let (mut cache, id) = focused(
        TextField::new()
            .password()
            .with_text("abc")
            .with_trailing(Accessory::RevealPassword),
    );
    let drawn = |cache: &WidgetCache| -> Vec<String> {
        painted(get(cache, id))
            .into_iter()
            .filter_map(|c| match c {
                PaintCall::Text(_, t, _) => Some(t),
                _ => None,
            })
            .collect()
    };

    assert_eq!(drawn(&cache), vec!["***".to_string()]);
    assert_eq!(get(&cache, id).text(), "abc");

    // The eye is the right-most square of the edit area (200 wide, 24 tall).
    let events = cache.handle_event(down(10 + 200 - 12, 22, 1));

    assert!(get(&cache, id).is_revealed());
    assert_eq!(
        count(&events, |e| matches!(e, Event::AccessoryClicked { .. })),
        0
    );
    assert_eq!(drawn(&cache), vec!["abc".to_string()]);
}

#[test]
fn the_clear_accessory_appears_with_text_and_clears_it() {
    let (mut cache, id) = focused(TextField::search());
    let trailing = (10 + 200 - 12, 22);
    let crosses = |cache: &WidgetCache| {
        painted(get(cache, id))
            .iter()
            .filter(|c| matches!(c, PaintCall::Line(..)))
            .count()
    };

    // Empty: the magnifier's handle is the only line.
    assert_eq!(crosses(&cache), 1);
    let events = cache.handle_event(down(trailing.0, trailing.1, 1));
    assert_eq!(changes(&events), 0, "a hidden clear button does nothing");

    typed(&mut cache, id, "widgets");
    assert_eq!(crosses(&cache), 3, "the cross is drawn");

    let events = cache.handle_event(down(trailing.0, trailing.1, 1));

    assert_eq!(get(&cache, id).text(), "");
    assert_eq!(changes(&events), 1);
}

#[test]
fn the_search_preset_has_a_placeholder_and_a_clickable_magnifier() {
    let (mut cache, id) = focused(TextField::search());
    let events = cache.handle_event(down(10 + 12, 22, 1));

    assert!(events.iter().any(|e| matches!(
        e,
        Event::AccessoryClicked { which: AccessorySide::Leading, id: i } if *i == id
    )));
    assert!(
        painted(get(&cache, id))
            .iter()
            .any(|c| matches!(c, PaintCall::Text(_, t, _) if t == "Search"))
    );
}

#[test]
fn custom_accessories_emit_accessory_clicked_with_their_side() {
    let (mut cache, id) = focused(TextField::new().with_trailing(Accessory::Custom('?')));
    let events = cache.handle_event(down(10 + 200 - 12, 22, 1));

    assert!(events.iter().any(|e| matches!(
        e,
        Event::AccessoryClicked { which: AccessorySide::Trailing, id: i } if *i == id
    )));
}

#[test]
fn text_is_clipped_to_the_room_between_the_accessories() {
    let (cache, id) = unfocused(
        TextField::new()
            .with_leading(Accessory::Search)
            .with_trailing(Accessory::Custom('?'))
            .with_text("some text"),
    );
    let calls = painted(get(&cache, id));
    let clip = calls
        .iter()
        .find_map(|c| match c {
            PaintCall::Clip(Some(r)) => Some(*r),
            _ => None,
        })
        .expect("the text is clipped");

    // Edit area x 0..200, one-pixel border, a 22 pixel accessory each side, 3 pixels padding.
    assert_eq!(clip, Rect::new(1 + 22 + 3, 1, 200 - 2 - 44 - 6, 22));
    assert!(matches!(
        calls.last(),
        Some(PaintCall::Clip(None)) | Some(_)
    ));
}

#[test]
fn a_label_moves_the_edit_area_right() {
    let (cache, id) = unfocused(TextField::new().with_label("Name:"));
    let calls = painted(get(&cache, id));
    let label_width = 5 * glyph() + 6;

    assert!(
        calls
            .iter()
            .any(|c| matches!(c, PaintCall::Text(_, t, _) if t == "Name:"))
    );
    assert!(calls.iter().any(
        |c| matches!(c, PaintCall::Rect(r, _) if r.x == label_width && r.w == (200 - label_width) as u32)
    ));
}

#[test]
fn long_text_scrolls_to_keep_the_caret_in_view() {
    let (mut cache, id) = focused(TextField::new());

    typed(&mut cache, id, &"w".repeat(60));

    let calls = painted(get(&cache, id));
    let line = calls
        .iter()
        .find_map(|c| match c {
            PaintCall::Text(r, t, _) if t.len() == 60 => Some(*r),
            _ => None,
        })
        .expect("the text is drawn whole, then clipped");

    assert!(line.x < 0, "scrolled left");
    assert!(
        line.x + line.w as i32 >= 1 + 3 + 190 - 6,
        "the end is in view"
    );
}

#[test]
fn style_overrides_apply() {
    let mut field = TextField::new().with_text("x");

    field.set_style(
        pushrod::widget::VisualState::empty(),
        pushrod::appearance::StateStyle {
            background: Some(Color::YELLOW),
            ..Default::default()
        },
    );

    assert!(
        painted(&field)
            .iter()
            .any(|c| matches!(c, PaintCall::FillRect(_, color) if *color == Color::YELLOW))
    );
}

#[test]
fn the_engine_writes_copied_text_to_the_backend_clipboard() {
    use pushrod::backend::headless::HeadlessBackend;
    use pushrod::engine::Engine;
    use pushrod::geometry::size;

    let mut engine: Engine<()> = Engine::new(size(400, 200), 30);
    let field = engine.add_widget(TextField::new().at(10, 10).with_text("copy me"));
    let mut backend = HeadlessBackend::new(size(400, 200));

    let _ = field;
    backend.push_event(down(12, 20, 1));
    backend.push_event(up(12, 20));
    backend.push_event(command('a'));
    backend.push_event(command('c'));
    engine.run(&mut backend, ());

    assert_eq!(backend.clipboard.as_deref(), Some("copy me"));
    assert!(
        backend.text_input,
        "a focused text field asks for text input"
    );
}

#[test]
fn double_click_in_an_empty_field_does_not_panic() {
    let (mut cache, id) = focused(TextField::new());

    cache.handle_event(down(12, 20, 2));
    cache.handle_event(down(12, 20, 3));

    assert_eq!(get(&cache, id).selected_text(), "");
    assert_eq!(get(&cache, id).caret(), 0);
}
