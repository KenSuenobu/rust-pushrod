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

//! The default [`TextLayout`]: measures with SDL2_ttf.
//!
//! Lines are wrapped greedily at spaces, and the caret positions are measured from the width of
//! each prefix of the line.  Text is laid out left to right with no shaping or font fallback (use
//! the `parley` feature for those), and no glyph runs are produced because the SDL2 painter draws
//! the text with SDL2_ttf.

use crate::font::load_ttf;
use crate::painter::to_sdl_font_style;
use pushrod::error::Error;
use pushrod::font::FontRegistry;
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::TextStyle;
use pushrod::text::layout::{LaidOut, Line, TextGeometry, TextLayout};
use sdl2::ttf::Font;
use std::collections::HashMap;
use std::ops::Range;

/// Splits `text` into lines no wider than `max_width`, breaking at spaces and at newlines.
///
/// A word wider than `max_width` stays whole on a line of its own.
///
/// # Arguments
///
/// * `text` - The text to wrap.
/// * `max_width` - The width in pixels at which to wrap, or `None` to break only at newlines.
/// * `measure` - Returns the width in pixels of a piece of text.
///
/// # Returns
///
/// The byte range of each line.  Ranges exclude the newline that ended the line and trailing
/// spaces at a wrap point, and the ranges are in order.
pub fn wrap_lines(
    text: &str,
    max_width: Option<u32>,
    measure: &mut dyn FnMut(&str) -> u32,
) -> Vec<Range<usize>> {
    let mut lines = Vec::new();
    let mut offset = 0;

    for paragraph in text.split('\n') {
        let end_of_paragraph = offset + paragraph.len();
        let mut start = offset;
        let mut line_end = offset;
        let mut word_start = offset;

        for (i, ch) in paragraph.char_indices().chain([(paragraph.len(), ' ')]) {
            if ch != ' ' {
                continue;
            }

            let word_end = offset + i;
            let candidate = &text[start..word_end];

            if max_width.is_some_and(|m| measure(candidate) > m) && word_start > start {
                lines.push(start..line_end);
                start = word_start;
            }

            line_end = word_end;
            word_start = word_end + 1;
        }

        lines.push(start..end_of_paragraph.max(start));
        offset = end_of_paragraph + 1;
    }

    lines
}

/// The caret x positions of one line.
struct LineCarets {
    /// The bytes of the text on the line.
    range: Range<usize>,
    /// The top of the line in pixels.
    y: i32,
    /// `(byte index, x)` for each character boundary in the line, including its end.
    carets: Vec<(usize, i32)>,
}

/// Caret, selection and hit-testing for left-to-right lines of equal height.
struct LtrGeometry {
    lines: Vec<LineCarets>,
    line_height: u32,
}

impl LtrGeometry {
    /// The line containing a byte index (the last line for indexes past the end).
    ///
    /// Returns `None` only for a layout with no lines.
    fn line_of(&self, index: usize) -> Option<&LineCarets> {
        self.lines
            .iter()
            .find(|l| index <= l.range.end)
            .or_else(|| self.lines.last())
    }

    /// The x of a byte index on a line.
    fn x_of(line: &LineCarets, index: usize) -> i32 {
        let index = index.clamp(line.range.start, line.range.end);
        line.carets
            .iter()
            .rev()
            .find(|(i, _)| *i <= index)
            .map_or(0, |(_, x)| *x)
    }
}

impl TextGeometry for LtrGeometry {
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
        match self.line_of(index) {
            Some(line) => Rect {
                x: Self::x_of(line, index),
                y: line.y,
                w: 1,
                h: self.line_height,
            },
            None => Rect {
                x: 0,
                y: 0,
                w: 1,
                h: self.line_height,
            },
        }
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
        let mut rects = Vec::new();

        for line in &self.lines {
            let start = range.start.max(line.range.start);
            let end = range.end.min(line.range.end);

            if start < end {
                let x0 = Self::x_of(line, start);
                let x1 = Self::x_of(line, end);
                rects.push(Rect {
                    x: x0,
                    y: line.y,
                    w: (x1 - x0).max(0) as u32,
                    h: self.line_height,
                });
            }
        }

        rects
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
        let row = (point.y.max(0) as u32 / self.line_height.max(1)) as usize;
        let Some(line) = self.lines.get(row).or_else(|| self.lines.last()) else {
            return 0;
        };

        line.carets
            .iter()
            .min_by_key(|(_, x)| (x - point.x).abs())
            .map_or(line.range.start, |(i, _)| *i)
    }
}

/// A [`TextLayout`] that measures with SDL2_ttf.  It keeps each font it loads for reuse.
#[derive(Default)]
pub struct TtfLayout {
    fonts: HashMap<(String, u16), Font<'static, 'static>>,
    errors: Vec<Error>,
    registry: FontRegistry,
}

impl TtfLayout {
    /// Creates an engine with no fonts loaded.
    ///
    /// # Returns
    ///
    /// The new `TtfLayout`.
    pub fn new() -> Self {
        Self::default()
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

    /// Removes and returns the errors [`TextLayout::layout`] recorded since the last call.
    ///
    /// `layout` cannot fail (its trait has no error), so when a font will not load it records the
    /// error here and returns an empty layout.  Call [`try_layout`](TtfLayout::try_layout)
    /// instead to get the error directly.
    ///
    /// # Returns
    ///
    /// The errors, in order; empty if every layout succeeded.
    pub fn take_errors(&mut self) -> Vec<Error> {
        std::mem::take(&mut self.errors)
    }

    /// Lays out `text`, loading the font on first use.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to lay out.
    /// * `style` - The font, size and style.
    /// * `max_width` - The width in pixels at which to wrap, or `None` for no wrapping.
    ///
    /// # Returns
    ///
    /// The size, lines and geometry of the text; there are no glyph runs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Font`] if the font's family is not registered or its file or data cannot
    /// be loaded.
    pub fn try_layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: Option<u32>,
    ) -> Result<LaidOut, Error> {
        let family = self.registry.resolve_family(&style.font);
        let key = (family.clone(), style.font.size());

        if !self.fonts.contains_key(&key) {
            let source = self
                .registry
                .source(&style.font)
                .ok_or_else(|| Error::font(&family, "the family is not registered"))?;
            let font = load_ttf(&source, style.font.size())
                .map_err(|message| Error::font(&family, message))?;

            self.fonts.insert(key.clone(), font);
        }

        let font = self
            .fonts
            .get_mut(&key)
            .ok_or_else(|| Error::font(&family, "the font is not loaded"))?;

        font.set_style(to_sdl_font_style(style.font.style()));

        let line_height = font.height().max(1) as u32;

        Ok(lay_out(text, max_width, line_height, &mut |s: &str| {
            font.size_of(s).map_or(0, |(w, _)| w)
        }))
    }
}

impl TextLayout for TtfLayout {
    /// Lays out `text`, loading the font on first use.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to lay out.
    /// * `style` - The font, size and style.
    /// * `max_width` - The width in pixels at which to wrap, or `None` for no wrapping.
    ///
    /// # Returns
    ///
    /// The size, lines and geometry of the text; there are no glyph runs.
    ///
    /// A font that cannot be loaded gives an empty layout, and the error is kept for
    /// [`TtfLayout::take_errors`].
    fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<u32>) -> LaidOut {
        match self.try_layout(text, style, max_width) {
            Ok(laid) => laid,
            Err(error) => {
                self.errors.push(error);

                lay_out(text, max_width, 1, &mut |_: &str| 0)
            }
        }
    }
}

/// Lays out text from a width measurer, so a missing font can fall back to zero widths.
///
/// # Arguments
///
/// * `text` - The text to lay out.
/// * `max_width` - The width in pixels at which to wrap, or `None` for no wrapping.
/// * `line_height` - The height of one line in pixels; at least 1.
/// * `width_of` - Returns the width in pixels of a piece of text.
///
/// # Returns
///
/// The size, lines and geometry of the text; there are no glyph runs.
fn lay_out(
    text: &str,
    max_width: Option<u32>,
    line_height: u32,
    width_of: &mut dyn FnMut(&str) -> u32,
) -> LaidOut {
    let ranges = wrap_lines(text, max_width, &mut *width_of);

    let mut lines = Vec::new();
    let mut carets_by_line = Vec::new();
    let mut width = 0;

    for (row, range) in ranges.iter().enumerate() {
        let slice = &text[range.clone()];
        let mut carets: Vec<(usize, i32)> = slice
            .char_indices()
            .map(|(i, _)| (range.start + i, width_of(&slice[..i]) as i32))
            .collect();
        let line_width = width_of(slice);
        carets.push((range.end, line_width as i32));

        width = width.max(line_width);
        let y = row as i32 * line_height as i32;
        lines.push(Line {
            text_range: range.clone(),
            bounds: Rect {
                x: 0,
                y,
                w: line_width,
                h: line_height,
            },
        });
        carets_by_line.push(LineCarets {
            range: range.clone(),
            y,
            carets,
        });
    }

    let min_width = text
        .split_whitespace()
        .map(&mut *width_of)
        .max()
        .unwrap_or(0);

    LaidOut {
        size: Size {
            w: width,
            h: line_height * lines.len() as u32,
        },
        min_width,
        lines,
        runs: Vec::new(),
        geometry: Box::new(LtrGeometry {
            lines: carets_by_line,
            line_height,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 10 pixels per character.
    fn ten(s: &str) -> u32 {
        s.chars().count() as u32 * 10
    }

    fn slices<'a>(text: &'a str, lines: &[Range<usize>]) -> Vec<&'a str> {
        lines.iter().map(|r| &text[r.clone()]).collect()
    }

    #[test]
    fn no_width_breaks_only_at_newlines() {
        let t = "one two\nthree";
        assert_eq!(
            slices(t, &wrap_lines(t, None, &mut ten)),
            ["one two", "three"]
        );
    }

    #[test]
    fn wraps_at_spaces() {
        let t = "aaa bbb ccc";
        assert_eq!(
            slices(t, &wrap_lines(t, Some(70), &mut ten)),
            ["aaa bbb", "ccc"]
        );
    }

    #[test]
    fn long_word_stays_whole() {
        let t = "abcdefghij kk";
        assert_eq!(
            slices(t, &wrap_lines(t, Some(30), &mut ten)),
            ["abcdefghij", "kk"]
        );
    }

    #[test]
    fn empty_text_is_one_empty_line() {
        assert_eq!(wrap_lines("", Some(50), &mut ten), vec![0..0]);
        assert_eq!(wrap_lines("a\n", None, &mut ten), vec![0..1, 2..2]);
    }

    #[test]
    fn a_missing_font_is_an_error_and_layout_falls_back_to_empty() {
        use pushrod::color::{Color, FontStyle};
        use pushrod::font::Font;
        use pushrod::paint::TextJustify;

        let style = TextStyle {
            font: Font::new("Missing", 16, FontStyle::NORMAL),
            font_color: Color::BLACK,
            justification: TextJustify::Left,
        };
        let mut engine = TtfLayout::new();

        assert!(matches!(
            engine.try_layout("hello", &style, None),
            Err(Error::Font { .. })
        ));

        let laid = engine.layout("hello", &style, None);

        assert_eq!(laid.size.w, 0);
        assert_eq!(laid.lines.len(), 1);
        assert_eq!(engine.take_errors().len(), 1);
        assert!(engine.take_errors().is_empty());
    }

    #[test]
    fn measures_with_a_real_font() {
        use pushrod::color::{Color, FontStyle};
        use pushrod::font::Font;
        use pushrod::paint::TextJustify;

        let style = TextStyle {
            font: Font::new("", 16, FontStyle::NORMAL),
            font_color: Color::BLACK,
            justification: TextJustify::Left,
        };
        let mut engine = TtfLayout::new();
        let wide = engine.layout("alpha beta gamma", &style, None);
        let narrow = engine.layout("alpha beta gamma", &style, Some(wide.size.w / 2));

        assert_eq!(wide.lines.len(), 1);
        assert!(narrow.lines.len() > 1);
        assert_eq!(narrow.size.h, wide.size.h * narrow.lines.len() as u32);
        assert!(wide.min_width > 0 && wide.min_width < wide.size.w);
        assert!(wide.runs.is_empty());
        assert!(wide.geometry.caret_rect(5).x < wide.geometry.caret_rect(10).x);
    }

    #[test]
    fn geometry_maps_carets_selection_and_points() {
        let g = LtrGeometry {
            lines: vec![LineCarets {
                range: 0..3,
                y: 0,
                carets: vec![(0, 0), (1, 10), (2, 20), (3, 30)],
            }],
            line_height: 16,
        };
        assert_eq!(
            g.caret_rect(2),
            Rect {
                x: 20,
                y: 0,
                w: 1,
                h: 16
            }
        );
        assert_eq!(g.caret_rect(99).x, 30);
        assert_eq!(
            g.selection_rects(1..3),
            vec![Rect {
                x: 10,
                y: 0,
                w: 20,
                h: 16
            }]
        );
        assert!(g.selection_rects(2..2).is_empty());
        assert_eq!(g.hit_test(Point { x: 14, y: 5 }), 1);
        assert_eq!(g.hit_test(Point { x: 400, y: 500 }), 3);
    }
}
