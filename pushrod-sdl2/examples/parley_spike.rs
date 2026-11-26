// Parley Compatibility Spike
// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The compatibility gate for the Parley text layout engine (issue #111).
//!
//! Renders three labels and a text field (with a caret and a selection) through
//! Parley -> swash glyph bitmap -> SDL2 atlas texture -> `Canvas::copy`, the same texture and
//! blit path every other widget uses.  It prints the three gate measurements:
//!
//! * (a) glyphs rasterized after warm-up (must be 0 across the measured frames),
//! * (b) the draw path (atlas texture copies only),
//! * (c) binary size and the time to create the engine and lay out the first text.
//!
//! Run it with `cargo run --release -p pushrod-sdl2 --example parley_spike --features parley`.  Pass `--window`
//! to keep the window open until it is closed, or `--screenshot <file.png>` to save the last frame;
//! otherwise it draws a few frames and exits.  Set
//! `SDL_VIDEODRIVER=dummy` to run without a display.
//!
//! All sample text is fictional.

use pushrod::color::{Color, FontStyle};
use pushrod::font::Font;
use pushrod::geometry::Point;
use pushrod::paint::{TextJustify, TextStyle};
use pushrod::text::raster::{GlyphCache, ShelfPacker};
use pushrod::text::{LaidOut, LayoutCache, ParleyLayout};
use sdl2::image::SaveSurface;
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;
use std::collections::HashMap;
use std::time::Instant;

/// Atlas texture size in pixels.
const ATLAS: u32 = 1024;

/// Frames drawn after the warm-up frame.
const MEASURED_FRAMES: usize = 30;

/// Where a glyph's bitmap sits in the atlas.
struct Placed {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    left: i32,
    top: i32,
    color: bool,
}

/// The glyph atlas: rasterizes through the cache and uploads each bitmap once.
struct Atlas {
    texture: Texture,
    packer: ShelfPacker,
    glyphs: GlyphCache,
    placed: HashMap<(u64, u32, u32, u32), Option<Placed>>,
    uploads: usize,
}

impl Atlas {
    /// Creates an empty atlas texture.
    fn new(canvas: &Canvas<Window>) -> Self {
        let mut texture = canvas
            .texture_creator()
            .create_texture_streaming(PixelFormatEnum::ABGR8888, ATLAS, ATLAS)
            .expect("atlas texture");
        texture.set_blend_mode(BlendMode::Blend);
        Self {
            texture,
            packer: ShelfPacker::new(ATLAS, ATLAS),
            glyphs: GlyphCache::new(),
            placed: HashMap::new(),
            uploads: 0,
        }
    }

    /// Draws a laid-out text with its top-left corner at `(x, y)`.  A glyph seen for the first
    /// time is rasterized and uploaded; every other glyph is a texture copy.
    fn draw(&mut self, canvas: &mut Canvas<Window>, laid: &LaidOut, x: i32, y: i32, color: Color) {
        for run in &laid.runs {
            for g in &run.glyphs {
                let key = (run.font.id, run.font.index, g.id, run.font_size.to_bits());

                if !self.placed.contains_key(&key) {
                    let placed = self.upload(&run.font, g.id, run.font_size);
                    self.placed.insert(key, placed);
                }

                if let Some(p) = &self.placed[&key] {
                    if p.color {
                        self.texture.set_color_mod(255, 255, 255);
                    } else {
                        self.texture.set_color_mod(color.r, color.g, color.b);
                    }
                    let src = SdlRect::new(p.x as i32, p.y as i32, p.w, p.h);
                    let dst = SdlRect::new(
                        x + (g.x.round() as i32) + p.left,
                        y + (g.y.round() as i32) - p.top,
                        p.w,
                        p.h,
                    );
                    canvas.copy(&self.texture, src, dst).expect("copy glyph");
                }
            }
        }
    }

    /// Rasterizes a glyph and copies its pixels into the atlas texture.
    fn upload(&mut self, font: &pushrod::text::FontData, id: u32, size: f32) -> Option<Placed> {
        let bitmap = self.glyphs.get(font, id, size)?.clone();
        let slot = self
            .packer
            .pack(bitmap.width, bitmap.height)
            .expect("atlas has room");

        let rgba: Vec<u8> = if bitmap.color {
            bitmap.data.clone()
        } else {
            bitmap
                .data
                .iter()
                .flat_map(|&a| [255, 255, 255, a])
                .collect()
        };

        let rect = SdlRect::new(slot.x as i32, slot.y as i32, bitmap.width, bitmap.height);
        self.texture
            .update(rect, &rgba, bitmap.width as usize * 4)
            .expect("upload glyph");
        self.uploads += 1;

        Some(Placed {
            x: slot.x,
            y: slot.y,
            w: bitmap.width,
            h: bitmap.height,
            left: bitmap.left,
            top: bitmap.top,
            color: bitmap.color,
        })
    }
}

/// Builds a text style for the bundled font.
fn style(size: u16) -> TextStyle {
    TextStyle {
        font: Font::new("", size, FontStyle::NORMAL),
        font_color: Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
        justification: TextJustify::Left,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let keep_open = args.iter().any(|a| a == "--window");

    let sdl = sdl2::init().expect("sdl");
    let window = sdl
        .video()
        .expect("video")
        .window("Pushrod Parley spike", 640, 360)
        .position_centered()
        .build()
        .expect("window");
    let mut canvas = window.into_canvas().build().expect("canvas");
    let mut events = sdl.event_pump().expect("events");

    let started = Instant::now();
    let mut engine = ParleyLayout::new();
    let created = started.elapsed();

    let mut atlas = Atlas::new(&canvas);

    // One cache per widget: the layout is computed only when text, font, or width change.
    let samples: [(&str, u16, i32); 3] = [
        ("Hello, Pushrod: Latin text with fi ligatures", 22, 12),
        ("שלום עולם — right-to-left Hebrew and Latin", 22, 52),
        (
            "日本語のテキスト and emoji 😀🎉 fall back to other fonts",
            22,
            92,
        ),
    ];
    let mut caches: Vec<LayoutCache> = samples.iter().map(|_| LayoutCache::new()).collect();
    let mut field = LayoutCache::new();
    let field_text = "Jane Example";

    let frame = |canvas: &mut Canvas<Window>,
                 atlas: &mut Atlas,
                 engine: &mut ParleyLayout,
                 caches: &mut [LayoutCache],
                 field: &mut LayoutCache| {
        canvas.set_draw_color(SdlColor::RGB(250, 250, 250));
        canvas.clear();

        for ((text, size, y), cache) in samples.iter().zip(caches.iter_mut()) {
            let laid = cache.get(engine, text, &style(*size), Some(600));
            atlas.draw(
                canvas,
                laid,
                16,
                *y,
                Color {
                    r: 20,
                    g: 20,
                    b: 20,
                    a: 255,
                },
            );
        }

        // A text field: border, selection of "Exa", caret after "Jane E".
        canvas.set_draw_color(SdlColor::RGB(120, 120, 120));
        canvas
            .draw_rect(SdlRect::new(16, 150, 300, 32))
            .expect("border");
        let laid = field.get(engine, field_text, &style(20), None);
        canvas.set_draw_color(SdlColor::RGB(180, 205, 255));
        for r in laid.geometry.selection_rects(5..8) {
            canvas
                .fill_rect(SdlRect::new(22 + r.x, 156 + r.y, r.w, r.h))
                .expect("selection");
        }
        let caret = laid.geometry.caret_rect(8);
        canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
        canvas
            .fill_rect(SdlRect::new(22 + caret.x, 156 + caret.y, caret.w, caret.h))
            .expect("caret");
        atlas.draw(
            canvas,
            laid,
            22,
            156,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        );

        assert_eq!(
            laid.geometry.hit_test(Point {
                x: caret.x,
                y: caret.y + 2
            }),
            8,
            "hit test inverts the caret"
        );
        canvas.present();
    };

    // Warm-up: the first frame rasterizes and uploads every glyph.
    let first_started = Instant::now();
    frame(
        &mut canvas,
        &mut atlas,
        &mut engine,
        &mut caches,
        &mut field,
    );
    let first = first_started.elapsed();
    let (warm_raster, warm_uploads) = (atlas.glyphs.rasterized(), atlas.uploads);

    // Measured frames: nothing may rasterize or upload.
    let measured = Instant::now();
    for _ in 0..MEASURED_FRAMES {
        frame(
            &mut canvas,
            &mut atlas,
            &mut engine,
            &mut caches,
            &mut field,
        );
    }
    let per_frame = measured.elapsed() / MEASURED_FRAMES as u32;

    let new_raster = atlas.glyphs.rasterized() - warm_raster;
    let new_uploads = atlas.uploads - warm_uploads;
    let layouts: usize = caches.iter().map(LayoutCache::layouts).sum::<usize>() + field.layouts();
    let binary = std::env::current_exe()
        .and_then(std::fs::metadata)
        .map(|m| m.len())
        .unwrap_or(0);

    println!("Parley compatibility gate");
    println!("  warm-up: {warm_raster} glyphs rasterized, {warm_uploads} uploaded to the atlas");
    println!(
        "  (a) after warm-up, over {MEASURED_FRAMES} frames: {new_raster} rasterized, {new_uploads} uploaded (must be 0)"
    );
    println!(
        "  (b) draw path: Canvas::copy from one atlas texture with color modulation (same blit path as widgets)"
    );
    println!(
        "  (c) binary: {:.1} MiB; engine + system font discovery: {:?}; first frame: {:?}; steady frame: {:?}",
        binary as f64 / 1_048_576.0,
        created,
        first,
        per_frame
    );
    println!(
        "  layouts computed: {layouts} for {} widgets (cached across frames)",
        samples.len() + 1
    );

    assert_eq!(
        new_raster, 0,
        "gate (a) failed: glyphs rasterized after warm-up"
    );
    assert_eq!(
        new_uploads, 0,
        "gate (a) failed: atlas uploads after warm-up"
    );
    assert_eq!(
        layouts,
        samples.len() + 1,
        "layouts must be computed once per widget"
    );

    if let Some(path) = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
    {
        let (w, h) = canvas.output_size().expect("output size");
        let mut pixels = canvas
            .read_pixels(None, PixelFormatEnum::ABGR8888)
            .expect("read pixels");
        let surface =
            sdl2::surface::Surface::from_data(&mut pixels, w, h, w * 4, PixelFormatEnum::ABGR8888)
                .expect("surface");
        surface.save(path).expect("save screenshot");
    }

    while !events
        .keyboard_state()
        .is_scancode_pressed(sdl2::keyboard::Scancode::Escape)
        && keep_open
    {
        for e in events.poll_iter() {
            if matches!(e, sdl2::event::Event::Quit { .. }) {
                return;
            }
        }
        frame(
            &mut canvas,
            &mut atlas,
            &mut engine,
            &mut caches,
            &mut field,
        );
    }
}
