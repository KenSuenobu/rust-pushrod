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

//! # Text layout
//!
//! A [`TextLayout`] turns a string, a [`TextStyle`] and a maximum width into a [`LaidOut`]: the
//! size of the text, where each line sits, the glyph runs to draw, and the caret and selection
//! geometry a text field needs.  A layout engine only *computes*; drawing stays with the painter,
//! which rasterizes each glyph once into the atlas texture and copies rectangles from it.
//!
//! Engines that ship with Pushrod:
//!
//! * `TtfLayout` (in the `pushrod-sdl2` crate) measures with SDL2_ttf.  It reports sizes,
//!   lines and caret geometry but no glyph runs; the SDL2 painter draws its text with SDL2_ttf.
//! * `ParleyLayout` (the `parley` Cargo feature) uses Parley for shaping, bidirectional text,
//!   line breaking and font fallback, and returns glyph runs.
//!
//! Applications can implement [`TextLayout`] for another engine (Cosmic Text, a bitmap font).
//! Wrap an engine in a [`LayoutCache`] so a widget lays its text out again only when the text,
//! font, or width changes.

use crate::font::Font;
use crate::geometry::{Point, Rect, Size};
use crate::paint::TextStyle;
use std::ops::Range;
use std::sync::Arc;

/// A font file shared by the glyph runs that use it.
#[derive(Clone)]
pub struct FontData {
    /// A process-unique identifier for the font file, usable as a cache key.
    pub id: u64,

    /// The index of the font in a font collection, or `0` for a single font.
    pub index: u32,

    /// The bytes of the font file, shared and never copied per run.
    pub bytes: Arc<dyn AsRef<[u8]> + Send + Sync>,
}

impl std::fmt::Debug for FontData {
    /// Formats the identifier and index, not the font bytes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// The formatter's result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontData")
            .field("id", &self.id)
            .field("index", &self.index)
            .finish()
    }
}

/// One glyph, positioned relative to the top-left corner of the laid-out text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionedGlyph {
    /// The glyph's identifier within its font.
    pub id: u32,

    /// Horizontal position of the glyph origin, in pixels.
    pub x: f32,

    /// Vertical position of the glyph origin (on the baseline), in pixels.
    pub y: f32,
}

/// A run of glyphs sharing one font and size.
#[derive(Debug, Clone)]
pub struct GlyphRun {
    /// The font the glyphs come from (the fallback font when the requested one lacks a glyph).
    pub font: FontData,

    /// The font size in pixels.
    pub font_size: f32,

    /// The glyphs, in visual order.
    pub glyphs: Vec<PositionedGlyph>,
}

/// One laid-out line of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The bytes of the source text on this line.
    pub text_range: Range<usize>,

    /// The line's bounds, relative to the top-left corner of the laid-out text.
    pub bounds: Rect,
}

/// Caret, selection and hit-test geometry for one laid-out text.
pub trait TextGeometry {
    /// The caret rectangle for a byte index into the text.
    ///
    /// # Arguments
    ///
    /// * `index` - A byte index; values past the end of the text give the caret at the end.
    ///
    /// # Returns
    ///
    /// A one-pixel-wide rectangle as tall as the line.
    fn caret_rect(&self, index: usize) -> Rect;

    /// The rectangles covering a selection, one or more per line.
    ///
    /// # Arguments
    ///
    /// * `range` - The selected bytes of the text.
    ///
    /// # Returns
    ///
    /// The rectangles to fill (several per line for mixed-direction text); empty for an empty range.
    fn selection_rects(&self, range: Range<usize>) -> Vec<Rect>;

    /// The byte index of the caret position nearest a point.
    ///
    /// # Arguments
    ///
    /// * `point` - A point relative to the top-left corner of the laid-out text.
    ///
    /// # Returns
    ///
    /// A byte index on a character boundary of the text.
    fn hit_test(&self, point: Point) -> usize;
}

/// The result of laying out a text.
pub struct LaidOut {
    /// The size of the text in pixels.
    pub size: Size,

    /// The width of the longest unbreakable word: the narrowest width the text can wrap to.
    pub min_width: u32,

    /// The lines, top to bottom.
    pub lines: Vec<Line>,

    /// The glyph runs to draw; empty for engines whose painter draws the text itself.
    pub runs: Vec<GlyphRun>,

    /// Caret, selection and hit-test geometry.
    pub geometry: Box<dyn TextGeometry>,
}

impl LaidOut {
    /// The size hint for layout managers.
    ///
    /// # Returns
    ///
    /// `(min, preferred)`: the narrowest width the text wraps to and the size it was laid out at.
    pub fn size_hint(&self) -> (u32, Size) {
        (self.min_width, self.size)
    }
}

/// A text layout engine.
pub trait TextLayout {
    /// Lays out a text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to lay out.
    /// * `style` - The font, size and style.
    /// * `max_width` - The width in pixels at which to wrap, or `None` for no wrapping.
    ///
    /// # Returns
    ///
    /// The size, lines, glyph runs and geometry of the text.
    fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<u32>) -> LaidOut;
}

/// The inputs a cached layout was computed from.
#[derive(Clone, PartialEq)]
struct LayoutKey {
    text: String,
    font: Font,
    max_width: Option<u32>,
}

/// Keeps the last layout of one widget's text and recomputes it only when an input changes.
#[derive(Default)]
pub struct LayoutCache {
    entry: Option<(LayoutKey, LaidOut)>,
    layouts: usize,
}

impl LayoutCache {
    /// Creates an empty cache.
    ///
    /// # Returns
    ///
    /// The new `LayoutCache`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the layout for the inputs, computing it with `engine` only if the text, font,
    /// font size, font style or width differ from the previous call.  Changing only the color
    /// or the justification does not lay out again.
    ///
    /// # Arguments
    ///
    /// * `engine` - The layout engine to use on a miss.
    /// * `text` - The text to lay out.
    /// * `style` - The font, size and style.
    /// * `max_width` - The width in pixels at which to wrap, or `None` for no wrapping.
    ///
    /// # Returns
    ///
    /// The cached or freshly computed layout.
    pub fn get(
        &mut self,
        engine: &mut dyn TextLayout,
        text: &str,
        style: &TextStyle,
        max_width: Option<u32>,
    ) -> &LaidOut {
        let key = LayoutKey {
            text: text.to_string(),
            font: style.font.clone(),
            max_width,
        };

        if !matches!(&self.entry, Some((k, _)) if *k == key) {
            self.entry = None;
        }

        let layouts = &mut self.layouts;
        let (_, laid) = self.entry.get_or_insert_with(|| {
            *layouts += 1;
            let laid = engine.layout(text, style, max_width);

            (key, laid)
        });

        laid
    }

    /// Forgets the cached layout so the next [`get`](Self::get) lays out again.
    pub fn invalidate(&mut self) {
        self.entry = None;
    }

    /// How many times the engine has been asked to lay out (cache misses).
    ///
    /// # Returns
    ///
    /// The number of layouts computed so far.
    pub fn layouts(&self) -> usize {
        self.layouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, FontStyle};
    use crate::paint::TextJustify;

    struct NullGeometry;

    impl TextGeometry for NullGeometry {
        /// A fixed caret.
        ///
        /// # Arguments
        ///
        /// * `_index` - Ignored.
        ///
        /// # Returns
        ///
        /// An empty rectangle.
        fn caret_rect(&self, _index: usize) -> Rect {
            Rect::default()
        }
        /// No selection.
        ///
        /// # Arguments
        ///
        /// * `_range` - Ignored.
        ///
        /// # Returns
        ///
        /// No rectangles.
        fn selection_rects(&self, _range: Range<usize>) -> Vec<Rect> {
            Vec::new()
        }
        /// Always the start.
        ///
        /// # Arguments
        ///
        /// * `_point` - Ignored.
        ///
        /// # Returns
        ///
        /// Zero.
        fn hit_test(&self, _point: Point) -> usize {
            0
        }
    }

    /// A fake engine: 10 pixels per character, one line.
    struct Fake;

    impl TextLayout for Fake {
        /// Ten pixels per character.
        ///
        /// # Arguments
        ///
        /// * `text` - The text to measure.
        /// * `_style` - Ignored.
        /// * `_max` - Ignored.
        ///
        /// # Returns
        ///
        /// A one-line layout.
        fn layout(&mut self, text: &str, _style: &TextStyle, _max: Option<u32>) -> LaidOut {
            let size = Size {
                w: text.chars().count() as u32 * 10,
                h: 20,
            };
            LaidOut {
                size,
                min_width: 10,
                lines: vec![Line {
                    text_range: 0..text.len(),
                    bounds: Rect {
                        x: 0,
                        y: 0,
                        w: size.w,
                        h: size.h,
                    },
                }],
                runs: Vec::new(),
                geometry: Box::new(NullGeometry),
            }
        }
    }

    fn style() -> TextStyle {
        TextStyle {
            font: Font::new("a", 12, FontStyle::NORMAL),
            font_color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            justification: TextJustify::Left,
        }
    }

    #[test]
    fn same_inputs_lay_out_once() {
        let (mut cache, mut engine) = (LayoutCache::new(), Fake);
        cache.get(&mut engine, "hello", &style(), Some(100));
        let laid = cache.get(&mut engine, "hello", &style(), Some(100));
        assert_eq!(laid.size.w, 50);
        assert_eq!(cache.layouts(), 1);
    }

    #[test]
    fn text_font_size_style_and_width_changes_recompute() {
        let (mut cache, mut engine) = (LayoutCache::new(), Fake);
        cache.get(&mut engine, "a", &style(), None);
        cache.get(&mut engine, "ab", &style(), None);
        let mut s = style();
        s.font = Font::new("b", 12, FontStyle::NORMAL);
        cache.get(&mut engine, "ab", &s, None);
        s.font = Font::new("b", 14, FontStyle::NORMAL);
        cache.get(&mut engine, "ab", &s, None);
        s.font = Font::new("b", 14, FontStyle::BOLD);
        cache.get(&mut engine, "ab", &s, None);
        cache.get(&mut engine, "ab", &s, Some(50));
        assert_eq!(cache.layouts(), 6);
    }

    #[test]
    fn color_and_justification_do_not_recompute() {
        let (mut cache, mut engine) = (LayoutCache::new(), Fake);
        cache.get(&mut engine, "a", &style(), None);
        let mut s = style();
        s.font_color = Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        s.justification = TextJustify::Center;
        cache.get(&mut engine, "a", &s, None);
        assert_eq!(cache.layouts(), 1);
    }

    #[test]
    fn invalidate_forces_a_new_layout() {
        let (mut cache, mut engine) = (LayoutCache::new(), Fake);
        cache.get(&mut engine, "a", &style(), None);
        cache.invalidate();
        cache.get(&mut engine, "a", &style(), None);
        assert_eq!(cache.layouts(), 2);
    }

    #[test]
    fn size_hint_reports_min_and_preferred() {
        let mut engine = Fake;
        let laid = engine.layout("abc", &style(), None);
        assert_eq!(laid.size_hint(), (10, Size { w: 30, h: 20 }));
    }
}
