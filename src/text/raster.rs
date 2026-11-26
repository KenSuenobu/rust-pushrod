// Text Layout
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

//! Rasterizes glyphs on the CPU, once each, for the GPU glyph atlas.
//!
//! A [`GlyphCache`] turns `(font, glyph, size)` into a [`GlyphBitmap`] with swash the first time
//! it is asked and returns the stored bitmap afterwards, so after warm-up drawing text is
//! texture copies only.  [`ShelfPacker`] places the bitmaps into an atlas texture.

use super::layout::FontData;
use std::collections::HashMap;
use swash::FontRef;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::Format;

/// One rasterized glyph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphBitmap {
    /// Pixels from the glyph origin to the left edge of the bitmap.
    pub left: i32,

    /// Pixels from the baseline up to the top edge of the bitmap.
    pub top: i32,

    /// Bitmap width in pixels.
    pub width: u32,

    /// Bitmap height in pixels.
    pub height: u32,

    /// `true` for a color glyph (emoji): `data` is RGBA and is drawn without color modulation.
    /// `false` for a coverage mask: `data` is one alpha byte per pixel, drawn tinted.
    pub color: bool,

    /// The pixels, row by row.
    pub data: Vec<u8>,
}

/// Rasterizes glyphs with swash and remembers every bitmap.
#[derive(Default)]
pub struct GlyphCache {
    context: ScaleContext,
    bitmaps: HashMap<(u64, u32, u32, u32), Option<GlyphBitmap>>,
    rasterized: usize,
}

impl std::fmt::Debug for GlyphCache {
    /// Shows the cache's counts; swash's `ScaleContext` has no `Debug`.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatter's result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphCache")
            .field("bitmaps", &self.bitmaps.len())
            .field("rasterized", &self.rasterized)
            .finish_non_exhaustive()
    }
}

impl GlyphCache {
    /// Creates an empty cache.
    ///
    /// # Returns
    ///
    /// The new `GlyphCache`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the bitmap of a glyph, rasterizing it only the first time.
    ///
    /// # Arguments
    ///
    /// * `font` - The font the glyph belongs to.
    /// * `glyph` - The glyph identifier within the font.
    /// * `size` - The font size in pixels.
    ///
    /// # Returns
    ///
    /// The bitmap, or `None` for a glyph with no pixels (a space) or a font swash cannot read.
    pub fn get(&mut self, font: &FontData, glyph: u32, size: f32) -> Option<&GlyphBitmap> {
        let key = (font.id, font.index, glyph, size.to_bits());

        if !self.bitmaps.contains_key(&key) {
            self.rasterized += 1;
            let bitmap = Self::render(&mut self.context, font, glyph, size);
            self.bitmaps.insert(key, bitmap);
        }

        self.bitmaps[&key].as_ref()
    }

    /// How many glyphs have been rasterized.  It stops growing once the text on screen has been
    /// drawn once.
    ///
    /// # Returns
    ///
    /// The number of rasterizations so far.
    pub fn rasterized(&self) -> usize {
        self.rasterized
    }

    /// Rasterizes one glyph.
    fn render(
        context: &mut ScaleContext,
        font: &FontData,
        glyph: u32,
        size: f32,
    ) -> Option<GlyphBitmap> {
        let font_ref = FontRef::from_index(font.bytes.as_ref().as_ref(), font.index as usize)?;
        let mut scaler = context.builder(font_ref).size(size).hint(true).build();
        let image = Render::new(&[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ])
        .format(Format::Alpha)
        .render(&mut scaler, glyph as u16)?;

        if image.placement.width == 0 || image.placement.height == 0 {
            return None;
        }

        Some(GlyphBitmap {
            left: image.placement.left,
            top: image.placement.top,
            width: image.placement.width,
            height: image.placement.height,
            color: matches!(image.content, Content::Color),
            data: image.data,
        })
    }
}

/// Where a bitmap sits in the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasSlot {
    /// Left edge in the atlas, in pixels.
    pub x: u32,

    /// Top edge in the atlas, in pixels.
    pub y: u32,
}

/// Packs rectangles into an atlas of a fixed size, in rows ("shelves").
#[derive(Debug)]
pub struct ShelfPacker {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    row_height: u32,
}

impl ShelfPacker {
    /// Creates a packer for an empty atlas.
    ///
    /// # Arguments
    ///
    /// * `width` - Atlas width in pixels.
    /// * `height` - Atlas height in pixels.
    ///
    /// # Returns
    ///
    /// The new `ShelfPacker`.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            x: 0,
            y: 0,
            row_height: 0,
        }
    }

    /// Reserves space for a rectangle, with a one pixel gap so scaled copies don't bleed.
    ///
    /// # Arguments
    ///
    /// * `w` - Rectangle width in pixels.
    /// * `h` - Rectangle height in pixels.
    ///
    /// # Returns
    ///
    /// The slot, or `None` if the atlas is full or the rectangle is larger than the atlas.
    pub fn pack(&mut self, w: u32, h: u32) -> Option<AtlasSlot> {
        let (pw, ph) = (w + 1, h + 1);

        if pw > self.width || ph > self.height {
            return None;
        }

        if self.x + pw > self.width {
            self.y += self.row_height;
            self.x = 0;
            self.row_height = 0;
        }

        if self.y + ph > self.height {
            return None;
        }

        let slot = AtlasSlot {
            x: self.x,
            y: self.y,
        };
        self.x += pw;
        self.row_height = self.row_height.max(ph);
        Some(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn open_sans() -> FontData {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/OpenSans-Regular.ttf"
        ))
        .expect("the bundled font exists");
        FontData {
            id: 1,
            index: 0,
            bytes: Arc::new(bytes),
        }
    }

    #[test]
    fn a_glyph_is_rasterized_once() {
        let (mut cache, font) = (GlyphCache::new(), open_sans());
        let first = cache.get(&font, 36, 16.0).cloned().expect("'A' has pixels");
        assert!(first.width > 0 && first.height > 0 && !first.color);
        assert_eq!(first.data.len() as u32, first.width * first.height);

        for _ in 0..10 {
            assert_eq!(cache.get(&font, 36, 16.0), Some(&first));
        }
        assert_eq!(cache.rasterized(), 1);

        cache.get(&font, 36, 24.0);
        assert_eq!(cache.rasterized(), 2);
    }

    #[test]
    fn packer_fills_rows_then_reports_full() {
        let mut p = ShelfPacker::new(20, 10);
        assert_eq!(p.pack(7, 4), Some(AtlasSlot { x: 0, y: 0 }));
        assert_eq!(p.pack(7, 4), Some(AtlasSlot { x: 8, y: 0 }));
        assert_eq!(p.pack(7, 4), Some(AtlasSlot { x: 0, y: 5 }));
        assert_eq!(p.pack(30, 4), None);
        assert_eq!(p.pack(19, 9), None);
    }
}
