//! Image loading (issue #33): PNG and BMP files load through `SDL2_image` into the shared
//! texture cache once, draw with every fit mode, and a missing file is an error, not a panic.
//! Runs on SDL's dummy video driver with the software renderer, so no display is needed.
//!
//! The checks share one test function because SDL's video subsystem is initialized once.

use pushrod::geometry::{Rect, size};
use pushrod::paint::Painter;
use pushrod_sdl2::font::FontCache;
use pushrod_sdl2::{ImageCache, Sdl2Painter};
use pushrod_widgets::image_widget::{FitMode, layout};
use sdl2::pixels::{Color, PixelFormatEnum};

/// Path to a sample image shipped with the examples.
fn asset(name: &str) -> String {
    format!("{}/examples/assets/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn png_and_bmp_load_once_draw_in_every_fit_mode_and_missing_files_fail() {
    // SAFETY: set before any other thread in this test binary reads the environment.
    unsafe { std::env::set_var("SDL_VIDEODRIVER", "dummy") };

    let sdl = sdl2::init().unwrap();
    let window = sdl
        .video()
        .unwrap()
        .window("image", 120, 80)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().software().build().unwrap();
    let mut fonts = FontCache::new();
    let mut images = ImageCache::new();
    let (png, bmp) = (asset("sample.png"), asset("sample.bmp"));

    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
    canvas.clear();

    {
        let mut painter = Sdl2Painter::new(&mut canvas, &mut fonts).with_images(&mut images);

        // Sizes come from the files; loading twice keeps one cache entry per path.
        assert_eq!(painter.image_size(&png).unwrap(), size(48, 32));
        assert_eq!(painter.image_size(&bmp).unwrap(), size(24, 16));
        assert_eq!(painter.image_size(&png).unwrap(), size(48, 32));

        for mode in [
            FitMode::Center,
            FitMode::Stretch,
            FitMode::Fit,
            FitMode::Tile,
        ] {
            for (source, bounds) in layout(mode, size(48, 32), size(120, 80)) {
                painter.image_part(&png, source, bounds).unwrap();
            }
        }

        assert!(painter.image_size(&asset("missing.png")).is_err());
        assert!(
            painter
                .image_part(&asset("missing.png"), None, Rect::new(0, 0, 10, 10))
                .is_err()
        );
        assert_eq!(painter.take_errors().len(), 1);
    }

    assert_eq!(images.len(), 2);
    assert!(!images.is_empty());

    // The stretched image really reached the canvas: the border pixel is black.
    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
    canvas.clear();
    {
        let mut painter = Sdl2Painter::new(&mut canvas, &mut fonts).with_images(&mut images);

        painter
            .image_part(&png, None, Rect::new(0, 0, 48, 32))
            .unwrap();
    }

    let bytes = canvas.read_pixels(None, PixelFormatEnum::ARGB8888).unwrap();

    assert_eq!(&bytes[0..3], &[0, 0, 0]);

    // Without a cache the painter reports an error instead of panicking.
    let mut bare = Sdl2Painter::new(&mut canvas, &mut fonts);

    assert!(bare.image_size(&png).is_err());
}
