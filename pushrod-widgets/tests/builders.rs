//! Tests for the short constructors, their defaults, and the chained setters.

use pushrod::geometry::{point, size};
use pushrod::widget::Widget;
use pushrod_widgets::prelude::*;

#[test]
fn constructors_need_at_most_one_required_argument() {
    let _ = (
        Button::new("OK"),
        Text::new("Hi"),
        Frame::new(),
        Panel::new(),
    );
}

#[test]
fn button_defaults_and_chained_setters() {
    let default = Button::new("OK");
    assert_eq!(default.get_origin(), point(0, 0));
    assert_eq!(default.get_size(), size(100, 24));
    assert_eq!(default.get_text(), "OK");
    assert_eq!(default.get_border_width(), 2);

    let button = Button::new("OK").at(20, 60).size(120, 24).border_width(4);
    assert_eq!(button.get_origin(), point(20, 60));
    assert_eq!(button.get_size(), size(120, 24));
    assert_eq!(button.get_border_width(), 4);
}

#[test]
fn text_frame_and_panel_defaults() {
    let text = Text::new("Hello").at(5, 6).size(50, 10);
    assert_eq!(text.get_origin(), point(5, 6));
    assert_eq!(text.get_size(), size(50, 10));
    assert_eq!(text.get_text(), "Hello");

    let frame = Frame::new();
    assert_eq!(frame.get_size(), size(100, 100));
    assert_eq!(frame.get_border_width(), 1);
    assert_eq!(Frame::default().get_border_width(), 1);

    let panel = Panel::new().at(1, 2).size(3, 4);
    assert_eq!(panel.get_origin(), point(1, 2));
    assert_eq!(panel.get_size(), size(3, 4));
}
