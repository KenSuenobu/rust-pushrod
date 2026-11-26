// ImageWidget tests.  The widget paints into a RecordingPainter, so no window is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{Event, EventMask};
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::Widget;
use pushrod_widgets::image_widget::{FitMode, ImageWidget, MAX_TILES, layout};
use pushrod_widgets::prelude::ImageWidget as PreludeImageWidget;

/// Paints the widget with `logo.png` registered as a 40 by 20 image.
fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    p.images.insert("logo.png".to_string(), size(40, 20));
    widget.paint(&mut p);
    p.calls
}

fn parts(calls: &[PaintCall]) -> Vec<(Option<Rect>, Rect)> {
    calls
        .iter()
        .filter_map(|call| match call {
            PaintCall::ImagePart(_, source, bounds) => Some((*source, *bounds)),
            _ => None,
        })
        .collect()
}

#[test]
fn stretch_fills_the_widget() {
    assert_eq!(
        layout(FitMode::Stretch, size(40, 20), size(100, 100)),
        vec![(None, Rect::new(0, 0, 100, 100))]
    );
}

#[test]
fn center_draws_at_its_own_size_in_the_middle() {
    assert_eq!(
        layout(FitMode::Center, size(40, 20), size(100, 100)),
        vec![(None, Rect::new(30, 40, 40, 20))]
    );
}

#[test]
fn center_crops_an_image_larger_than_the_widget() {
    assert_eq!(
        layout(FitMode::Center, size(100, 20), size(40, 100)),
        vec![(Some(Rect::new(30, 0, 40, 20)), Rect::new(0, 40, 40, 20))]
    );
}

#[test]
fn fit_keeps_the_aspect_ratio_and_centers() {
    // 40x20 into 100x100: width limits, so 100x50, centered vertically.
    assert_eq!(
        layout(FitMode::Fit, size(40, 20), size(100, 100)),
        vec![(None, Rect::new(0, 25, 100, 50))]
    );
    // 20x40 into 100x100: height limits.
    assert_eq!(
        layout(FitMode::Fit, size(20, 40), size(100, 100)),
        vec![(None, Rect::new(25, 0, 50, 100))]
    );
}

#[test]
fn fit_never_collapses_to_zero_pixels() {
    let placed = layout(FitMode::Fit, size(1000, 1), size(10, 10));

    assert_eq!(placed, vec![(None, Rect::new(0, 4, 10, 1))]);
}

#[test]
fn tile_repeats_and_crops_the_edges() {
    let placed = layout(FitMode::Tile, size(40, 20), size(100, 30));

    assert_eq!(
        placed,
        vec![
            (None, Rect::new(0, 0, 40, 20)),
            (None, Rect::new(40, 0, 40, 20)),
            (Some(Rect::new(0, 0, 20, 20)), Rect::new(80, 0, 20, 20)),
            (Some(Rect::new(0, 0, 40, 10)), Rect::new(0, 20, 40, 10)),
            (Some(Rect::new(0, 0, 40, 10)), Rect::new(40, 20, 40, 10)),
            (Some(Rect::new(0, 0, 20, 10)), Rect::new(80, 20, 20, 10)),
        ]
    );
}

#[test]
fn tile_count_is_capped() {
    let placed = layout(FitMode::Tile, size(1, 1), size(1000, 1000));

    assert_eq!(placed.len() as u32, MAX_TILES);
}

#[test]
fn empty_image_or_area_places_nothing() {
    for mode in [
        FitMode::Center,
        FitMode::Stretch,
        FitMode::Fit,
        FitMode::Tile,
    ] {
        assert!(layout(mode, size(0, 5), size(10, 10)).is_empty());
        assert!(layout(mode, size(5, 5), size(0, 10)).is_empty());
    }
}

#[test]
fn paints_the_background_then_the_image() {
    let image = ImageWidget::new("logo.png")
        .size(100, 100)
        .fit(FitMode::Stretch);
    let calls = painted(&image);

    assert_eq!(calls[0], PaintCall::Fill(Color::RGBA(255, 255, 255, 0)));
    assert_eq!(
        calls[1],
        PaintCall::ImagePart("logo.png".into(), None, Rect::new(0, 0, 100, 100))
    );
    assert_eq!(calls.len(), 2);
}

#[test]
fn default_fit_is_center() {
    let image = ImageWidget::new("logo.png");

    assert_eq!(image.fit_mode(), FitMode::Center);
    assert_eq!(parts(&painted(&image)).len(), 1);
}

#[test]
fn tile_paints_every_tile() {
    let image = ImageWidget::new("logo.png")
        .size(100, 30)
        .fit(FitMode::Tile);

    assert_eq!(parts(&painted(&image)).len(), 6);
}

#[test]
fn a_missing_file_draws_a_crossed_box_and_does_not_panic() {
    let image = ImageWidget::new("nope.png").size(30, 20);
    let calls = painted(&image);

    assert!(parts(&calls).is_empty());
    assert!(calls.contains(&PaintCall::Line(point(0, 0), point(29, 19), Color::BLACK)));
    assert!(calls.contains(&PaintCall::Line(point(0, 19), point(29, 0), Color::BLACK)));
    // The box: the border.
    assert_eq!(
        calls
            .iter()
            .filter(|c| matches!(c, PaintCall::Rect(..)))
            .count(),
        1
    );
}

#[test]
fn the_missing_marker_color_can_be_set() {
    let image = ImageWidget::new("nope.png").missing_color(Color::RED);

    assert!(painted(&image).iter().any(|c| matches!(
        c,
        PaintCall::Line(_, _, color) if *color == Color::RED
    )));
}

#[test]
fn set_path_and_set_fit_invalidate() {
    let mut image = ImageWidget::new("logo.png");
    image.set_invalidated(false);

    image.set_path("other.png");
    assert_eq!(image.path(), "other.png");
    assert!(image.is_invalidated());

    image.set_invalidated(false);
    image.set_fit(FitMode::Tile);
    assert_eq!(image.fit_mode(), FitMode::Tile);
    assert!(image.is_invalidated());
}

#[test]
fn zero_size_draws_only_the_background() {
    let image = ImageWidget::new("logo.png").size(0, 0);

    assert_eq!(
        painted(&image),
        vec![PaintCall::Fill(Color::RGBA(255, 255, 255, 0))]
    );
}

#[test]
fn display_only_by_default_and_clickable_on_request() {
    let plain = ImageWidget::new("logo.png");
    let clickable = PreludeImageWidget::new("logo.png").clickable().clickable();

    assert_eq!(plain.event_mask(), EventMask::NONE);
    assert!(!plain.is_clickable());
    assert!(clickable.is_clickable());
    assert_ne!(clickable.event_mask(), EventMask::NONE);
}

#[test]
fn a_clickable_image_emits_clicked() {
    let mut cache = WidgetCache::new();
    let image = cache.add(Box::new(
        ImageWidget::new("logo.png")
            .at(10, 10)
            .size(50, 50)
            .clickable(),
    ));
    let at = |x, y| InputEvent::MouseMove {
        position: point(x, y),
    };

    cache.handle_event(at(20, 20));
    cache.handle_event(InputEvent::MouseDown {
        position: point(20, 20),
        button: MouseButton::Left,
        clicks: 1,
    });

    let events = cache.handle_event(InputEvent::MouseUp {
        position: point(20, 20),
        button: MouseButton::Left,
        clicks: 1,
    });

    assert!(events.iter().any(|event| matches!(
        event,
        Event::Clicked { id, button: MouseButton::Left, count: 1 } if *id == image
    )));
}

#[test]
fn a_plain_image_emits_nothing() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(
        ImageWidget::new("logo.png").at(10, 10).size(50, 50),
    ));
    cache.handle_event(InputEvent::MouseMove {
        position: point(20, 20),
    });
    cache.handle_event(InputEvent::MouseDown {
        position: point(20, 20),
        button: MouseButton::Left,
        clicks: 1,
    });

    let events = cache.handle_event(InputEvent::MouseUp {
        position: point(20, 20),
        button: MouseButton::Left,
        clicks: 1,
    });

    assert!(!events.iter().any(|e| matches!(e, Event::Clicked { .. })));
}
