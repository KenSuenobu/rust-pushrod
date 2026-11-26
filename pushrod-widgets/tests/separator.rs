// Separator tests.  The widget paints into a RecordingPainter, so no window is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::event::EventMask;
use pushrod::geometry::{point, size};
use pushrod::widget::Widget;
use pushrod_widgets::prelude::Separator as PreludeSeparator;
use pushrod_widgets::separator::Separator;

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

#[test]
fn horizontal_is_one_pixel_high() {
    let rule = Separator::horizontal(100);

    assert_eq!(rule.get_size(), size(100, 1));
    assert!(rule.is_horizontal());
    assert_eq!(
        painted(&rule),
        vec![PaintCall::Line(
            point(0, 0),
            point(99, 0),
            Color::RGBA(64, 64, 64, 255)
        )]
    );
}

#[test]
fn vertical_is_one_pixel_wide() {
    let rule = Separator::vertical(40).line_color(Color::BLUE);

    assert_eq!(rule.get_size(), size(1, 40));
    assert!(!rule.is_horizontal());
    assert_eq!(
        painted(&rule),
        vec![PaintCall::Line(point(0, 0), point(0, 39), Color::BLUE)]
    );
}

#[test]
fn etched_horizontal_draws_two_lines() {
    let rule = Separator::horizontal(50).etched();

    assert_eq!(rule.get_size(), size(50, 2));
    assert!(rule.is_etched());
    let calls = painted(&rule);
    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[1],
        PaintCall::Line(point(0, 1), point(49, 1), Color::WHITE)
    );
}

#[test]
fn etched_vertical_draws_two_lines() {
    let rule = Separator::vertical(30).etched();

    assert_eq!(rule.get_size(), size(2, 30));
    assert_eq!(
        painted(&rule)[1],
        PaintCall::Line(point(1, 0), point(1, 29), Color::WHITE)
    );
}

#[test]
fn set_etched_toggles_thickness_and_invalidates() {
    let mut rule = Separator::horizontal(20).etched();
    rule.set_invalidated(false);

    rule.set_etched(false);

    assert_eq!(rule.get_size(), size(20, 1));
    assert!(rule.is_invalidated());
    assert_eq!(painted(&rule).len(), 1);
}

#[test]
fn zero_length_draws_nothing() {
    assert!(painted(&Separator::horizontal(0)).is_empty());
    assert!(painted(&Separator::vertical(0)).is_empty());
}

#[test]
fn set_line_color_invalidates() {
    let mut rule = Separator::horizontal(20);
    rule.set_invalidated(false);

    rule.set_line_color(Color::BLACK);

    assert!(rule.is_invalidated());
}

#[test]
fn is_display_only() {
    assert_eq!(Separator::horizontal(10).event_mask(), EventMask::NONE);
    let _ = PreludeSeparator::vertical(10).at(1, 2);
}
