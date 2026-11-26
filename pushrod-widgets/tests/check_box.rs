// CheckBox tests: painting goes to a RecordingPainter and input through the WidgetCache, so no
// window is opened.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::Event;
use pushrod::geometry::{Rect, point};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::{Behavior, VisualState, Widget};
use pushrod_widgets::check_box::{BOX_SIZE, CheckBox};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::prelude::CheckBox as PreludeCheckBox;

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

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

fn key(key: Key, repeat: bool) -> InputEvent {
    InputEvent::KeyDown {
        key,
        modifiers: Modifiers::empty(),
        repeat,
    }
}

/// A cache with a 150x20 check box at (10, 10) under a root panel.
fn with_box(check_box: CheckBox) -> (WidgetCache, u32) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 200)));

    let id = cache.add_child(0, Box::new(check_box.at(10, 10)));

    (cache, id)
}

fn toggles(events: &[Event]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Toggled { checked, .. } => Some(*checked),
            _ => None,
        })
        .collect()
}

/// Presses and releases at window position (x, y).
fn click_at(cache: &mut WidgetCache, x: i32, y: i32) -> Vec<Event> {
    let mut events = cache.handle_event(motion(x, y));

    events.extend(cache.handle_event(down(x, y)));
    events.extend(cache.handle_event(up(x, y)));
    events
}

fn get(cache: &WidgetCache, id: u32) -> &CheckBox {
    cache.get_as::<CheckBox>(id).unwrap()
}

#[test]
fn clicking_the_box_toggles_and_emits_toggled() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    assert_eq!(toggles(&click_at(&mut cache, 16, 20)), vec![true]);
    assert!(get(&cache, id).is_checked());
    assert!(get(&cache, id).state().contains(VisualState::CHECKED));
    assert_eq!(toggles(&click_at(&mut cache, 16, 20)), vec![false]);
    assert!(!get(&cache, id).state().contains(VisualState::CHECKED));
}

#[test]
fn clicking_the_label_toggles_too() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    assert_eq!(toggles(&click_at(&mut cache, 100, 20)), vec![true]);
    assert!(get(&cache, id).is_checked());
}

#[test]
fn releasing_outside_does_not_toggle() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    cache.handle_event(motion(16, 20));
    cache.handle_event(down(16, 20));
    cache.handle_event(motion(250, 150));
    assert!(toggles(&cache.handle_event(up(250, 150))).is_empty());
    assert!(!get(&cache, id).is_checked());
    assert!(!get(&cache, id).is_pressed());
}

#[test]
fn space_toggles_when_focused_but_not_when_repeating() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    cache.set_focus(id);
    assert_eq!(
        toggles(&cache.handle_event(key(Key::Char(' '), false))),
        vec![true]
    );
    assert!(toggles(&cache.handle_event(key(Key::Char(' '), true))).is_empty());
    assert!(get(&cache, id).is_checked());
    assert!(toggles(&cache.handle_event(key(Key::Char('a'), false))).is_empty());
}

#[test]
fn space_does_nothing_without_focus() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    assert!(toggles(&cache.handle_event(key(Key::Char(' '), false))).is_empty());
    assert!(!get(&cache, id).is_checked());
}

#[test]
fn clicking_takes_focus() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    click_at(&mut cache, 16, 20);
    assert_eq!(cache.focused(), Some(id));
}

#[test]
fn an_indeterminate_box_becomes_checked_on_click_and_on_space() {
    let (mut cache, id) = with_box(CheckBox::new("All").indeterminate());

    assert!(get(&cache, id).is_indeterminate() && !get(&cache, id).is_checked());
    assert_eq!(toggles(&click_at(&mut cache, 16, 20)), vec![true]);
    assert!(get(&cache, id).is_checked() && !get(&cache, id).is_indeterminate());

    let (mut cache, id) = with_box(CheckBox::new("All").indeterminate());

    cache.set_focus(id);
    assert_eq!(
        toggles(&cache.handle_event(key(Key::Char(' '), false))),
        vec![true]
    );
}

#[test]
fn a_disabled_check_box_ignores_input() {
    let (mut cache, id) = with_box(CheckBox::new("Send email"));

    cache.get_mut(id).unwrap().set_enabled(false);
    assert!(toggles(&click_at(&mut cache, 16, 20)).is_empty());
    assert!(!get(&cache, id).is_checked());
}

/// The rectangles filled and the lines drawn, to tell the marks apart.
fn marks(check_box: &CheckBox) -> (Vec<Rect>, usize) {
    let calls = painted(check_box);
    let fills = calls
        .iter()
        .filter_map(|c| match c {
            PaintCall::FillRect(r, _) => Some(*r),
            _ => None,
        })
        .collect();
    let lines = calls
        .iter()
        .filter(|c| matches!(c, PaintCall::Line(..)))
        .count();

    (fills, lines)
}

#[test]
fn unchecked_draws_an_empty_box_frame_and_label() {
    let check_box = CheckBox::new("Send email");
    let calls = painted(&check_box);

    assert_eq!(calls.first(), Some(&PaintCall::Fill(check_box.get_color())));
    assert!(calls.contains(&PaintCall::Rect(
        Rect::new(1, 3, BOX_SIZE, BOX_SIZE),
        Color::BLACK
    )));
    assert!(
        calls
            .iter()
            .any(|c| matches!(c, PaintCall::Text(b, ..) if b.x == 1 + BOX_SIZE as i32 + 6))
    );

    let (fills, lines) = marks(&check_box);

    assert_eq!(
        fills,
        vec![Rect::new(1, 3, BOX_SIZE, BOX_SIZE)],
        "only the white interior"
    );
    assert_eq!(lines, 0);
}

#[test]
fn checked_draws_a_check_mark() {
    let (fills, lines) = marks(&CheckBox::new("x").checked(true));

    assert_eq!(fills.len(), 1);
    assert!(lines > 0);
}

#[test]
fn indeterminate_draws_a_dash_inside_the_box() {
    let (fills, lines) = marks(&CheckBox::new("x").indeterminate());

    assert_eq!(fills.len(), 2, "interior, then the dash");
    assert_eq!(lines, 0);
    assert_eq!(fills[1], Rect::new(4, 9, BOX_SIZE - 6, 2));
}

#[test]
fn pressed_draws_a_solid_black_box_with_no_mark() {
    let (mut cache, id) = with_box(CheckBox::new("x").checked(true));

    cache.handle_event(motion(16, 20));
    cache.handle_event(down(16, 20));

    let check_box = get(&cache, id);

    assert!(check_box.is_pressed());
    assert!(painted(check_box).contains(&PaintCall::FillRect(
        Rect::new(1, 3, BOX_SIZE, BOX_SIZE),
        Color::BLACK
    )));
    assert_eq!(marks(check_box).1, 0);
}

#[test]
fn a_narrow_check_box_draws_no_label() {
    let check_box = CheckBox::new("Send email").size(18, 20);

    assert!(
        !painted(&check_box)
            .iter()
            .any(|c| matches!(c, PaintCall::Text(..)))
    );
}

#[test]
fn builders_and_setters() {
    let mut check_box = PreludeCheckBox::new("a").at(5, 6).size(80, 22);

    assert_eq!(check_box.get_origin(), point(5, 6));
    assert!(check_box.behavior().contains(Behavior::FOCUSABLE));
    assert_eq!(check_box.get_text(), "a");

    check_box.set_invalidated(false);
    check_box.set_text("b");
    assert_eq!(check_box.get_text(), "b");
    assert!(check_box.is_invalidated());

    check_box.set_invalidated(false);
    check_box.set_checked(false);
    assert!(!check_box.is_invalidated(), "no change, no repaint");

    check_box.set_indeterminate(true);
    check_box.set_checked(true);
    assert!(check_box.is_checked() && !check_box.is_indeterminate());

    check_box.set_indeterminate(true);
    assert!(!check_box.is_checked(), "entering mixed unchecks");
}
