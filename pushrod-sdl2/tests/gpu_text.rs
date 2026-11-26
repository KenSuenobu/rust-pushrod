//! GPU text (issue #3): fonts load once, glyphs rasterize once, and repainting only copies
//! atlas textures.  Runs on SDL's dummy video driver with the software renderer, so no display
//! is needed.
//!
//! The checks share one test function because SDL's video subsystem is initialized once.

use pushrod::geometry::Point;
use pushrod_sdl2::font::FontCache;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

const FONT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/OpenSans-Regular.ttf"
);

/// Counts the pixels on the canvas that are not the white background.
fn inked_pixels(canvas: &Canvas<Window>) -> Vec<Color> {
    let (w, h) = canvas.output_size().unwrap();
    let bytes = canvas.read_pixels(None, PixelFormatEnum::ARGB8888).unwrap();

    assert_eq!(bytes.len(), (w * h * 4) as usize);

    bytes
        .chunks(4)
        .map(|p| Color::RGBA(p[2], p[1], p[0], p[3]))
        .filter(|c| *c != Color::RGBA(255, 255, 255, 255))
        .collect()
}

#[test]
fn text_is_rasterized_once_and_repainted_with_texture_copies() {
    // SAFETY: set before any other thread in this test binary reads the environment.
    unsafe { std::env::set_var("SDL_VIDEODRIVER", "dummy") };

    let sdl = sdl2::init().unwrap();
    let window = sdl
        .video()
        .unwrap()
        .window("gpu text", 240, 120)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().software().build().unwrap();
    let mut fonts = FontCache::new();

    canvas.set_draw_color(Color::WHITE);
    canvas.clear();

    // Warm-up: the font loads and the glyphs rasterize.
    let block = fonts.layout(&canvas, FONT, 16, FontStyle::NORMAL, "Hello Pushrod", 240);
    let warm = fonts.stats();

    assert_eq!(warm.fonts_loaded, 1);
    assert_eq!(warm.atlas_pages, 1);
    // "Hello Pushrod" has 10 distinct letters (H e l o P u s h r d) and a space that draws nothing.
    assert_eq!(warm.glyphs_rasterized, 10);
    assert!(block.size.w > 0 && block.size.h > 0);

    // Repainting, with a different color each time, never loads or rasterizes anything.
    for color in [Color::RED, Color::GREEN, Color::BLUE] {
        let again = fonts.layout(&canvas, FONT, 16, FontStyle::NORMAL, "Hello Pushrod", 240);

        assert_eq!(again, block);

        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        fonts.draw(&mut canvas, &again, Point::new(4, 4), color);

        let ink = inked_pixels(&canvas);

        assert!(!ink.is_empty(), "text must be drawn");
        // Color modulation tints the white atlas.
        let darkest = ink
            .iter()
            .map(|p| (p.r as u32, p.g as u32, p.b as u32))
            .min_by_key(|(r, g, b)| r + g + b)
            .unwrap();
        // The channels `color` lacks drop to (near) zero where the glyph is solid.
        if color.r == 0 {
            assert!(darkest.0 < 40, "red channel should be modulated away");
        }
        if color.g == 0 {
            assert!(darkest.1 < 40, "green channel should be modulated away");
        }
        assert_eq!(fonts.stats(), warm, "repainting must not load or rasterize");
    }

    // Changing the text rasterizes only the glyphs that are new.
    fonts.layout(&canvas, FONT, 16, FontStyle::NORMAL, "Hello Pushrod!", 240);
    assert_eq!(fonts.stats().glyphs_rasterized, 11);
    assert_eq!(fonts.stats().fonts_loaded, 1);

    // A new size or style is a new font and its own glyphs.
    fonts.layout(&canvas, FONT, 24, FontStyle::NORMAL, "Hi", 240);
    assert_eq!(fonts.stats().fonts_loaded, 2);

    // Wrapping: a narrow width breaks into more lines, so the block is taller.
    let wide = fonts.layout(
        &canvas,
        FONT,
        16,
        FontStyle::NORMAL,
        "one two three four",
        400,
    );
    let narrow = fonts.layout(
        &canvas,
        FONT,
        16,
        FontStyle::NORMAL,
        "one two three four",
        60,
    );

    assert!(narrow.size.h > wide.size.h);
    assert!(narrow.size.w <= 60 + 20);

    // Newlines always break.
    let two = fonts.layout(&canvas, FONT, 16, FontStyle::NORMAL, "a\nb", 400);
    let one = fonts.layout(&canvas, FONT, 16, FontStyle::NORMAL, "a", 400);

    assert_eq!(two.size.h, one.size.h * 2);
}

#[test]
fn a_new_font_cache_has_loaded_nothing() {
    let fonts = FontCache::new();

    assert_eq!(fonts.stats().fonts_loaded, 0);
    assert_eq!(fonts.stats().glyphs_rasterized, 0);
}
