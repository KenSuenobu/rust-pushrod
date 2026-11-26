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

//! The Parley [`TextLayout`]: shaping, bidirectional text, line breaking and font fallback.
//!
//! Parley only computes positions.  The glyph runs it produces are drawn by rasterizing each
//! glyph once (see [`raster`](super::raster)) into the atlas texture and copying rectangles.
//! Fonts named in a [`TextStyle`] are loaded from their files once and registered with Parley;
//! glyphs the font lacks (CJK, emoji, Arabic) come from the system's fallback fonts.

use super::layout::{FontData, GlyphRun, LaidOut, Line, PositionedGlyph, TextGeometry, TextLayout};
use crate::color::FontStyle;
use crate::font::{Font, FontRegistry, FontSource};
use crate::geometry::{Point, Rect, Size};
use crate::paint::TextStyle;
use parley::fontique::Blob;
use parley::layout::{Affinity, Cursor, Layout, PositionedLayoutItem, Selection};
use parley::style::{FontFamily, FontFamilyName, FontStyle as ParleyFontStyle, FontWeight};
use parley::{BoundingBox, FontContext, GenericFamily, LayoutContext, StyleProperty};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

/// Converts a Parley bounding box to a pixel rectangle.
///
/// # Arguments
///
/// * `b` - The bounding box.
/// * `min_width` - The least width to give the rectangle (a caret is one pixel wide).
///
/// # Returns
///
/// The rectangle, rounded outward.
fn to_rect(b: BoundingBox, min_width: u32) -> Rect {
    let (x0, y0) = (b.x0.floor() as i32, b.y0.floor() as i32);
    let (x1, y1) = (b.x1.ceil() as i32, b.y1.ceil() as i32);
    Rect {
        x: x0,
        y: y0,
        w: ((x1 - x0).max(0) as u32).max(min_width),
        h: (y1 - y0).max(0) as u32,
    }
}

/// Caret, selection and hit-testing backed by a Parley layout.
struct ParleyGeometry {
    layout: Layout<()>,
}

impl TextGeometry for ParleyGeometry {
    /// The caret rectangle for a byte index; see [`TextGeometry::caret_rect`].
    ///
    /// # Arguments
    ///
    /// * `index` - A byte index into the text.
    ///
    /// # Returns
    ///
    /// A one-pixel-wide rectangle as tall as the line.
    fn caret_rect(&self, index: usize) -> Rect {
        to_rect(
            Cursor::from_byte_index(&self.layout, index, Affinity::Downstream)
                .geometry(&self.layout, 1.0),
            1,
        )
    }

    /// The rectangles covering a selection; see [`TextGeometry::selection_rects`].
    ///
    /// # Arguments
    ///
    /// * `range` - The selected bytes of the text.
    ///
    /// # Returns
    ///
    /// The rectangles to fill, empty for an empty range.
    fn selection_rects(&self, range: Range<usize>) -> Vec<Rect> {
        let anchor = Cursor::from_byte_index(&self.layout, range.start, Affinity::Downstream);
        let focus = Cursor::from_byte_index(&self.layout, range.end, Affinity::Upstream);

        Selection::new(anchor, focus)
            .geometry(&self.layout)
            .into_iter()
            .map(|(b, _)| to_rect(b, 0))
            .collect()
    }

    /// The byte index nearest a point; see [`TextGeometry::hit_test`].
    ///
    /// # Arguments
    ///
    /// * `point` - A point relative to the top-left corner of the text.
    ///
    /// # Returns
    ///
    /// A byte index on a character boundary.
    fn hit_test(&self, point: Point) -> usize {
        Cursor::from_point(&self.layout, point.x as f32, point.y as f32).index()
    }
}

/// A [`TextLayout`] that lays text out with Parley.
pub struct ParleyLayout {
    fonts: FontContext,
    context: LayoutContext<()>,
    families: HashMap<String, Option<String>>,
    registry: FontRegistry,
}

impl std::fmt::Debug for ParleyLayout {
    /// Shows the resolved font families; Parley's contexts have no `Debug`.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatter's result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParleyLayout")
            .field("families", &self.families)
            .finish_non_exhaustive()
    }
}

impl ParleyLayout {
    /// Creates an engine that discovers the system's fonts for fallback.
    ///
    /// # Returns
    ///
    /// The new `ParleyLayout`.
    pub fn new() -> Self {
        Self {
            fonts: FontContext::new(),
            context: LayoutContext::new(),
            families: HashMap::new(),
            registry: FontRegistry::new(),
        }
    }

    /// Resolves fonts with an application's registry instead of the built-in one.
    ///
    /// # Arguments
    ///
    /// * `registry` - The registry, usually `engine.fonts().clone()`.
    ///
    /// # Returns
    ///
    /// This engine, using the registry.
    #[must_use]
    pub fn with_fonts(mut self, registry: FontRegistry) -> Self {
        self.registry = registry;
        self
    }

    /// Registers a font's data with Parley, once per registered family.
    ///
    /// # Arguments
    ///
    /// * `font` - The font to resolve through the registry.
    ///
    /// # Returns
    ///
    /// The family name to request, or `None` if the family is not registered or its file cannot
    /// be read (the generic sans-serif family is used instead).
    fn family_for(&mut self, font: &Font) -> Option<String> {
        let key = self.registry.resolve_family(font);

        if let Some(known) = self.families.get(&key) {
            return known.clone();
        }

        let family = self
            .registry
            .source(font)
            .and_then(|source| match source {
                FontSource::Path(path) => std::fs::read(path).ok(),
                FontSource::Bytes(bytes) => Some(bytes.to_vec()),
            })
            .and_then(|bytes| {
                let (id, _) = self
                    .fonts
                    .collection
                    .register_fonts(Blob::new(Arc::new(bytes)), None)
                    .into_iter()
                    .next()?;
                self.fonts.collection.family_name(id).map(str::to_string)
            });

        self.families.insert(key, family.clone());
        family
    }
}

impl Default for ParleyLayout {
    /// Creates an engine; the same as [`ParleyLayout::new`].
    ///
    /// # Returns
    ///
    /// The new `ParleyLayout`.
    fn default() -> Self {
        Self::new()
    }
}

impl TextLayout for ParleyLayout {
    /// Lays out `text`; see [`TextLayout::layout`].
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
    fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<u32>) -> LaidOut {
        let mut families = Vec::new();

        if let Some(name) = self.family_for(&style.font) {
            families.push(FontFamilyName::Named(name.into()));
        }
        families.push(FontFamilyName::Generic(GenericFamily::SansSerif));

        let mut builder = self
            .context
            .ranged_builder(&mut self.fonts, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(f32::from(style.font.size())));
        builder.push_default(StyleProperty::FontFamily(FontFamily::List(families.into())));

        if style.font.style().contains(FontStyle::BOLD) {
            builder.push_default(StyleProperty::FontWeight(FontWeight::BOLD));
        }
        if style.font.style().contains(FontStyle::ITALIC) {
            builder.push_default(StyleProperty::FontStyle(ParleyFontStyle::Italic));
        }

        let mut layout: Layout<()> = builder.build(text);
        let min_width = layout.calculate_content_widths().min.ceil() as u32;
        layout.break_all_lines(max_width.map(|w| w as f32));

        let mut lines = Vec::new();
        let mut runs = Vec::new();

        for line in layout.lines() {
            let m = line.metrics();
            lines.push(Line {
                text_range: line.text_range(),
                bounds: to_rect(
                    BoundingBox::new(
                        f64::from(m.inline_min_coord),
                        f64::from(m.block_min_coord),
                        f64::from(m.inline_max_coord),
                        f64::from(m.block_max_coord),
                    ),
                    0,
                ),
            });

            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    let font = run.run().font();
                    let (bytes, id) = font.data.clone().into_raw_parts();

                    runs.push(GlyphRun {
                        font: FontData {
                            id,
                            index: font.index,
                            bytes,
                        },
                        font_size: run.run().font_size(),
                        glyphs: run
                            .positioned_glyphs()
                            .map(|g| PositionedGlyph {
                                id: g.id,
                                x: g.x,
                                y: g.y,
                            })
                            .collect(),
                    });
                }
            }
        }

        LaidOut {
            size: Size {
                w: layout.width().ceil() as u32,
                h: layout.height().ceil() as u32,
            },
            min_width,
            lines,
            runs,
            geometry: Box::new(ParleyGeometry { layout }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::paint::TextJustify;

    fn style() -> TextStyle {
        TextStyle {
            font: Font::new("", 16, FontStyle::NORMAL),
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
    fn lays_out_latin_text_with_glyph_runs() {
        let laid = ParleyLayout::new().layout("Hello", &style(), None);
        assert!(laid.size.w > 0 && laid.size.h > 0);
        assert_eq!(laid.lines.len(), 1);
        assert_eq!(laid.runs.iter().map(|r| r.glyphs.len()).sum::<usize>(), 5);
    }

    #[test]
    fn wraps_to_the_width_and_reports_a_min_width() {
        let mut engine = ParleyLayout::new();
        let text = "alpha beta gamma delta epsilon";
        let wide = engine.layout(text, &style(), None);
        let narrow = engine.layout(text, &style(), Some(wide.size.w / 3));
        assert!(narrow.lines.len() > 1);
        assert!(narrow.size.w <= wide.size.w / 3 + 1);
        assert!(wide.min_width > 0 && wide.min_width < wide.size.w);
    }

    #[test]
    fn caret_moves_right_through_ltr_text_and_hit_test_inverts_it() {
        let laid = ParleyLayout::new().layout("abc", &style(), None);
        let xs: Vec<i32> = (0..=3).map(|i| laid.geometry.caret_rect(i).x).collect();
        assert!(xs.windows(2).all(|w| w[0] < w[1]), "{xs:?}");
        assert_eq!(laid.geometry.hit_test(Point { x: xs[2], y: 2 }), 2);
    }

    #[test]
    fn selection_covers_the_selected_characters() {
        let laid = ParleyLayout::new().layout("abcdef", &style(), None);
        let rects = laid.geometry.selection_rects(1..4);
        assert!(!rects.is_empty());
        let (x0, x1) = (laid.geometry.caret_rect(1).x, laid.geometry.caret_rect(4).x);
        assert!((rects[0].x - x0).abs() <= 1);
        assert!((rects[0].x + rects[0].w as i32 - x1).abs() <= 1);
        assert!(laid.geometry.selection_rects(2..2).is_empty());
    }

    #[test]
    fn right_to_left_text_runs_right_to_left() {
        let laid = ParleyLayout::new().layout("שלום", &style(), None);
        let (first, last) = (
            laid.geometry.caret_rect(0).x,
            laid.geometry.caret_rect("שלום".len()).x,
        );
        assert!(
            first > last,
            "caret at the start of RTL text is right of the end: {first} {last}"
        );
    }

    #[test]
    fn cjk_and_emoji_produce_glyphs_from_fallback_fonts() {
        let mut engine = ParleyLayout::new();
        for text in ["日本語", "😀"] {
            let laid = engine.layout(text, &style(), None);
            assert!(laid.size.w > 0, "{text}");
            assert!(laid.runs.iter().any(|r| !r.glyphs.is_empty()), "{text}");
        }
    }
}
