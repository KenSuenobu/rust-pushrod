//! High-DPI drawing with the real `SDL2` painter (issue #25), on SDL's software renderer: at
//! 1x, 1.5x and 2x a one-pixel border covers whole physical pixels, and text is rasterized at
//! the scaled font size instead of being stretched.

use pushrod::color::Color;
use pushrod::font::Font;
use pushrod::geometry::{Rect, stroke_width, to_physical};
use pushrod::paint::{Painter, TextJustify, TextStyle};
use pushrod_sdl2::Sdl2Painter;
use pushrod_sdl2::font::FontCache;
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;

/// The pixels of an area of the canvas, as rows of "is it dark" flags.
///
/// # Arguments
///
/// * `canvas` - The canvas to read.
/// * `area` - The area, in physical pixels.
///
/// # Returns
///
/// One row per physical pixel row, `true` where the pixel is darker than mid-gray.
fn dark_pixels(canvas: &Canvas<Window>, area: Rect) -> Vec<Vec<bool>> {
    let bytes = canvas
        .read_pixels(
            Some(sdl2::rect::Rect::new(area.x, area.y, area.w, area.h)),
            PixelFormatEnum::ABGR8888,
        )
        .unwrap();

    bytes
        .chunks(area.w as usize * 4)
        .map(|row| row.chunks(4).map(|px| px[0] < 128).collect())
        .collect()
}

/// Clears the canvas to white.
///
/// # Arguments
///
/// * `canvas` - The canvas.
fn clear(canvas: &mut Canvas<Window>) {
    canvas.set_clip_rect(None);
    canvas.set_draw_color(SdlColor::WHITE);
    canvas.clear();
}

#[test]
fn borders_and_text_are_drawn_at_physical_resolution() {
    if std::env::var_os("SDL_VIDEODRIVER").is_none() {
        // SAFETY: set before any other thread in this test binary reads the environment.
        unsafe { std::env::set_var("SDL_VIDEODRIVER", "dummy") };
    }

    let sdl = sdl2::init().unwrap();
    let window = sdl
        .video()
        .unwrap()
        .window("high-dpi", 200, 120)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().software().build().unwrap();
    let mut fonts = FontCache::new();
    let style = TextStyle::new(
        Font::default().with_size(14),
        Color::BLACK,
        TextJustify::Left,
    );
    let logical = Rect::new(2, 2, 40, 20);
    let mut ink_rows = Vec::new();

    for scale in [1.0, 1.5, 2.0] {
        // A one-pixel border: exactly `stroke_width` physical pixels on each side.
        clear(&mut canvas);
        Sdl2Painter::new(&mut canvas, &mut fonts)
            .with_scale(scale)
            .rect(logical, Color::BLACK);

        let area = to_physical(logical, scale);
        let pixels = dark_pixels(&canvas, area);
        let middle = &pixels[pixels.len() / 2];
        let t = stroke_width(scale) as usize;

        assert!(middle[..t].iter().all(|d| *d), "left edge at {scale}x");
        assert!(!middle[t], "inside the left edge at {scale}x");
        assert!(
            middle[middle.len() - t..].iter().all(|d| *d),
            "right at {scale}x"
        );
        assert!(
            !middle[middle.len() - t - 1],
            "inside the right edge at {scale}x"
        );

        // Text: the logical line height stays about the same, so the glyphs were rasterized
        // larger rather than drawn at 1x and stretched.
        clear(&mut canvas);

        let mut painter = Sdl2Painter::new(&mut canvas, &mut fonts).with_scale(scale);
        let line = painter.line_height(&style);

        painter.text(Rect::new(0, 0, 100, 40), &style, Color::BLACK, "Hg");
        assert!(painter.take_errors().is_empty());

        let rows = dark_pixels(&canvas, to_physical(Rect::new(0, 0, 100, 40), scale))
            .iter()
            .filter(|row| row.iter().any(|d| *d))
            .count();

        ink_rows.push((scale, line, rows));
    }

    let (_, line_1x, rows_1x) = ink_rows[0];

    for (scale, line, rows) in &ink_rows[1..] {
        assert!(line.abs_diff(line_1x) <= 1, "line height at {scale}x");
        assert!(
            *rows as f32 >= rows_1x as f32 * scale - 2.0,
            "text at {scale}x has {rows} rows of ink, 1x has {rows_1x}"
        );
    }
}
