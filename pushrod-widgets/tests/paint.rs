// Paint tests.  Each widget paints into a RecordingPainter, so no window or display is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::{Color, FontStyle};
use pushrod::geometry::{Rect, point, size};
use pushrod::widget::Widget;
use pushrod_widgets::base_widget::BaseWidget;
use pushrod_widgets::box_widget::BoxWidget;
use pushrod_widgets::button_widget::ButtonWidget;
use pushrod_widgets::text_widget::{TextJustify, TextWidget};

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

#[test]
fn base_widget_fills() {
    let mut w = BaseWidget::new(point(0, 0), size(10, 10));
    w.set_color(Color::GRAY);
    assert_eq!(painted(&w), vec![PaintCall::Fill(Color::GRAY)]);
}

#[test]
fn box_widget_fills_then_borders() {
    let mut w = BoxWidget::new(point(0, 0), size(10, 10), Color::BLUE, 2);
    w.set_color(Color::CYAN);
    assert_eq!(
        painted(&w),
        vec![
            PaintCall::Fill(Color::CYAN),
            PaintCall::Rect(Rect::new(0, 0, 10, 10), Color::BLUE),
            PaintCall::Rect(Rect::new(1, 1, 8, 8), Color::BLUE),
        ]
    );
}

#[test]
fn text_widget_fills_then_draws_text() {
    let mut w = TextWidget::new(
        point(0, 0),
        size(100, 20),
        String::from("font.ttf"),
        FontStyle::NORMAL,
        10,
        Color::BLACK,
        TextJustify::Left,
        String::from("Hello"),
    );
    w.set_color(Color::WHITE);
    assert_eq!(
        painted(&w),
        vec![
            PaintCall::Fill(Color::WHITE),
            PaintCall::Text(
                Rect::new(0, 0, 100, 20),
                String::from("Hello"),
                Color::BLACK
            ),
        ]
    );
}

#[test]
fn button_widget_draws_label_inset_and_frame() {
    let mut w = ButtonWidget::new(
        point(0, 0),
        size(60, 20),
        String::from("font.ttf"),
        FontStyle::NORMAL,
        10,
        Color::BLACK,
        TextJustify::Center,
        String::from("OK"),
        2,
    );
    w.set_color(Color::WHITE);
    assert_eq!(
        painted(&w),
        vec![
            PaintCall::Fill(Color::WHITE),
            PaintCall::Text(Rect::new(3, 0, 54, 20), String::from("OK"), Color::BLACK),
            PaintCall::Rect(Rect::new(0, 0, 60, 20), Color::BLACK),
        ]
    );
}
