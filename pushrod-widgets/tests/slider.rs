// Slider tests: painting goes to a RecordingPainter and input through the WidgetCache and the
// Engine, so no window is opened.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::Event;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::{Behavior, Widget};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::prelude::Slider as PreludeSlider;
use pushrod_widgets::slider::{KNOB_LENGTH, KNOB_THICKNESS, Slider};

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

fn down_with(x: i32, y: i32, button: MouseButton) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button,
        clicks: 1,
    }
}

fn down(x: i32, y: i32) -> InputEvent {
    down_with(x, y, MouseButton::Left)
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

/// A cache with the slider at window position (10, 10) under a root panel.  A default slider is
/// 200 by 24, so its track runs from local x = 5 for 190 pixels, centered at local y = 12.
fn with_slider(slider: Slider) -> (WidgetCache, u32) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 200)));

    let id = cache.add_child(0, Box::new(slider.at(10, 10)));

    (cache, id)
}

fn get(cache: &WidgetCache, id: u32) -> &Slider {
    cache.get_as::<Slider>(id).unwrap()
}

fn changes(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::ValueChanged { .. }))
        .count()
}

fn finishes(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::EditingFinished { .. }))
        .count()
}

/// A 0..190 slider, so a local x coordinate `n` (5..=195) is the value `n - 5`.
fn unit_slider() -> Slider {
    Slider::new(0.0, 190.0)
}

#[test]
fn defaults() {
    let slider = Slider::new(0.0, 100.0);

    assert_eq!(slider.range(), (0.0, 100.0));
    assert_eq!(slider.get_value(), 0.0);
    assert_eq!(slider.get_step(), 0.0);
    assert!(!slider.shows_value());
    assert!(!slider.is_vertical());
    assert!(!slider.is_pressed());
    assert_eq!(slider.get_size(), size(200, 24));
    assert!(slider.behavior().contains(Behavior::FOCUSABLE));
    assert!(Slider::new(0.0, 1.0).vertical().is_vertical());
    let _: Option<PreludeSlider> = None;
}

#[test]
fn bad_bounds_are_repaired() {
    assert_eq!(Slider::new(10.0, 0.0).range(), (0.0, 10.0));
    assert_eq!(Slider::new(f64::NAN, f64::INFINITY).range(), (0.0, 100.0));
    assert_eq!(Slider::new(5.0, 5.0).range(), (5.0, 5.0));
}

#[test]
fn values_are_clamped_and_snapped() {
    let slider = Slider::new(0.0, 100.0).step(10.0);

    assert_eq!(slider.snap(44.0), 40.0);
    assert_eq!(slider.snap(46.0), 50.0);
    assert_eq!(slider.snap(-5.0), 0.0);
    assert_eq!(slider.snap(500.0), 100.0);
    assert_eq!(slider.snap(f64::NAN), 0.0);
    assert_eq!(Slider::new(0.0, 100.0).snap(33.3), 33.3);
    // Steps count from the minimum, and the maximum is never exceeded.
    assert_eq!(Slider::new(5.0, 28.0).step(10.0).snap(28.0), 25.0);
    // A bad step means no snapping.
    assert_eq!(Slider::new(0.0, 100.0).step(-1.0).snap(33.3), 33.3);
    assert_eq!(Slider::new(0.0, 100.0).step(f64::NAN).get_step(), 0.0);
    assert_eq!(Slider::new(0.0, 100.0).value(250.0).get_value(), 100.0);
    assert_eq!(
        Slider::new(0.0, 100.0).step(5.0).value(42.0).get_value(),
        40.0
    );
}

#[test]
fn set_value_range_and_step_adjust_the_value_and_emit_nothing() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 100.0).value(80.0));
    let slider = cache.get_as_mut::<Slider>(id).unwrap();

    slider.set_invalidated(false);
    slider.set_value(30.0);
    assert_eq!(slider.get_value(), 30.0);
    assert!(slider.is_invalidated());
    slider.set_range(0.0, 20.0);
    assert_eq!(slider.get_value(), 20.0);
    // 20 is 2.5 steps of 8; it rounds up to 24, which is clamped back to the maximum.
    slider.set_step(8.0);
    assert_eq!(slider.get_value(), 20.0);
    assert_eq!(slider.get_step(), 8.0);
    assert!(
        cache
            .take_pending_events()
            .iter()
            .all(|e| matches!(e, Event::Engine(_)))
    );
}

#[test]
fn positions_and_values_convert_both_ways() {
    let slider = Slider::new(0.0, 190.0);

    assert_eq!(slider.position_of(0.0), 5);
    assert_eq!(slider.position_of(190.0), 195);
    assert_eq!(slider.position_of(95.0), 100);
    assert_eq!(slider.position_of(-50.0), 5);
    assert_eq!(slider.position_of(999.0), 195);
    assert_eq!(slider.value_at(5), 0.0);
    assert_eq!(slider.value_at(100), 95.0);
    assert_eq!(slider.value_at(-30), 0.0);
    assert_eq!(slider.value_at(900), 190.0);

    // A zero range or a zero length track does not divide by zero.
    assert_eq!(Slider::new(3.0, 3.0).value_at(50), 3.0);
    assert_eq!(Slider::new(3.0, 3.0).position_of(3.0), 5);
    assert_eq!(Slider::new(0.0, 10.0).size(5, 24).value_at(50), 0.0);
}

#[test]
fn a_vertical_slider_has_its_minimum_at_the_bottom() {
    let slider = Slider::new(0.0, 100.0).vertical().size(24, 110);

    // The track runs from y = 5 for 100 pixels.
    assert_eq!(slider.position_of(0.0), 105);
    assert_eq!(slider.position_of(100.0), 5);
    assert_eq!(slider.value_at(105), 0.0);
    assert_eq!(slider.value_at(5), 100.0);
    assert_eq!(slider.value_at(55), 50.0);
    assert_eq!(
        slider.knob_bounds(),
        Rect::new(12 - 9, 105 - 5, KNOB_LENGTH, KNOB_THICKNESS)
    );
}

#[test]
fn clicking_the_track_moves_the_knob_there_and_finishes_on_release() {
    let (mut cache, id) = with_slider(unit_slider());

    // Local x 105 is window x 115.
    cache.handle_event(motion(115, 22));

    let pressed = cache.handle_event(down(115, 22));

    assert_eq!(changes(&pressed), 1);
    assert_eq!(get(&cache, id).get_value(), 100.0);
    assert!(get(&cache, id).is_pressed());

    let released = cache.handle_event(up(115, 22));

    assert_eq!(finishes(&released), 1);
    assert_eq!(changes(&released), 0);
    assert!(!get(&cache, id).is_pressed());
}

#[test]
fn clicking_where_the_knob_already_is_emits_no_value_change() {
    let (mut cache, id) = with_slider(unit_slider().value(95.0));

    cache.handle_event(motion(110, 22));

    let pressed = cache.handle_event(down(110, 22));

    assert_eq!(changes(&pressed), 0);
    assert_eq!(get(&cache, id).get_value(), 95.0);
    assert_eq!(finishes(&cache.handle_event(up(110, 22))), 1);
}

#[test]
fn dragging_changes_the_value_continuously() {
    let (mut cache, id) = with_slider(unit_slider());

    cache.handle_event(motion(15, 22));
    cache.handle_event(down(15, 22));

    let mut total = 0;

    for x in [30, 60, 90, 120] {
        let moved = cache.handle_event(motion(x, 22));

        assert_eq!(changes(&moved), 1);
        assert_eq!(finishes(&moved), 0);
        assert_eq!(get(&cache, id).get_value(), (x - 10 - 5) as f64);
        total += 1;
    }

    assert_eq!(total, 4);
    assert_eq!(finishes(&cache.handle_event(up(120, 22))), 1);

    // After the release, motion does nothing.
    assert_eq!(changes(&cache.handle_event(motion(40, 22))), 0);
    assert_eq!(get(&cache, id).get_value(), 105.0);
}

#[test]
fn dragging_outside_the_slider_clamps_to_the_ends() {
    let (mut cache, id) = with_slider(unit_slider());

    cache.handle_event(motion(15, 22));
    cache.handle_event(down(15, 22));
    cache.handle_event(motion(290, 190));
    assert_eq!(get(&cache, id).get_value(), 190.0);
    cache.handle_event(motion(0, 0));
    assert_eq!(get(&cache, id).get_value(), 0.0);
    // Releasing far away still finishes the edit.
    assert_eq!(finishes(&cache.handle_event(up(290, 190))), 1);
}

#[test]
fn grabbing_the_knob_does_not_make_it_jump() {
    let (mut cache, id) = with_slider(unit_slider().value(95.0));

    // The knob is centered at local x 100; press 3 pixels to the right of that.
    cache.handle_event(motion(113, 22));

    let pressed = cache.handle_event(down(113, 22));

    assert_eq!(changes(&pressed), 0);
    assert_eq!(get(&cache, id).get_value(), 95.0);

    // Moving 10 pixels moves the value by 10.
    cache.handle_event(motion(123, 22));
    assert_eq!(get(&cache, id).get_value(), 105.0);
    cache.handle_event(up(123, 22));
}

#[test]
fn only_the_left_button_drags() {
    let (mut cache, id) = with_slider(unit_slider());

    cache.handle_event(motion(115, 22));

    let events = cache.handle_event(down_with(115, 22, MouseButton::Right));

    assert_eq!(changes(&events), 0);
    assert_eq!(get(&cache, id).get_value(), 0.0);
    assert!(!get(&cache, id).is_pressed());
}

#[test]
fn snapping_applies_while_dragging() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 190.0).step(10.0));

    cache.handle_event(motion(15, 22));
    cache.handle_event(down(15, 22));

    // Local x 62 is the value 57, which snaps to 60.
    let moved = cache.handle_event(motion(72, 22));

    assert_eq!(changes(&moved), 1);
    assert_eq!(get(&cache, id).get_value(), 60.0);

    // Moving within the same snap position emits nothing.
    assert_eq!(changes(&cache.handle_event(motion(73, 22))), 0);
    cache.handle_event(up(73, 22));
}

#[test]
fn a_vertical_slider_follows_the_vertical_pointer() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 100.0).vertical().size(24, 110));

    // Local y 55 is window y 65: the middle of the track.
    cache.handle_event(motion(22, 65));
    cache.handle_event(down(22, 65));
    assert_eq!(get(&cache, id).get_value(), 50.0);
    cache.handle_event(motion(22, 15));
    assert_eq!(get(&cache, id).get_value(), 100.0);
    cache.handle_event(motion(22, 115));
    assert_eq!(get(&cache, id).get_value(), 0.0);
    cache.handle_event(up(22, 115));
}

#[test]
fn arrow_keys_move_by_the_step_when_focused() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 100.0).step(5.0).value(50.0));

    cache.set_focus(id);

    assert_eq!(changes(&cache.handle_event(key(Key::Right))), 1);
    assert_eq!(get(&cache, id).get_value(), 55.0);
    cache.handle_event(key(Key::Up));
    assert_eq!(get(&cache, id).get_value(), 60.0);
    cache.handle_event(key(Key::Left));
    cache.handle_event(key(Key::Down));
    assert_eq!(get(&cache, id).get_value(), 50.0);
}

#[test]
fn without_a_step_a_key_moves_a_hundredth_of_the_range() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 50.0));

    cache.set_focus(id);
    assert_eq!(get(&cache, id).key_step(), 0.5);
    cache.handle_event(key(Key::Right));
    assert_eq!(get(&cache, id).get_value(), 0.5);
}

#[test]
fn page_home_and_end_keys() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 100.0).step(2.0).value(50.0));

    cache.set_focus(id);
    cache.handle_event(key(Key::PageUp));
    assert_eq!(get(&cache, id).get_value(), 70.0);
    cache.handle_event(key(Key::PageDown));
    cache.handle_event(key(Key::PageDown));
    assert_eq!(get(&cache, id).get_value(), 30.0);
    cache.handle_event(key(Key::End));
    assert_eq!(get(&cache, id).get_value(), 100.0);
    cache.handle_event(key(Key::Home));
    assert_eq!(get(&cache, id).get_value(), 0.0);
}

#[test]
fn keys_stop_at_the_ends_without_emitting() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 10.0).step(1.0));

    cache.set_focus(id);
    assert_eq!(changes(&cache.handle_event(key(Key::Left))), 0);
    assert_eq!(get(&cache, id).get_value(), 0.0);
    cache.handle_event(key(Key::End));
    assert_eq!(changes(&cache.handle_event(key(Key::Right))), 0);
    assert_eq!(get(&cache, id).get_value(), 10.0);
}

#[test]
fn keys_do_nothing_without_focus_and_other_keys_are_ignored() {
    let (mut cache, id) = with_slider(Slider::new(0.0, 100.0).step(5.0));

    assert_eq!(changes(&cache.handle_event(key(Key::Right))), 0);
    assert_eq!(get(&cache, id).get_value(), 0.0);

    cache.set_focus(id);
    assert_eq!(changes(&cache.handle_event(key(Key::Char('a')))), 0);
    assert_eq!(get(&cache, id).get_value(), 0.0);
}

#[test]
fn a_disabled_slider_ignores_the_mouse_and_keys() {
    let (mut cache, id) = with_slider(unit_slider());

    cache.get_mut(id).unwrap().set_enabled(false);
    cache.set_focus(id);
    cache.handle_event(motion(115, 22));

    assert_eq!(changes(&cache.handle_event(down(115, 22))), 0);
    cache.handle_event(up(115, 22));
    assert_eq!(changes(&cache.handle_event(key(Key::End))), 0);
    assert_eq!(get(&cache, id).get_value(), 0.0);
}

#[test]
fn clicking_takes_focus() {
    let (mut cache, id) = with_slider(unit_slider());

    cache.handle_event(motion(115, 22));
    cache.handle_event(down(115, 22));
    cache.handle_event(up(115, 22));
    assert_eq!(cache.focused(), Some(id));
}

#[test]
fn engine_callbacks_see_value_changes_and_the_end_of_the_edit() {
    let mut engine: Engine<(u32, u32)> = Engine::new(size(300, 200), 60);
    let id = engine.add_widget(unit_slider().at(10, 10));
    let mut counts = (0, 0);

    engine.on_value_change(id, |c, _| c.0 += 1);
    engine.on_editing_finished(id, |c, _| c.1 += 1);

    engine.dispatch(motion(15, 22), &mut counts);
    engine.dispatch(down(15, 22), &mut counts);
    engine.dispatch(motion(60, 22), &mut counts);
    engine.dispatch(motion(90, 22), &mut counts);
    // The press at the left end leaves the value at 0, so only the two moves change it.
    assert_eq!(counts, (2, 0));
    engine.dispatch(up(90, 22), &mut counts);
    assert_eq!(counts, (2, 1));
}

fn black_rects(widget: &dyn Widget) -> Vec<Rect> {
    painted(widget)
        .into_iter()
        .filter_map(|call| match call {
            PaintCall::FillRect(r, c) if c == Color::BLACK => Some(r),
            _ => None,
        })
        .collect()
}

#[test]
fn paints_the_track_and_a_white_knob() {
    let slider = unit_slider().value(95.0);
    let calls = painted(&slider);
    let knob = Rect::new(95, 3, KNOB_THICKNESS, KNOB_LENGTH);

    assert_eq!(slider.knob_bounds(), knob);
    assert_eq!(black_rects(&slider), vec![Rect::new(5, 11, 190, 2)]);
    assert!(calls.contains(&PaintCall::FillRect(knob, Color::WHITE)));
    assert!(calls.contains(&PaintCall::Rect(knob, Color::BLACK)));
}

#[test]
fn the_knob_is_inverted_while_pressed() {
    let (mut cache, id) = with_slider(unit_slider().value(95.0));
    let knob = get(&cache, id).knob_bounds();

    cache.handle_event(motion(110, 22));
    cache.handle_event(down(110, 22));
    assert!(painted(get(&cache, id)).contains(&PaintCall::FillRect(knob, Color::BLACK)));
    cache.handle_event(up(110, 22));
    assert!(painted(get(&cache, id)).contains(&PaintCall::FillRect(knob, Color::WHITE)));
}

#[test]
fn ticks_are_drawn_at_each_spacing() {
    let slider = Slider::new(0.0, 190.0).ticks(95.0);
    let ticks: Vec<Rect> = black_rects(&slider)
        .into_iter()
        .filter(|r| r.w == 1)
        .collect();

    assert_eq!(
        ticks,
        vec![
            Rect::new(5, 23, 1, 4),
            Rect::new(100, 23, 1, 4),
            Rect::new(195, 23, 1, 4)
        ]
    );
    assert!(
        black_rects(&Slider::new(0.0, 190.0))
            .iter()
            .all(|r| r.w != 1)
    );
    // A bad spacing draws none.
    assert!(
        black_rects(&Slider::new(0.0, 190.0).ticks(-3.0))
            .iter()
            .all(|r| r.w != 1)
    );
}

#[test]
fn the_value_text_is_shown_beside_the_track_only_when_asked() {
    let plain = Slider::new(0.0, 100.0).value(40.0);
    let shown = plain_with_text(40.0);

    assert!(
        !painted(&plain)
            .iter()
            .any(|c| matches!(c, PaintCall::Text(..)))
    );
    assert!(
        painted(&shown)
            .iter()
            .any(|c| matches!(c, PaintCall::Text(_, t, _) if t == "40"))
    );

    // Showing the value shortens the track to make room for it.
    assert!(shown.position_of(100.0) < plain.position_of(100.0));
}

fn plain_with_text(value: f64) -> Slider {
    Slider::new(0.0, 100.0).value(value).show_value(true)
}

#[test]
fn value_text_formatting() {
    assert_eq!(plain_with_text(40.0).value_text(), "40");
    assert_eq!(plain_with_text(0.5).value_text(), "0.5");
    assert_eq!(plain_with_text(12.345).value_text(), "12.35");
    assert_eq!(plain_with_text(7.0001).value_text(), "7");
    assert_eq!(Slider::new(-10.0, 10.0).value(-3.25).value_text(), "-3.25");
}

#[test]
fn a_tiny_slider_does_not_panic() {
    for (w, h) in [(0, 0), (1, 1), (5, 5), (11, 11), (40, 3)] {
        for vertical in [false, true] {
            let mut slider = Slider::new(0.0, 10.0)
                .size(w, h)
                .ticks(1.0)
                .show_value(true);

            if vertical {
                slider = slider.vertical();
            }

            painted(&slider);
            assert!((0.0..=10.0).contains(&slider.value_at(7)));
        }
    }
}

#[test]
fn style_colors_override_the_frame() {
    use pushrod::appearance::StateStyle;
    use pushrod::widget::VisualState;

    let mut slider = unit_slider();
    let ink = Color::RGBA(1, 2, 3, 255);

    slider.set_style(
        VisualState::empty(),
        StateStyle {
            border_color: Some(ink),
            ..Default::default()
        },
    );
    assert!(painted(&slider).contains(&PaintCall::Rect(slider.knob_bounds(), ink)));
}
