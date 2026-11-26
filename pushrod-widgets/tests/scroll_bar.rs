// ScrollBar tests.  Input is dispatched to a headless engine and time is driven by hand through
// `Engine::advance`, so no window is opened and nothing waits.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::color::Color;
use pushrod::composite::{Composite, TypedParts};
use pushrod::engine::Engine;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::{VisualState, Widget, WidgetId};
use pushrod_widgets::frame::Frame;
use pushrod_widgets::glyph_button::{GlyphButton, REPEAT_DELAY, REPEAT_INTERVAL};
use pushrod_widgets::prelude::{ScrollBar as PreludeScrollBar, ScrollBarParts};
use pushrod_widgets::scroll_bar::{
    DEFAULT_PAGE_SIZE, DEFAULT_STEP, MIN_THUMB_LENGTH, ScrollBar, THUMB_COLOR, THUMB_DRAGGED_COLOR,
};
use std::time::{Duration, Instant};

const EPSILON: f64 = 1e-9;

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

/// Moves to a point, presses, and releases there.
fn click(engine: &mut Engine<u32>, count: &mut u32, x: i32, y: i32) {
    engine.dispatch(motion(x, y), count);
    engine.dispatch(down(x, y), count);
    engine.dispatch(up(x, y), count);
}

/// An engine holding `bar` at (10, 10), counting its `ValueChanged` events into the state.
fn engine_with(bar: ScrollBar) -> (Engine<u32>, WidgetId, ScrollBarParts) {
    let mut engine: Engine<u32> = Engine::new(size(300, 300), 60);
    let id = engine.add_widget(bar.at(10, 10));
    let parts = engine
        .widgets()
        .get_as::<ScrollBar>(id)
        .unwrap()
        .typed_parts();

    engine.on_value_change(id, |count, _ui| *count += 1);

    (engine, id, parts)
}

fn value_of(engine: &mut Engine<u32>, id: WidgetId) -> f64 {
    engine
        .widgets()
        .get_as::<ScrollBar>(id)
        .unwrap()
        .get_value()
}

fn rect_of(engine: &mut Engine<u32>, id: WidgetId) -> Rect {
    let widget = engine.widgets().get(id).unwrap();
    let (origin, bounds) = (widget.get_origin(), widget.get_size());

    Rect::new(origin.x, origin.y, bounds.w, bounds.h)
}

/// Draws a frame, which arranges the parts a scroll bar asked to move.
fn draw(engine: &mut Engine<u32>) {
    let mut backend = HeadlessBackend::new(size(300, 300));

    engine.widgets().draw_loop(&mut backend);
}

#[test]
fn defaults() {
    let bar = ScrollBar::vertical();

    assert!(bar.is_vertical());
    assert_eq!(bar.get_size(), size(16, 200));
    assert_eq!(bar.get_value(), 0.0);
    assert_eq!(bar.get_page_size(), DEFAULT_PAGE_SIZE);
    assert_eq!(bar.get_step(), DEFAULT_STEP);
    assert!(!bar.is_dragging());

    let bar = PreludeScrollBar::horizontal();

    assert!(!bar.is_vertical());
    assert_eq!(bar.get_size(), size(200, 16));
}

#[test]
fn parts_are_named_typed_and_laid_out() {
    let (mut engine, id, parts) = engine_with(ScrollBar::vertical());

    assert_eq!(
        engine.widgets().parts(id),
        vec![
            ("track".to_string(), parts.track),
            ("thumb".to_string(), parts.thumb),
            ("up".to_string(), parts.up),
            ("down".to_string(), parts.down),
        ]
    );
    assert_eq!(engine.widgets().part(id, "thumb"), Some(parts.thumb));

    // Arrows are square at the ends; the track shares their border lines.
    assert_eq!(rect_of(&mut engine, parts.up), Rect::new(0, 0, 16, 16));
    assert_eq!(rect_of(&mut engine, parts.down), Rect::new(0, 184, 16, 16));
    assert_eq!(rect_of(&mut engine, parts.track), Rect::new(0, 15, 16, 170));

    // The thumb sits inside the track: 166 pixels of room, a quarter of it at the top.
    assert_eq!(rect_of(&mut engine, parts.thumb), Rect::new(2, 17, 12, 42));

    let glyph = |engine: &mut Engine<u32>, part| {
        engine
            .widgets()
            .get_as::<GlyphButton>(part)
            .unwrap()
            .get_glyph()
    };

    assert_eq!(glyph(&mut engine, parts.up), '\u{2191}');
    assert_eq!(glyph(&mut engine, parts.down), '\u{2193}');
    assert!(engine.widgets().get_as::<Frame>(parts.track).is_some());
    assert_eq!(
        engine.widgets().get(parts.thumb).unwrap().get_color(),
        THUMB_COLOR
    );
}

#[test]
fn horizontal_parts_run_left_to_right() {
    let (mut engine, _, parts) = engine_with(ScrollBar::horizontal().value(1.0));

    assert_eq!(rect_of(&mut engine, parts.up), Rect::new(0, 0, 16, 16));
    assert_eq!(rect_of(&mut engine, parts.down), Rect::new(184, 0, 16, 16));
    assert_eq!(rect_of(&mut engine, parts.track), Rect::new(15, 0, 170, 16));
    // At the end: 17 + (166 - 42).
    assert_eq!(rect_of(&mut engine, parts.thumb), Rect::new(141, 2, 42, 12));
    assert_eq!(
        engine
            .widgets()
            .get_as::<GlyphButton>(parts.up)
            .unwrap()
            .get_glyph(),
        '\u{2190}'
    );
}

#[test]
fn thumb_length_reflects_the_visible_proportion() {
    let length = |page: f64| ScrollBar::vertical().page_size(page).thumb_bounds().h;

    assert_eq!(length(0.5), 83);
    assert_eq!(length(1.0), 166);
    assert_eq!(length(0.0), MIN_THUMB_LENGTH);
    assert_eq!(length(0.01), MIN_THUMB_LENGTH);

    let mut bar = ScrollBar::vertical();

    bar.set_proportion(30.0, 120.0);
    assert_eq!(bar.get_page_size(), 0.25);
    bar.set_proportion(500.0, 120.0);
    assert_eq!(bar.get_page_size(), 1.0);
    bar.set_proportion(10.0, 0.0);
    assert_eq!(bar.get_page_size(), 1.0);
}

#[test]
fn a_tiny_bar_has_no_thumb_and_does_not_panic() {
    let bar = ScrollBar::vertical().size(16, 20);

    assert_eq!(bar.thumb_bounds().h, 0);
    assert_eq!(bar.track_bounds(), Rect::new(0, 9, 16, 2));

    let bar = ScrollBar::vertical().size(0, 0);

    assert_eq!(bar.thumb_bounds(), Rect::new(2, 2, 0, 0));
}

#[test]
fn set_value_and_page_size_clamp_and_emit_nothing() {
    let (mut engine, id, parts) = engine_with(ScrollBar::vertical());
    let mut count = 0;
    let bar = engine.widgets().get_as_mut::<ScrollBar>(id).unwrap();

    bar.set_value(2.0);
    assert_eq!(bar.get_value(), 1.0);
    bar.set_value(f64::NAN);
    assert_eq!(bar.get_value(), 0.0);
    bar.set_value(0.5);
    bar.set_page_size(-1.0);
    assert_eq!(bar.get_page_size(), 0.0);
    bar.set_page_size(f64::NAN);
    assert_eq!(bar.get_page_size(), 1.0);
    bar.set_page_size(0.25);
    bar.set_step(-0.5);
    assert_eq!(bar.get_step(), 0.0);

    // The thumb follows on the next frame; no event was emitted.
    assert!(engine.widgets().needs_draw());
    draw(&mut engine);
    engine.advance(Instant::now(), &mut count);

    assert_eq!(rect_of(&mut engine, parts.thumb), Rect::new(2, 79, 12, 42));
    assert_eq!(count, 0);
}

#[test]
fn an_arrow_steps_at_once_and_repeats_while_held() {
    let (mut engine, id, parts) = engine_with(ScrollBar::vertical());
    let mut count = 0;
    let start = Instant::now();
    let ms = Duration::from_millis;

    engine.advance(start, &mut count);
    engine.dispatch(motion(18, 200), &mut count);
    engine.dispatch(down(18, 200), &mut count);

    assert_eq!(count, 1);
    assert!((value_of(&mut engine, id) - DEFAULT_STEP).abs() < EPSILON);

    engine.advance(start + ms(16), &mut count);
    engine.advance(start + ms(16) + REPEAT_DELAY, &mut count);
    engine.advance(start + ms(16) + REPEAT_DELAY + REPEAT_INTERVAL, &mut count);

    assert_eq!(count, 3);
    assert!((value_of(&mut engine, id) - DEFAULT_STEP * 3.0).abs() < EPSILON);

    // Released: the repeat stops.
    engine.dispatch(up(18, 200), &mut count);
    engine.advance(start + Duration::from_secs(5), &mut count);
    assert_eq!(count, 3);

    // The thumb moved down: 17 + round(0.15 * 124).
    draw(&mut engine);
    assert_eq!(rect_of(&mut engine, parts.thumb).y, 17 + 19);

    // The up arrow goes back.
    click(&mut engine, &mut count, 18, 15);
    assert_eq!(count, 4);
    assert!((value_of(&mut engine, id) - DEFAULT_STEP * 2.0).abs() < EPSILON);
}

#[test]
fn arrows_stop_at_the_ends_without_an_event() {
    let (mut engine, id, _) = engine_with(ScrollBar::vertical());
    let mut count = 0;

    click(&mut engine, &mut count, 18, 15);
    assert_eq!(count, 0);
    assert_eq!(value_of(&mut engine, id), 0.0);

    engine
        .widgets()
        .get_as_mut::<ScrollBar>(id)
        .unwrap()
        .set_value(0.99);
    click(&mut engine, &mut count, 18, 200);
    click(&mut engine, &mut count, 18, 200);

    assert_eq!(count, 1);
    assert_eq!(value_of(&mut engine, id), 1.0);
}

#[test]
fn pressing_the_track_pages_toward_the_pointer() {
    let (mut engine, id, parts) = engine_with(ScrollBar::vertical());
    let mut count = 0;
    let bar = engine.widgets().get_as::<ScrollBar>(id).unwrap();

    // A quarter visible: one page moves a third of the range.
    assert!((bar.page_step() - 1.0 / 3.0).abs() < EPSILON);

    // Below the thumb (which ends at y = 10 + 17 + 42).
    click(&mut engine, &mut count, 18, 150);
    assert_eq!(count, 1);
    assert!((value_of(&mut engine, id) - 1.0 / 3.0).abs() < EPSILON);

    draw(&mut engine);
    assert_eq!(rect_of(&mut engine, parts.thumb).y, 17 + 41);

    // Above the thumb.
    click(&mut engine, &mut count, 18, 30);
    assert_eq!(count, 2);
    assert_eq!(value_of(&mut engine, id), 0.0);
}

#[test]
fn page_step_is_never_less_than_the_arrow_step() {
    assert_eq!(ScrollBar::vertical().page_size(1.0).page_step(), 1.0);
    assert_eq!(
        ScrollBar::vertical().page_size(0.0).step(0.2).page_step(),
        0.2
    );
}

#[test]
fn dragging_the_thumb_follows_the_pointer_even_outside_the_bar() {
    let (mut engine, id, parts) = engine_with(ScrollBar::vertical());
    let mut count = 0;

    // Grab the thumb 5 pixels below its top (window y = 10 + 17 + 5).
    engine.dispatch(motion(18, 32), &mut count);
    engine.dispatch(down(18, 32), &mut count);

    assert!(
        engine
            .widgets()
            .get_as::<ScrollBar>(id)
            .unwrap()
            .is_dragging()
    );
    assert_eq!(count, 0);

    // The pressed thumb is drawn dark.
    let thumb = engine.widgets().get(parts.thumb).unwrap();
    assert!(thumb.state().contains(VisualState::PRESSED));
    assert_eq!(thumb.style().background, Some(THUMB_DRAGGED_COLOR));

    // Half the travel (124 / 2), with the pointer off to the side.
    engine.dispatch(motion(200, 32 + 62), &mut count);
    assert_eq!(count, 1);
    assert!((value_of(&mut engine, id) - 0.5).abs() < EPSILON);

    draw(&mut engine);
    assert_eq!(rect_of(&mut engine, parts.thumb).y, 17 + 62);

    // Far past the end: clamped, then one more move at the end emits nothing.
    engine.dispatch(motion(200, 900), &mut count);
    engine.dispatch(motion(200, 950), &mut count);
    assert_eq!(count, 2);
    assert_eq!(value_of(&mut engine, id), 1.0);

    engine.dispatch(up(200, 950), &mut count);
    assert!(
        !engine
            .widgets()
            .get_as::<ScrollBar>(id)
            .unwrap()
            .is_dragging()
    );
    assert_eq!(
        engine
            .widgets()
            .get(parts.thumb)
            .unwrap()
            .style()
            .background,
        None
    );

    // After the release, moving does nothing.
    engine.dispatch(motion(18, 32), &mut count);
    assert_eq!(count, 2);
}

#[test]
fn a_disabled_scroll_bar_ignores_its_arrows() {
    let (mut engine, id, _) = engine_with(ScrollBar::vertical());
    let mut count = 0;

    engine.widgets().get_mut(id).unwrap().set_enabled(false);
    click(&mut engine, &mut count, 18, 200);
    click(&mut engine, &mut count, 18, 150);

    assert_eq!(count, 0);
    assert_eq!(value_of(&mut engine, id), 0.0);
}

#[test]
fn parts_are_restyled_through_the_normal_api() {
    let (mut engine, id, parts) = engine_with(ScrollBar::vertical());
    let mut backend = HeadlessBackend::new(size(300, 300));

    engine.widgets().draw_loop(&mut backend);
    backend.painted.clear();

    let arrow = engine
        .widgets()
        .get_as_mut::<GlyphButton>(parts.down)
        .unwrap();
    arrow.set_glyph('+');
    arrow.set_font_color(Color::BLUE);
    arrow.set_color(Color::YELLOW);

    let thumb = engine.widgets().get_as_mut::<Frame>(parts.thumb).unwrap();
    thumb.set_color(Color::GREEN);
    thumb.set_border_color(Color::RED);

    engine.widgets().draw_loop(&mut backend);

    let arrow = engine.widgets().get_as::<GlyphButton>(parts.down).unwrap();
    assert_eq!(arrow.get_glyph(), '+');
    assert_eq!(arrow.get_font_color(), Color::BLUE);
    assert!(
        backend
            .painted
            .iter()
            .any(|(painted, _)| *painted == parts.down)
    );
    assert!(
        backend
            .painted
            .iter()
            .any(|(painted, _)| *painted == parts.thumb)
    );

    // A restyled arrow still scrolls.
    let mut count = 0;
    click(&mut engine, &mut count, 18, 200);
    assert_eq!(count, 1);
    assert!(
        engine
            .widgets()
            .get_as::<ScrollBar>(id)
            .unwrap()
            .part("down")
            .is_some()
    );
}
