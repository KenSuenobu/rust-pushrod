// ProgressBar tests: painting goes to a RecordingPainter and time is driven by hand through
// `Engine::advance`, so no window is opened and nothing waits.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::{Event, EventMask};
use pushrod::geometry::{Rect, size};
use pushrod::widget::Widget;
use pushrod_widgets::prelude::ProgressBar as PreludeProgressBar;
use pushrod_widgets::progress_bar::{ProgressBar, SWEEP};
use std::time::{Duration, Instant};

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

/// The black rectangles filled inside the frame.
fn fills(widget: &dyn Widget) -> Vec<Rect> {
    painted(widget)
        .into_iter()
        .filter_map(|call| match call {
            PaintCall::FillRect(r, c) if c == Color::BLACK => Some(r),
            _ => None,
        })
        .collect()
}

fn bar(max: f64, value: f64) -> ProgressBar {
    ProgressBar::new(max).size(102, 16).value(value)
}

fn engine_with(bar: ProgressBar) -> (Engine<()>, u32) {
    let mut engine: Engine<()> = Engine::new(size(300, 100), 60);
    let id = engine.add_widget(bar);

    (engine, id)
}

fn get(engine: &mut Engine<()>, id: u32) -> &ProgressBar {
    engine.widgets().get_as::<ProgressBar>(id).unwrap()
}

#[test]
fn defaults() {
    let bar = ProgressBar::new(10.0);

    assert_eq!(bar.max(), 10.0);
    assert_eq!(bar.get_value(), Some(0.0));
    assert!(!bar.is_indeterminate());
    assert!(!bar.is_outline_mode());
    assert_eq!(bar.get_size(), size(200, 16));
    assert_eq!(bar.event_mask(), EventMask::TIMER);
    assert!(ProgressBar::indeterminate().is_indeterminate());
    let _: Option<PreludeProgressBar> = None;
}

#[test]
fn fraction_is_value_over_max_and_clamped() {
    assert_eq!(bar(100.0, 40.0).fraction(), Some(0.4));
    assert_eq!(bar(100.0, 0.0).fraction(), Some(0.0));
    assert_eq!(bar(100.0, 100.0).fraction(), Some(1.0));
    assert_eq!(bar(100.0, 250.0).fraction(), Some(1.0));
    assert_eq!(bar(100.0, -5.0).fraction(), Some(0.0));
    assert_eq!(bar(100.0, f64::NAN).fraction(), Some(0.0));
    assert_eq!(bar(0.0, 5.0).fraction(), Some(0.0));
    assert_eq!(bar(-1.0, 5.0).fraction(), Some(0.0));
    assert_eq!(bar(f64::INFINITY, 5.0).fraction(), Some(0.0));
    assert_eq!(ProgressBar::indeterminate().fraction(), None);
}

#[test]
fn determinate_fill_is_proportional() {
    // 102 wide: a 100 pixel track inside the frame.
    assert_eq!(fills(&bar(100.0, 40.0)), vec![Rect::new(1, 1, 40, 14)]);
    assert_eq!(fills(&bar(200.0, 100.0)), vec![Rect::new(1, 1, 50, 14)]);
    assert_eq!(fills(&bar(100.0, 100.0)), vec![Rect::new(1, 1, 100, 14)]);
    assert!(fills(&bar(100.0, 0.0)).is_empty());
}

#[test]
fn paints_a_white_track_and_a_frame() {
    let calls = painted(&bar(100.0, 0.0));

    assert_eq!(calls[0], PaintCall::Fill(Color::WHITE));
    assert!(calls.contains(&PaintCall::Rect(Rect::new(0, 0, 102, 16), Color::BLACK)));
}

#[test]
fn a_tiny_bar_does_not_panic() {
    for (w, h) in [(0, 0), (1, 1), (2, 2), (3, 3), (50, 2)] {
        let bar = ProgressBar::indeterminate().size(w, h);

        painted(&bar);
        painted(&bar.value(1.0));
    }
}

#[test]
fn setting_the_value_invalidates_and_emits_nothing() {
    let (mut engine, id) = engine_with(bar(100.0, 0.0));

    engine.widgets().get_mut(id).unwrap().set_invalidated(false);
    engine
        .widgets()
        .get_as_mut::<ProgressBar>(id)
        .unwrap()
        .set_value(Some(75.0));
    assert!(get(&mut engine, id).is_invalidated());
    assert_eq!(get(&mut engine, id).fraction(), Some(0.75));
    // The bar emits no events of its own (any queued event is an engine event).
    assert!(
        engine
            .widgets()
            .take_pending_events()
            .iter()
            .all(|e| matches!(e, Event::Engine(_)))
    );
}

#[test]
fn set_max_rescales_the_fill() {
    let mut bar = bar(100.0, 50.0);

    bar.set_max(200.0);
    assert_eq!(bar.max(), 200.0);
    assert_eq!(fills(&bar), vec![Rect::new(1, 1, 25, 14)]);
}

#[test]
fn indeterminate_draws_a_block_that_sweeps_and_returns() {
    let mut bar = ProgressBar::indeterminate().size(102, 16);

    // At the start the block sits at the left, a quarter of the track wide.
    assert_eq!(fills(&bar), vec![Rect::new(1, 1, 25, 14)]);
    assert_eq!(bar.sweep_position(), 0.0);

    bar.set_value(None);
    assert_eq!(bar.sweep_position(), 0.0);
}

#[test]
fn ticks_move_the_block_back_and_forth_and_keep_animating() {
    let (mut engine, id) = engine_with(ProgressBar::indeterminate().size(102, 16));
    let start = Instant::now();

    engine.widgets().request_animation(id);
    engine.advance(start, &mut ());
    engine.widgets().get_mut(id).unwrap().set_invalidated(false);

    // Half a sweep: the block is half way along.
    engine.advance(start + SWEEP / 2, &mut ());
    assert!((get(&mut engine, id).sweep_position() - 0.5).abs() < 1e-3);
    assert!(get(&mut engine, id).is_invalidated());
    assert!(engine.widgets().animation_requested());

    // A full sweep: at the right end.
    engine.advance(start + SWEEP, &mut ());
    assert!((get(&mut engine, id).sweep_position() - 1.0).abs() < 1e-3);
    assert_eq!(fills(get(&mut engine, id)), vec![Rect::new(76, 1, 25, 14)]);

    // On the way back.
    engine.advance(start + SWEEP + SWEEP / 2, &mut ());
    assert!((get(&mut engine, id).sweep_position() - 0.5).abs() < 1e-3);

    // After a full trip it wraps to the start; a very long gap wraps too.
    engine.advance(start + SWEEP * 2, &mut ());
    assert!(get(&mut engine, id).sweep_position().abs() < 1e-3);
    engine.advance(start + SWEEP * 2 + SWEEP * 20 + SWEEP / 4, &mut ());
    assert!((get(&mut engine, id).sweep_position() - 0.25).abs() < 1e-3);
}

#[test]
fn the_animation_emits_no_widget_events_or_callbacks() {
    let (mut engine, id) = engine_with(ProgressBar::indeterminate());
    let mut calls = 0;
    let start = Instant::now();

    engine.on_toggle(id, |_, _, _| {});
    engine.on_click(id, |_, _| {});
    engine.widgets().request_animation(id);

    for step in 0..10 {
        engine.advance(start + Duration::from_millis(100 * step), &mut ());
        calls += engine.widgets().take_pending_events().len();
    }

    // Only engine events (the animation ticks) are queued; the bar emits none of its own.
    assert!(engine.widgets().animation_requested());
    assert!(calls <= 10);
}

#[test]
fn a_determinate_bar_stops_asking_for_ticks() {
    let (mut engine, id) = engine_with(bar(100.0, 10.0));
    let start = Instant::now();

    engine.widgets().request_animation(id);
    engine.advance(start, &mut ());
    engine.advance(start + Duration::from_millis(50), &mut ());
    assert!(!engine.widgets().animation_requested());
    assert_eq!(get(&mut engine, id).sweep_position(), 0.0);
}

#[test]
fn going_determinate_while_busy_stops_the_animation() {
    let (mut engine, id) = engine_with(ProgressBar::indeterminate());
    let start = Instant::now();

    engine.widgets().request_animation(id);
    engine.advance(start, &mut ());
    engine.advance(start + Duration::from_millis(100), &mut ());
    assert!(engine.widgets().animation_requested());

    engine
        .widgets()
        .get_as_mut::<ProgressBar>(id)
        .unwrap()
        .set_value(Some(30.0));
    engine.advance(start + Duration::from_millis(200), &mut ());
    engine.advance(start + Duration::from_millis(300), &mut ());
    assert!(!engine.widgets().animation_requested());
}

#[test]
fn outline_mode_is_static_stripes_and_requests_no_ticks() {
    let (mut engine, id) = engine_with(ProgressBar::indeterminate().size(102, 16));
    let start = Instant::now();

    engine
        .widgets()
        .get_as_mut::<ProgressBar>(id)
        .unwrap()
        .set_outline_mode(true);
    assert!(get(&mut engine, id).is_outline_mode());
    assert!(!get(&mut engine, id).is_animating());

    let before = painted(get(&mut engine, id));
    let stripes = fills(get(&mut engine, id));

    // Stripes 4 pixels wide, every 8 pixels, across the 100 pixel track.
    assert_eq!(stripes.len(), 13);
    assert_eq!(stripes[0], Rect::new(1, 1, 4, 14));
    assert_eq!(stripes[1], Rect::new(9, 1, 4, 14));
    assert_eq!(stripes[12], Rect::new(97, 1, 4, 14));

    engine.widgets().request_animation(id);
    engine.advance(start, &mut ());
    engine.advance(start + SWEEP, &mut ());
    assert!(!engine.widgets().animation_requested());
    assert_eq!(painted(get(&mut engine, id)), before);

    // Back to live drawing: it animates again once a tick is requested.
    engine
        .widgets()
        .get_as_mut::<ProgressBar>(id)
        .unwrap()
        .set_outline_mode(false);
    assert!(get(&mut engine, id).is_animating());
}

#[test]
fn outline_mode_does_not_change_a_determinate_bar() {
    let mut bar = bar(100.0, 40.0);
    let live = painted(&bar);

    bar.set_outline_mode(true);
    assert_eq!(painted(&bar), live);
}

#[test]
fn style_colors_override_the_frame_and_fill() {
    use pushrod::appearance::StateStyle;
    use pushrod::widget::VisualState;

    let mut bar = bar(100.0, 50.0);
    let style = StateStyle {
        border_color: Some(Color::RGBA(1, 2, 3, 255)),
        foreground: Some(Color::RGBA(9, 8, 7, 255)),
        ..Default::default()
    };
    bar.set_style(VisualState::empty(), style);

    let calls = painted(&bar);

    assert!(calls.contains(&PaintCall::Rect(
        Rect::new(0, 0, 102, 16),
        Color::RGBA(1, 2, 3, 255)
    )));
    assert!(calls.contains(&PaintCall::FillRect(
        Rect::new(1, 1, 50, 14),
        Color::RGBA(9, 8, 7, 255)
    )));
}
