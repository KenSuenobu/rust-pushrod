// Fonts and Text
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

//! # Fonts and the glyph atlas
//!
//! The [`FontCache`] loads each TrueType font once per `(path, size, style)` and rasterizes each
//! glyph once, with SDL2_ttf, into a **glyph atlas**: a GPU texture shared by every text widget.
//! Text is drawn by copying glyph rectangles out of the atlas with `Canvas::copy`, tinted with
//! `set_color_mod`, so after warm-up repainting text, or changing its text or color, performs
//! only texture copies and never rasterizes again.
//!
//! The `WidgetCache` owns the one `FontCache` and hands it to each `Sdl2Painter` while a widget
//! paints.  The glyph runs a [`TextLayout`](pushrod::text::TextLayout) engine produces stay separate:
//! this module is the drawing path for the SDL2 backend.
//!
//! SDL2_ttf is initialized once per process, the first time text is drawn, and never shut down
//! while the process runs.  Its initialization and shutdown are not thread-safe, so creating and
//! dropping engines (for example in tests running in parallel) must never touch it.
//!
//! Atlas textures are owned by the renderer that created them and are released with it; a
//! `FontCache` must not be reused with a different canvas.

use crate::ttf_layout::wrap_lines;
use pushrod::geometry::{Point, Rect, Size};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{BlendMode, Canvas, Texture, TextureAccess};
use sdl2::ttf::{Font, FontStyle, Sdl2TtfContext};
use sdl2::video::Window;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The process-wide SDL2_ttf context, created on first use and kept for the life of the process.
static TTF_CONTEXT: OnceLock<Result<Sdl2TtfContext, String>> = OnceLock::new();

/// Returns the process-wide SDL2_ttf context, initializing SDL2_ttf the first time.
///
/// # Returns
///
/// The context, or the SDL2_ttf error message if initialization failed (the failure is
/// remembered, so it is reported the same way on every call).
pub(crate) fn ttf_context() -> Result<&'static Sdl2TtfContext, String> {
    TTF_CONTEXT
        .get_or_init(|| sdl2::ttf::init().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| e.clone())
}

/// The width and height of a new atlas page in pixels.
const ATLAS_PAGE_SIZE: u32 = 1024;

/// Identifies one loaded font: file path, size in points and style bits.
#[derive(Clone, PartialEq, Eq, Hash)]
struct FaceKey {
    path: String,
    size: u16,
    style: i32,
}

/// Where one rasterized glyph lives in the atlas.
#[derive(Clone, Copy)]
struct Slot {
    /// The index of the atlas page holding the glyph.
    page: usize,
    /// The glyph's rectangle in that page.
    rect: Rect,
}

/// One glyph of a face: its advance and, unless it draws nothing, its atlas slot.
#[derive(Clone, Copy)]
struct Glyph {
    /// How far the pen moves after the glyph, in pixels.
    advance: i32,
    /// The glyph's atlas rectangle; `None` for glyphs with no pixels (spaces, controls).
    slot: Option<Slot>,
}

/// A loaded font and the glyphs rasterized from it.
struct Face {
    font: Font<'static, 'static>,
    glyphs: HashMap<char, Glyph>,
    line_height: i32,
}

/// One atlas texture with a shelf packer.
struct AtlasPage {
    texture: Texture,
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

impl AtlasPage {
    /// Reserves a rectangle in the page.
    ///
    /// # Arguments
    ///
    /// * `w` - The width needed in pixels.
    /// * `h` - The height needed in pixels.
    ///
    /// # Returns
    ///
    /// The reserved rectangle, or `None` if it does not fit.
    fn pack(&mut self, w: u32, h: u32) -> Option<Rect> {
        if self.cursor_x + w > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }

        if self.cursor_y + h > self.height || w > self.width {
            return None;
        }

        let rect = Rect::new(self.cursor_x as i32, self.cursor_y as i32, w, h);

        self.cursor_x += w;
        self.row_height = self.row_height.max(h);

        Some(rect)
    }
}

/// A glyph positioned for drawing, relative to the top-left corner of its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacedGlyph {
    /// The index of the atlas page holding the glyph.
    page: usize,
    /// The glyph's rectangle in that page.
    source: Rect,
    /// The glyph's position relative to the top-left corner of the text.
    position: Point,
}

/// Text that has been laid out and whose glyphs are all in the atlas: drawing it is only copies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphBlock {
    /// The size of the text in pixels.
    pub size: Size,
    /// The glyphs to copy, in drawing order.
    glyphs: Vec<PlacedGlyph>,
}

/// What the font cache has done so far; tests and tools use it to check that repainting does not
/// load or rasterize anything.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FontCacheStats {
    /// Fonts loaded from disk.
    pub fonts_loaded: usize,
    /// Glyphs rasterized into the atlas.
    pub glyphs_rasterized: usize,
    /// Atlas textures created.
    pub atlas_pages: usize,
}

/// Something the font cache did that the engine reports as an
/// [`EngineEvent`](crate::event::EngineEvent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FontNotice {
    /// A font was loaded from disk.
    FontLoaded { font_name: String, font_size: u16 },
    /// A new atlas texture was created; `pages` is how many exist now.
    AtlasPageAdded { pages: usize },
}

/// Loads fonts once and draws text from a glyph atlas on the GPU.
///
/// The [`Sdl2Backend`](crate::Sdl2Backend) owns one and passes it to each painter.
#[derive(Default)]
pub struct FontCache {
    faces: HashMap<FaceKey, Face>,
    pages: Vec<AtlasPage>,
    stats: FontCacheStats,
    notices: Vec<FontNotice>,
}

/// Returns the kerning adjustment between two glyphs.
///
/// # Arguments
///
/// * `font` - The font.
/// * `previous` - The character before the pair.
/// * `next` - The character after the pair.
///
/// # Returns
///
/// The adjustment in pixels (zero or negative for most pairs).
fn kerning(font: &Font<'static, 'static>, previous: char, next: char) -> i32 {
    match (font.find_glyph(previous), font.find_glyph(next)) {
        // SAFETY: `font.raw()` is a live `TTF_Font` for the duration of the call.
        (Some(a), Some(b)) => unsafe {
            sdl2::sys::ttf::TTF_GetFontKerningSizeGlyphs(font.raw(), a, b)
        },
        _ => 0,
    }
}

impl FontCache {
    /// Creates a font cache.
    ///
    /// SDL2_ttf is not touched here; it is initialized once per process when text is first laid
    /// out.
    ///
    /// # Returns
    ///
    /// The new `FontCache`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns what the cache has loaded and rasterized so far.
    ///
    /// # Returns
    ///
    /// The counters.
    pub fn stats(&self) -> FontCacheStats {
        self.stats
    }

    /// Removes and returns the notices recorded since the last call, in order.
    ///
    /// # Returns
    ///
    /// The notices; empty if there are none.
    pub(crate) fn take_notices(&mut self) -> Vec<FontNotice> {
        std::mem::take(&mut self.notices)
    }

    /// Loads a font if it is not loaded yet.
    ///
    /// # Arguments
    ///
    /// * `key` - The font's path, size and style.
    ///
    /// # Panics
    ///
    /// Panics if SDL2_ttf cannot be initialized or the font file cannot be loaded.
    fn ensure_face(&mut self, key: &FaceKey) {
        if self.faces.contains_key(key) {
            return;
        }

        let mut font = ttf_context()
            .expect("SDL2_ttf must initialize")
            .load_font(&key.path, key.size)
            .expect("the font file must load");

        font.set_style(FontStyle::from_bits_truncate(key.style));

        let line_height = font.height().max(1);

        self.faces.insert(
            key.clone(),
            Face {
                font,
                glyphs: HashMap::new(),
                line_height,
            },
        );
        self.stats.fonts_loaded += 1;
        self.notices.push(FontNotice::FontLoaded {
            font_name: key.path.clone(),
            font_size: key.size,
        });
    }

    /// Rasterizes a glyph into the atlas if it is not there yet.
    ///
    /// # Arguments
    ///
    /// * `c` - The canvas whose renderer creates and updates atlas textures.
    /// * `key` - The font the glyph belongs to; it must already be loaded.
    /// * `ch` - The character.
    ///
    /// # Returns
    ///
    /// The glyph's advance and atlas slot.
    ///
    /// # Panics
    ///
    /// Panics if SDL2 cannot create or update an atlas texture.
    fn ensure_glyph(&mut self, c: &Canvas<Window>, key: &FaceKey, ch: char) -> Glyph {
        let face = self.faces.get_mut(key).expect("the face is loaded");

        if let Some(glyph) = face.glyphs.get(&ch) {
            return *glyph;
        }

        let advance = face.font.find_glyph_metrics(ch).map_or_else(
            || face.font.size_of_char(ch).map_or(0, |(w, _)| w as i32),
            |m| m.advance,
        );

        // Whitespace and control characters draw nothing, so they get no atlas space.
        let surface = if ch.is_whitespace() || ch.is_control() {
            None
        } else {
            face.font
                .render_char(ch)
                .blended(Color::RGBA(255, 255, 255, 255))
                .ok()
                .and_then(|surface| surface.convert_format(PixelFormatEnum::ARGB8888).ok())
        };

        let slot = surface.and_then(|surface| {
            let (w, h) = (surface.width(), surface.height());
            let page = self.pack(c, w, h)?;
            let pitch = surface.pitch() as usize;

            surface.with_lock(|pixels| {
                self.pages[page.page]
                    .texture
                    .update(
                        sdl2::rect::Rect::new(page.rect.x, page.rect.y, w, h),
                        pixels,
                        pitch,
                    )
                    .expect("SDL2 must update the glyph atlas");
            });
            self.stats.glyphs_rasterized += 1;

            Some(page)
        });

        let glyph = Glyph { advance, slot };

        self.faces
            .get_mut(key)
            .expect("the face is loaded")
            .glyphs
            .insert(ch, glyph);

        glyph
    }

    /// Reserves room for a glyph bitmap, adding an atlas page when none has room.
    ///
    /// # Arguments
    ///
    /// * `c` - The canvas whose renderer creates the page.
    /// * `w` - The bitmap width in pixels.
    /// * `h` - The bitmap height in pixels.
    ///
    /// # Returns
    ///
    /// The reserved slot, or `None` for an empty bitmap.
    ///
    /// # Panics
    ///
    /// Panics if SDL2 cannot create the atlas texture.
    fn pack(&mut self, c: &Canvas<Window>, w: u32, h: u32) -> Option<Slot> {
        if w == 0 || h == 0 {
            return None;
        }

        if let Some(page) = self.pages.last_mut()
            && let Some(rect) = page.pack(w, h)
        {
            return Some(Slot {
                page: self.pages.len() - 1,
                rect,
            });
        }

        // A bitmap larger than a standard page gets a page of its own.
        let (page_w, page_h) = (ATLAS_PAGE_SIZE.max(w), ATLAS_PAGE_SIZE.max(h));
        let mut texture = c
            .texture_creator()
            .create_texture(
                PixelFormatEnum::ARGB8888,
                TextureAccess::Static,
                page_w,
                page_h,
            )
            .expect("SDL2 must create the glyph atlas");

        texture.set_blend_mode(BlendMode::Blend);

        let mut page = AtlasPage {
            texture,
            width: page_w,
            height: page_h,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
        };
        let rect = page.pack(w, h).expect("a new page fits the bitmap");

        self.pages.push(page);
        self.stats.atlas_pages += 1;
        self.notices.push(FontNotice::AtlasPageAdded {
            pages: self.pages.len(),
        });

        Some(Slot {
            page: self.pages.len() - 1,
            rect,
        })
    }

    /// Lays text out with the font's metrics and kerning, rasterizing any glyph not yet in the
    /// atlas.
    ///
    /// The text is wrapped at spaces to `width` pixels (words wider than `width` stay whole) and
    /// broken at newlines; lines are left aligned within the block.
    ///
    /// # Arguments
    ///
    /// * `c` - The canvas whose renderer creates and updates atlas textures.
    /// * `font_name` - Path to the TrueType font file.
    /// * `font_size` - The font size in points.
    /// * `font_style` - The font style (normal, bold, italic, ...).
    /// * `text` - The text.
    /// * `width` - The width in pixels at which to wrap the text.
    ///
    /// # Returns
    ///
    /// The laid-out block, ready for [`FontCache::draw`].
    ///
    /// # Panics
    ///
    /// Panics if SDL2_ttf cannot be initialized, the font file cannot be loaded, or SDL2 cannot
    /// create or update an atlas texture.
    pub fn layout(
        &mut self,
        c: &Canvas<Window>,
        font_name: &str,
        font_size: u16,
        font_style: FontStyle,
        text: &str,
        width: u32,
    ) -> GlyphBlock {
        let key = FaceKey {
            path: font_name.to_string(),
            size: font_size,
            style: font_style.bits(),
        };

        self.ensure_face(&key);

        // Make sure every glyph exists first so measuring needs only the face.
        for ch in text.chars().filter(|ch| *ch != '\n') {
            self.ensure_glyph(c, &key, ch);
        }

        let face = &self.faces[&key];
        let advance_of = |s: &str| -> u32 {
            let mut x = 0;
            let mut previous = None;

            for ch in s.chars() {
                if let Some(p) = previous {
                    x += kerning(&face.font, p, ch);
                }

                x += face.glyphs[&ch].advance;
                previous = Some(ch);
            }

            x.max(0) as u32
        };

        let ranges = wrap_lines(text, Some(width), &mut |s| advance_of(s));
        let mut glyphs = Vec::new();
        let mut block_width = 0;

        for (row, range) in ranges.iter().enumerate() {
            let y = row as i32 * face.line_height;
            let mut x = 0;
            let mut previous = None;

            for ch in text[range.clone()].chars() {
                if let Some(p) = previous {
                    x += kerning(&face.font, p, ch);
                }

                let glyph = face.glyphs[&ch];

                if let Some(slot) = glyph.slot {
                    glyphs.push(PlacedGlyph {
                        page: slot.page,
                        source: slot.rect,
                        position: Point::new(x, y),
                    });
                }

                x += glyph.advance;
                previous = Some(ch);
            }

            block_width = block_width.max(x.max(0) as u32);
        }

        GlyphBlock {
            size: Size::new(block_width, face.line_height as u32 * ranges.len() as u32),
            glyphs,
        }
    }

    /// Draws a laid-out block by copying glyph rectangles from the atlas.
    ///
    /// Nothing is rasterized or uploaded here.  The color is applied with `set_color_mod` and
    /// `set_alpha_mod` on the atlas, so any color uses the same cached glyphs.
    ///
    /// # Arguments
    ///
    /// * `c` - The canvas to draw into (usually a widget's texture).
    /// * `block` - The block from [`FontCache::layout`] on this cache.
    /// * `origin` - Where the block's top-left corner goes on the canvas.
    /// * `color` - The text color.
    ///
    /// # Panics
    ///
    /// Panics if SDL2 fails to copy a glyph.
    pub fn draw(
        &mut self,
        c: &mut Canvas<Window>,
        block: &GlyphBlock,
        origin: Point,
        color: Color,
    ) {
        for page in &mut self.pages {
            page.texture.set_color_mod(color.r, color.g, color.b);
            page.texture.set_alpha_mod(color.a);
        }

        for glyph in &block.glyphs {
            let source = sdl2::rect::Rect::new(
                glyph.source.x,
                glyph.source.y,
                glyph.source.w,
                glyph.source.h,
            );
            let dest = sdl2::rect::Rect::new(
                origin.x + glyph.position.x,
                origin.y + glyph.position.y,
                glyph.source.w,
                glyph.source.h,
            );

            c.copy(&self.pages[glyph.page].texture, source, dest)
                .unwrap();
        }
    }
}
