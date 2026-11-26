// GroupBox tests.  The widget paints into a RecordingPainter (font size 14: 7 px per character,
// line height 14), so no window is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::event::EventMask;
use pushrod::geometry::Rect;
use pushrod::widget::Widget;
use pushrod_widgets::group_box::GroupBox;
use pushrod_widgets::prelude::GroupBox as PreludeGroupBox;

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

fn edges(calls: &[PaintCall]) -> Vec<Rect> {
    calls
        .iter()
        .filter_map(|c| match c {
            PaintCall::FillRect(r, Color::BLACK) => Some(*r),
            _ => None,
        })
        .collect()
}

#[test]
fn title_interrupts_the_top_border() {
    let group = GroupBox::new("Size").size(100, 60);
    let calls = painted(&group);

    // "Size" is 4 * 7 = 28 px wide; the gap runs from 10 to 10 + 28 + 8 = 46, at y = 7.
    assert!(matches!(calls.get(1), Some(PaintCall::Text(_, t, _)) if t == "Size"));
    let top: Vec<Rect> = edges(&calls)
        .into_iter()
        .filter(|r| r.h == 1 && r.y == 7)
        .collect();
    assert!(top.contains(&Rect::new(0, 7, 10, 1)));
    assert!(top.contains(&Rect::new(46, 7, 54, 1)));
    assert!(
        !top.iter()
            .any(|r| r.x < 46 && r.x + r.w as i32 > 10 && r.w > 10)
    );
}

#[test]
fn draws_sides_and_bottom() {
    let calls = painted(&GroupBox::new("Size").size(100, 60));
    let all = edges(&calls);

    assert!(all.contains(&Rect::new(0, 7, 1, 53)));
    assert!(all.contains(&Rect::new(99, 7, 1, 53)));
    assert!(all.contains(&Rect::new(0, 59, 100, 1)));
}

#[test]
fn empty_title_draws_an_unbroken_border() {
    let calls = painted(&GroupBox::new("").size(100, 60));

    assert!(!calls.iter().any(|c| matches!(c, PaintCall::Text(..))));
    assert!(edges(&calls).contains(&Rect::new(0, 7, 100, 1)));
}

#[test]
fn long_title_is_shortened_to_fit() {
    let calls = painted(&GroupBox::new("A very long group title").size(60, 40));
    let text = calls.iter().find_map(|c| match c {
        PaintCall::Text(bounds, ..) => Some(*bounds),
        _ => None,
    });

    assert_eq!(text, Some(Rect::new(14, 0, 32, 14)));
}

#[test]
fn zero_border_draws_no_edges() {
    let calls = painted(&GroupBox::new("Size").border_width(0));

    assert!(edges(&calls).is_empty());
}

#[test]
fn tiny_sizes_do_not_panic() {
    for (w, h) in [(0, 0), (1, 1), (5, 3), (30, 10)] {
        painted(&GroupBox::new("Size").size(w, h));
    }
}

#[test]
fn set_title_invalidates() {
    let mut group = GroupBox::new("Size");
    group.set_invalidated(false);

    group.set_title("Color");

    assert_eq!(group.get_title(), "Color");
    assert!(group.is_invalidated());
}

#[test]
fn is_display_only_and_can_hold_children() {
    assert_eq!(GroupBox::new("Size").event_mask(), EventMask::NONE);
    let _ = PreludeGroupBox::new("Size").at(1, 2);
}
