//! Test application for GPU text (issue #3).
//!
//! Draws a few fictional strings in several colors through the glyph atlas and prints the font
//! cache counters after warm-up and after repainting: the second report must show no new fonts
//! loaded and no new glyphs rasterized.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example gpu_text`.  Set `SDL_VIDEODRIVER=dummy` to run without a
//! display.

use pushrod::geometry::Point;
use pushrod_sdl2::font::FontCache;
use sdl2::pixels::Color;
use sdl2::ttf::FontStyle;

const FONT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/OpenSans-Regular.ttf"
);

fn main() {
    let sdl = sdl2::init().expect("SDL2 must initialize");
    let window = sdl
        .video()
        .expect("SDL2 video must initialize")
        .window("GPU text", 320, 160)
        .build()
        .expect("the window must open");
    let mut canvas = window.into_canvas().build().expect("the canvas must build");
    let mut fonts = FontCache::new();
    let lines = [
        ("Quarterly report for Example Corp", Color::BLACK),
        (
            "Wrapped text keeps wrapping at the width of the block.",
            Color::RED,
        ),
        ("Same glyphs, new colors.", Color::BLUE),
    ];

    for frame in 0..2 {
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        let mut y = 4;

        for (text, color) in lines {
            let block = fonts.layout(&canvas, FONT, 16, FontStyle::NORMAL, text, 312);

            fonts.draw(&mut canvas, &block, Point::new(4, y), color);
            y += block.size.h as i32 + 4;
        }

        canvas.present();
        println!("frame {frame}: {:?}", fonts.stats());
    }
}
