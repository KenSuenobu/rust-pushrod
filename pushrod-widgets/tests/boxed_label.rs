// BoxedLabel tests.  The widget paints into a RecordingPainter, so no window is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::event::EventMask;
use pushrod::geometry::{Rect, point};
use pushrod::widget::Widget;
use pushrod_widgets::boxed_label::BoxedLabel;
use pushrod_widgets::prelude::BoxedLabel as PreludeBoxedLabel;

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

#[test]
fn paints_fill_then_text_then_border() {
    let mut label = BoxedLabel::new("Ready").size(80, 20);
    label.set_color(Color::WHITE);
    let calls = painted(&label);

    assert_eq!(calls.first(), Some(&PaintCall::Fill(Color::WHITE)));
    assert!(matches!(calls.get(1), Some(PaintCall::Text(..))));
    assert_eq!(
        calls.last(),
        Some(&PaintCall::Rect(Rect::new(0, 0, 80, 20), Color::BLACK))
    );
}

#[test]
fn text_is_centered_inside_the_border() {
    let label = BoxedLabel::new("Ready").size(80, 20).border_width(3);

    let calls = painted(&label);
    let text = calls.iter().find_map(|c| match c {
        PaintCall::Text(bounds, ..) => Some(*bounds),
        _ => None,
    });

    assert_eq!(text, Some(Rect::new(3, 3, 74, 14)));
}

#[test]
fn zero_border_draws_no_border() {
    let label = BoxedLabel::new("Ready").border_width(0);

    assert!(
        !painted(&label)
            .iter()
            .any(|c| matches!(c, PaintCall::Rect(..)))
    );
}

#[test]
fn is_display_only() {
    let label = BoxedLabel::new("Ready");

    assert_eq!(label.event_mask(), EventMask::NONE);
}

#[test]
fn setters_update_and_invalidate() {
    let mut label = BoxedLabel::new("a").at(5, 6).border_color(Color::BLUE);

    assert_eq!(label.get_origin(), point(5, 6));
    label.set_invalidated(false);
    label.set_text("b");
    assert_eq!(label.get_text(), "b");
    assert!(label.is_invalidated());

    label.set_invalidated(false);
    label.set_border_color(Color::RED);
    assert!(label.is_invalidated());
    label.set_invalidated(false);
    label.set_font_color(Color::RED);
    assert!(label.is_invalidated());
}

#[test]
fn tiny_widgets_do_not_underflow() {
    let label = BoxedLabel::new("x").size(1, 1).border_width(4);

    painted(&label);
}

#[test]
fn is_in_the_prelude() {
    let _ = PreludeBoxedLabel::new("ok");
}
