// GlyphButton tests.  Painting goes to a RecordingPainter and time is driven by hand through
// `Engine::advance`, so no window is opened and nothing waits.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::EventMask;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::{Behavior, Widget};
use pushrod_widgets::glyph_button::{GlyphButton, REPEAT_DELAY, REPEAT_INTERVAL};
use pushrod_widgets::prelude::GlyphButton as PreludeGlyphButton;
use std::time::{Duration, Instant};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

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

/// An engine with one 20x20 glyph button at (10, 10) that counts its clicks into the state.
fn engine_with(button: GlyphButton) -> (Engine<u32>, u32) {
    let mut engine: Engine<u32> = Engine::new(size(200, 100), 60);
    let id = engine.add_widget(button);

    engine.on_click(id, |count, _ui| *count += 1);

    (engine, id)
}

fn pressed_button() -> GlyphButton {
    GlyphButton::new('v')
        .at(10, 10)
        .size(20, 20)
        .activate_on_press()
}

#[test]
fn paints_fill_then_centered_glyph_then_border() {
    let mut button = GlyphButton::new('^').size(20, 20);
    button.set_color(Color::WHITE);
    let calls = painted(&button);

    assert_eq!(calls.first(), Some(&PaintCall::Fill(Color::WHITE)));
    assert!(
        matches!(calls.get(1), Some(PaintCall::Text(bounds, ..)) if *bounds == Rect::new(0, 0, 20, 20))
    );
    assert_eq!(
        calls.last(),
        Some(&PaintCall::Rect(Rect::new(0, 0, 20, 20), Color::BLACK))
    );
}

#[test]
fn pressed_state_inverts() {
    let (mut engine, id) = engine_with(GlyphButton::new('^').at(10, 10).size(20, 20));
    let mut count = 0;

    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);

    let button = engine.widgets().get_as::<GlyphButton>(id).unwrap();

    assert!(button.is_selected());
    assert_eq!(
        painted(button).first(),
        Some(&PaintCall::Fill(Color::BLACK))
    );

    engine.dispatch(up(15, 15), &mut count);
    assert!(
        !engine
            .widgets()
            .get_as::<GlyphButton>(id)
            .unwrap()
            .is_selected()
    );
}

#[test]
fn normal_mode_clicks_on_release_only() {
    let (mut engine, _) = engine_with(GlyphButton::new('^').at(10, 10).size(20, 20));
    let mut count = 0;

    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);
    assert_eq!(count, 0);
    engine.dispatch(up(15, 15), &mut count);
    assert_eq!(count, 1);
}

#[test]
fn activate_on_press_fires_immediately_and_not_again_on_release() {
    let (mut engine, _) = engine_with(pressed_button());
    let mut count = 0;

    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);
    assert_eq!(count, 1);
    engine.dispatch(up(15, 15), &mut count);
    assert_eq!(count, 1);
}

#[test]
fn held_button_repeats_after_the_delay_then_every_interval() {
    let (mut engine, _) = engine_with(pressed_button());
    let mut count = 0;
    let start = Instant::now();

    engine.advance(start, &mut count);
    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);
    assert_eq!(count, 1);

    // The first tick only primes the clock.
    engine.advance(start + ms(16), &mut count);
    assert_eq!(count, 1);

    // Just before the delay: nothing.
    engine.advance(start + ms(16) + REPEAT_DELAY - ms(1), &mut count);
    assert_eq!(count, 1);

    // The delay passes: first repeat.
    engine.advance(start + ms(16) + REPEAT_DELAY, &mut count);
    assert_eq!(count, 2);

    // One interval later: another.
    engine.advance(start + ms(16) + REPEAT_DELAY + REPEAT_INTERVAL, &mut count);
    assert_eq!(count, 3);
    engine.advance(
        start + ms(16) + REPEAT_DELAY + REPEAT_INTERVAL * 2,
        &mut count,
    );
    assert_eq!(count, 4);
}

#[test]
fn a_stale_first_tick_does_not_cause_a_burst() {
    let (mut engine, _) = engine_with(pressed_button());
    let mut count = 0;
    let start = Instant::now();

    engine.advance(start, &mut count);
    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);

    // Ten seconds since the engine last ticked: the press must not repeat for it.
    engine.advance(start + Duration::from_secs(10), &mut count);
    assert_eq!(count, 1);
}

#[test]
fn release_stops_the_repeat() {
    let (mut engine, _) = engine_with(pressed_button());
    let mut count = 0;
    let start = Instant::now();

    engine.advance(start, &mut count);
    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);
    engine.advance(start + ms(16), &mut count);
    engine.dispatch(up(15, 15), &mut count);
    engine.advance(start + Duration::from_secs(2), &mut count);
    engine.advance(start + Duration::from_secs(4), &mut count);

    assert_eq!(count, 1);
}

#[test]
fn leaving_pauses_the_repeat_and_returning_resumes_it() {
    let (mut engine, _) = engine_with(pressed_button());
    let mut count = 0;
    let start = Instant::now();

    engine.advance(start, &mut count);
    engine.dispatch(motion(15, 15), &mut count);
    engine.dispatch(down(15, 15), &mut count);
    engine.advance(start + ms(16), &mut count);

    // Drag out and wait far longer than the delay: nothing repeats.
    engine.dispatch(motion(150, 80), &mut count);
    engine.advance(start + Duration::from_secs(3), &mut count);
    engine.advance(start + Duration::from_secs(6), &mut count);
    assert_eq!(count, 1);

    // Drag back in: the clock resumes, first tick primes, then the delay applies.
    engine.dispatch(motion(15, 15), &mut count);
    engine.advance(start + Duration::from_secs(7), &mut count);
    engine.advance(start + Duration::from_secs(7) + REPEAT_DELAY, &mut count);
    assert_eq!(count, 2);
}

#[test]
fn builders_and_setters() {
    let mut button = GlyphButton::new('x').at(5, 6).size(12, 14);

    assert_eq!(button.get_origin(), point(5, 6));
    assert_eq!(button.get_size(), size(12, 14));
    assert_eq!(button.get_glyph(), 'x');
    assert!(!button.behavior().contains(Behavior::ACTIVATE_ON_PRESS));
    assert!(button.event_mask().contains(EventMask::TIMER));

    button.set_invalidated(false);
    button.set_glyph('y');
    assert_eq!(button.get_glyph(), 'y');
    assert!(button.is_invalidated());

    button.set_invalidated(false);
    button.set_glyph('y');
    assert!(
        !button.is_invalidated(),
        "an unchanged glyph does not repaint"
    );

    let on_press = PreludeGlyphButton::new('z').activate_on_press();
    assert!(on_press.behavior().contains(Behavior::ACTIVATE_ON_PRESS));
}

/// The widths of the rows (or heights of the columns) an arrow is drawn with.
fn arrow_extent(glyph: char, horizontal: bool) -> Vec<u32> {
    let button = GlyphButton::new(glyph).size(24, 24);

    painted(&button)
        .iter()
        .filter_map(|c| match c {
            PaintCall::FillRect(r, _) => Some(if horizontal { r.w } else { r.h }),
            _ => None,
        })
        .collect()
}

#[test]
fn arrow_characters_are_drawn_as_triangles_not_text() {
    for glyph in ['←', '↑', '→', '↓'] {
        let button = GlyphButton::new(glyph).size(24, 24);
        let calls = painted(&button);

        assert!(
            !calls.iter().any(|c| matches!(c, PaintCall::Text(..))),
            "{glyph}"
        );
        assert!(
            calls.iter().any(|c| matches!(c, PaintCall::FillRect(..))),
            "{glyph}"
        );
    }

    // Up grows from a one-pixel tip; down shrinks to one.
    assert_eq!(arrow_extent('↑', true), vec![1, 3, 5, 7, 9, 11, 13]);
    assert_eq!(arrow_extent('←', false), vec![1, 3, 5, 7, 9, 11, 13]);
}

#[test]
fn arrow_points_the_right_way() {
    let first = |glyph| {
        painted(&GlyphButton::new(glyph).size(24, 24))
            .iter()
            .find_map(|c| match c {
                PaintCall::FillRect(r, _) => Some(*r),
                _ => None,
            })
            .unwrap()
    };

    // The first rectangle is the tip: above center for up, below for down, and so on.
    assert!(first('↑').y < 12 && first('↓').y >= 12);
    assert!(first('←').x < 12 && first('→').x >= 12);
}

#[test]
fn font_and_glyph_color_are_set_after_creation() {
    use pushrod::color::FontStyle;
    use pushrod::font::Font;

    let mut button = GlyphButton::new('+').size(20, 20);
    button.set_invalidated(false);

    button.set_font_color(Color::BLUE);
    assert_eq!(button.get_font_color(), Color::BLUE);
    assert!(button.is_invalidated());

    button.set_invalidated(false);
    button.set_font(Font::new("", 18, FontStyle::BOLD));
    assert_eq!(button.get_font().size(), 18);
    assert!(button.is_invalidated());
    assert!(
        painted(&button)
            .iter()
            .any(|call| matches!(call, PaintCall::Text(_, _, color) if *color == Color::BLUE))
    );
}
