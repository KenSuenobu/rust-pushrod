// Paint tests.  Each widget paints into a RecordingPainter, so no window or display is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::{Color, FontStyle};
use pushrod::geometry::{Rect, point, size};
use pushrod::paint::Truncate;
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

fn plain_button() -> ButtonWidget {
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
    w
}

#[test]
fn button_hover_style_changes_the_fill_and_text_color_only_while_hovered() {
    use pushrod::appearance::StateStyle;
    use pushrod::widget::VisualState;

    let mut w = plain_button();
    let normal = painted(&w);

    w.set_style(
        VisualState::HOVERED,
        StateStyle {
            background: Some(Color::BLUE),
            foreground: Some(Color::WHITE),
            ..StateStyle::default()
        },
    );
    assert_eq!(
        painted(&w),
        normal,
        "not hovered yet: the look is unchanged"
    );

    w.set_state(VisualState::HOVERED);

    let hovered = painted(&w);

    assert_eq!(hovered[0], PaintCall::Fill(Color::BLUE));
    assert!(matches!(&hovered[1], PaintCall::Text(_, _, c) if *c == Color::WHITE));

    w.set_state(VisualState::empty());
    assert_eq!(painted(&w), normal, "leaving restores the default look");
}

#[test]
fn button_pressed_style_overrides_the_inverted_default() {
    use pushrod::appearance::StateStyle;
    use pushrod::widget::VisualState;

    let mut w = plain_button();

    w.set_style(
        VisualState::PRESSED,
        StateStyle {
            background: Some(Color::RGB(0, 0, 128)),
            border_color: Some(Color::BLUE),
            border_width: Some(2),
            ..StateStyle::default()
        },
    );
    w.set_state(VisualState::PRESSED);

    let calls = painted(&w);

    assert_eq!(calls[0], PaintCall::Fill(Color::RGB(0, 0, 128)));
    assert_eq!(
        calls[2],
        PaintCall::Rect(Rect::new(0, 0, 60, 20), Color::BLUE)
    );
    assert_eq!(calls.len(), 4, "fill, text, and a two-pixel border");
}

#[test]
fn box_base_and_text_widgets_honor_their_styles() {
    use pushrod::appearance::StateStyle;
    use pushrod::widget::VisualState;

    let style = StateStyle {
        background: Some(Color::YELLOW),
        foreground: Some(Color::RED),
        border_color: Some(Color::GREEN),
        border_width: Some(1),
        ..StateStyle::default()
    };

    let mut base = BaseWidget::new(point(0, 0), size(10, 10));
    base.set_style(VisualState::empty(), style.clone());
    assert_eq!(painted(&base), vec![PaintCall::Fill(Color::YELLOW)]);

    let mut b = BoxWidget::new(point(0, 0), size(10, 10), Color::BLUE, 2);
    b.set_style(VisualState::empty(), style.clone());
    assert_eq!(
        painted(&b),
        vec![
            PaintCall::Fill(Color::YELLOW),
            PaintCall::Rect(Rect::new(0, 0, 10, 10), Color::GREEN),
        ]
    );

    let mut t = TextWidget::new(
        point(0, 0),
        size(100, 20),
        String::from("font.ttf"),
        FontStyle::NORMAL,
        10,
        Color::BLACK,
        TextJustify::Left,
        String::from("Hi"),
    );
    t.set_style(VisualState::empty(), style);
    assert!(matches!(
        &painted(&t)[..],
        [PaintCall::Fill(f), PaintCall::Text(_, _, c)] if *f == Color::YELLOW && *c == Color::RED
    ));
}

/// A 50-pixel-wide text widget (ten characters at 5 pixels each in the recording painter).
fn narrow_text(msg: &str) -> TextWidget {
    TextWidget::new(
        point(0, 0),
        size(50, 20),
        String::from("font.ttf"),
        FontStyle::NORMAL,
        10,
        Color::BLACK,
        TextJustify::Left,
        String::from(msg),
    )
}

fn drawn_string(calls: &[PaintCall]) -> String {
    calls
        .iter()
        .find_map(|c| match c {
            PaintCall::Text(_, t, _) => Some(t.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn text_and_button_widgets_truncate_at_the_end_by_default() {
    let t = narrow_text("Quarterly report.txt");

    assert_eq!(t.get_truncate(), Truncate::End);
    assert!(!t.is_truncated(), "nothing painted yet");
    assert_eq!(drawn_string(&painted(&t)), "Quarterly\u{2026}");
    assert!(t.is_truncated());

    let mut b = ButtonWidget::new(
        point(0, 0),
        size(50, 20),
        String::from("font.ttf"),
        FontStyle::NORMAL,
        10,
        Color::BLACK,
        TextJustify::Center,
        String::from("Quarterly report.txt"),
        0,
    );

    assert_eq!(b.get_truncate(), Truncate::End);
    assert_eq!(drawn_string(&painted(&b)), "Quarterl\u{2026}");
    assert!(b.is_truncated());

    b.set_text(String::from("OK"));
    assert_eq!(drawn_string(&painted(&b)), "OK");
    assert!(!b.is_truncated(), "short text is drawn unchanged");
}

#[test]
fn text_widget_truncates_in_every_mode() {
    let mut t = narrow_text("Quarterly report.txt");

    for (mode, expected, truncated) in [
        (Truncate::End, "Quarterly\u{2026}", true),
        (Truncate::Start, "\u{2026}eport.txt", true),
        (Truncate::Middle, "Quart\u{2026}.txt", true),
        (Truncate::None, "Quarterly report.txt", false),
    ] {
        t.set_truncate(mode);
        assert_eq!(drawn_string(&painted(&t)), expected, "{mode:?}");
        assert_eq!(t.is_truncated(), truncated, "{mode:?}");
    }
}

#[test]
fn changing_the_truncate_mode_invalidates_the_widget() {
    let mut t = narrow_text("Quarterly report.txt");

    t.set_invalidated(false);
    t.set_truncate(Truncate::End);
    assert!(!t.is_invalidated(), "the same mode changes nothing");
    t.set_truncate(Truncate::Middle);
    assert!(t.is_invalidated());
}
