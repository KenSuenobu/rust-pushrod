// Paint
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

//! # Paint
//!
//! The backend-neutral drawing API.  A [`Widget`](crate::widget::Widget) draws by calling the
//! [`Painter`] it is handed, never an `SDL2` type, so widgets (and widget libraries) work with any
//! backend.  Reusable building blocks compose a `Widget`'s `paint`, all drawing into the `Widget`'s
//! own surface:
//!
//! ```text
//!   RootWidget   = fill
//!   Panel   = fill
//!   Frame    = fill + border
//!   Text   = fill + text
//!   Button = fill + text + border   (+ pressed state)
//! ```
//!
//! The `pushrod-sdl2` crate's `Sdl2Painter` is the default backend: each `Widget` paints
//! into its own GPU texture only when invalidated, and frames are blits of cached textures.  A
//! backend declares its [`CachePolicy`].  The [`RecordingPainter`](crate::backend::headless::RecordingPainter)
//! records calls for unit tests, without a window.

use crate::color::Color;
use crate::error::Error;
use crate::font::Font;
use crate::geometry::{Point, Rect, Size};
use crate::widget::VisualState;
use unicode_segmentation::UnicodeSegmentation;

/// Horizontal alignment of text within its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextJustify {
    /// Aligned to the left edge.
    Left,
    /// Centered horizontally.
    Center,
    /// Aligned to the right edge.
    Right,
}

/// What [`text`] does with text that is wider than its rectangle.
///
/// The text is shortened on a grapheme boundary (never inside a character) and an ellipsis (`…`),
/// measured in the same font, marks the cut.  Truncation applies to single-line text; text that
/// contains a newline is left alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Truncate {
    /// Don't shorten the text; it is clipped (or wrapped) as before.
    None,
    /// Keep the start and cut the end: `Quarterly rep…`.
    #[default]
    End,
    /// Keep the end and cut the start: `…report.txt`.
    Start,
    /// Keep both ends and cut the middle: `Quart….txt`, for paths and file names.
    Middle,
}

/// How [`paragraph`] breaks text into lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Wrap {
    /// One line; [`text`] draws it (see [`Truncate`]).
    #[default]
    None,
    /// Break at spaces; a word wider than the width is broken between graphemes.
    Word,
    /// Break between any two graphemes, as late as possible.
    Anywhere,
}

/// The ellipsis drawn where text is cut.
pub const ELLIPSIS: &str = "\u{2026}";

/// The result of [`text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextDrawn {
    /// The size of the text drawn; `0` by `0` if nothing was drawn.
    pub size: Size,
    /// Whether the text was shortened to fit (see [`Truncate`]), so a tooltip can show it in full.
    pub truncated: bool,
}

/// How a backend keeps painted `Widget`s between frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum CachePolicy {
    /// Each `Widget` paints into its own cached surface (a GPU texture) when invalidated, and frames
    /// are blits of those surfaces.  Used by GPU backends such as `SDL2`.
    #[default]
    TextureCache,

    /// Invalidated `Widget`s are painted straight to the screen with clipping, with no per-widget
    /// surface.  For memory-poor systems.  Declared by backends that need it; the `SDL2` backend
    /// never uses it.
    DirectDraw,
}

/// The font and alignment used to draw text with [`text`].
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// The font: a registered family, size, and style (see [`crate::font`]).
    pub font: Font,

    /// Text color.
    pub font_color: Color,

    /// Horizontal justification.
    pub justification: TextJustify,
}

impl TextStyle {
    /// Creates a `TextStyle`.
    ///
    /// # Arguments
    ///
    /// * `font` - The font handle.
    /// * `font_color` - The text color.
    /// * `justification` - The horizontal alignment.
    ///
    /// # Returns
    ///
    /// The new `TextStyle`.
    pub fn new(font: Font, font_color: Color, justification: TextJustify) -> Self {
        Self {
            font,
            font_color,
            justification,
        }
    }
}

/// The drawing operations a `Widget`'s `paint` can use.  Kept small: only what the helpers in this
/// module need.
///
/// Coordinates are relative to the `Widget`'s own upper left-hand corner.  A backend decides where
/// the pixels go (a GPU texture, a memory buffer, the screen with clipping).
pub trait Painter {
    /// Returns how this backend caches painted `Widget`s.
    ///
    /// # Returns
    ///
    /// The backend's [`CachePolicy`].
    fn cache_policy(&self) -> CachePolicy;

    /// Fills the whole drawing surface with a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The fill color.
    fn fill(&mut self, color: Color);

    /// Fills a rectangle with a color.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to fill.
    /// * `color` - The fill color.
    fn fill_rect(&mut self, bounds: Rect, color: Color);

    /// Draws a one-pixel rectangle outline.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to outline.
    /// * `color` - The line color.
    fn rect(&mut self, bounds: Rect, color: Color);

    /// Draws a one-pixel line.
    ///
    /// # Arguments
    ///
    /// * `from` - The start point.
    /// * `to` - The end point.
    /// * `color` - The line color.
    fn line(&mut self, from: Point, to: Point, color: Color);

    /// Draws text within a rectangle, aligned by `style`, wrapped at the rectangle's width, and
    /// centered vertically when it fits (top-aligned when it doesn't).
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to draw within.
    /// * `style` - The font and alignment.
    /// * `color` - The text color (overrides `style.font_color`).
    /// * `msg` - The text to draw.
    ///
    /// # Returns
    ///
    /// The size of the text drawn; `0` by `0` if nothing was drawn.
    fn text(&mut self, bounds: Rect, style: &TextStyle, color: Color, msg: &str) -> Size;

    /// Returns the height of one line of text in a font.
    ///
    /// # Arguments
    ///
    /// * `style` - The font.
    ///
    /// # Returns
    ///
    /// The line height in pixels.
    fn line_height(&mut self, style: &TextStyle) -> u32;

    /// Measures how wide text is on one line, without drawing it.
    ///
    /// # Arguments
    ///
    /// * `style` - The font.
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// The width in pixels.
    fn text_width(&mut self, style: &TextStyle, msg: &str) -> u32;

    /// Draws an image file, scaled to a rectangle.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file.
    /// * `bounds` - The rectangle to draw into.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the image is drawn.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] if the image cannot be loaded or drawn.
    fn image(&mut self, path: &str, bounds: Rect) -> Result<(), Error>;

    /// Fills a rectangle with a 50% checkerboard of two colors, for backends and displays with few
    /// colors.  The pixel at the rectangle's upper left-hand corner is `first`.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to fill.
    /// * `first` - The color of the corner pixel.
    /// * `second` - The other color.
    fn dither(&mut self, bounds: Rect, first: Color, second: Color);

    /// Draws a color over every other pixel of a rectangle (the pixels a [`dither`](Painter::dither)
    /// would draw in `first`), blending with what is already there and leaving the other pixels
    /// untouched.  The pixel at the rectangle's upper left-hand corner is drawn.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to draw over.
    /// * `color` - The color to blend; its alpha sets the strength.
    fn dither_overlay(&mut self, bounds: Rect, color: Color);

    /// Limits later drawing to a rectangle, or removes the limit.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to draw within, or `None` for no limit.
    fn clip(&mut self, bounds: Option<Rect>);
}

/// Fills the whole drawing surface with a color.
///
/// This is the background of most `Widget`s, and usually the first thing their `paint` draws.
///
/// # Arguments
///
/// * `p` - The painter, targeting the `Widget`.
/// * `color` - The fill color.
pub fn fill(p: &mut dyn Painter, color: Color) {
    p.fill(color);
}

/// Draws a border inside the edges of the surface.
///
/// The border is `width` one-pixel rectangles, each inset one pixel further.  A `width` of `0`
/// draws nothing, and drawing stops early if the border would cover the whole surface.
///
/// # Arguments
///
/// * `p` - The painter, targeting the `Widget`.
/// * `size` - The size of the surface (the `Widget`'s size).
/// * `color` - The border color.
/// * `width` - The border width in pixels.
pub fn border(p: &mut dyn Painter, size: Size, color: Color, width: u8) {
    for i in 0..width as u32 {
        if i * 2 >= size.w || i * 2 >= size.h {
            break;
        }

        p.rect(
            Rect::new(i as i32, i as i32, size.w - i * 2, size.h - i * 2),
            color,
        );
    }
}

/// Draws the generic look of a `Widget`'s visual state on top of what its `paint` drew.
///
/// Call it after `paint` (the `WidgetCache` does this for every `Widget`):
///
/// - [`VisualState::DISABLED`]: a 50% white dither washes out the `Widget`.
/// - [`VisualState::FOCUSED`]: a dotted outline, inset one pixel.
/// - [`VisualState::INDETERMINATE`]: a dotted horizontal bar through the middle.
///
/// The other states (`PRESSED`, `CHECKED`, `HOVERED`, `DRAG_OVER`) are drawn by the `Widget`, since
/// their look depends on what the `Widget` is.
///
/// # Arguments
///
/// * `p` - The painter, targeting the `Widget`.
/// * `size` - The size of the surface (the `Widget`'s size).
/// * `state` - The `Widget`'s visual state.
pub fn state_overlay(p: &mut dyn Painter, size: Size, state: VisualState) {
    if size.w == 0 || size.h == 0 {
        return;
    }

    if state.contains(VisualState::INDETERMINATE) && size.h >= 3 {
        let y = (size.h / 2) as i32;

        p.dither_overlay(
            Rect::new(1, y, size.w.saturating_sub(2).max(1), 1),
            Color::BLACK,
        );
    }

    if state.contains(VisualState::FOCUSED) && size.w > 2 && size.h > 2 {
        let outline = Color::BLACK;
        let (w, h) = (size.w - 2, size.h - 2);

        p.dither_overlay(Rect::new(1, 1, w, 1), outline);
        p.dither_overlay(Rect::new(1, h as i32, w, 1), outline);
        p.dither_overlay(Rect::new(1, 1, 1, h), outline);
        p.dither_overlay(Rect::new(w as i32, 1, 1, h), outline);
    }

    if state.contains(VisualState::DISABLED) {
        p.dither_overlay(Rect::new(0, 0, size.w, size.h), Color::WHITE);
    }
}

/// Draws a string of text within a rectangle.
///
/// The text is aligned horizontally according to `style`, wraps at the rectangle's width, and is
/// centered vertically when it fits (top-aligned when it doesn't).  If it is wider than the
/// rectangle, `truncate` shortens it with an ellipsis (see [`fit_text`]).  An empty string, or a
/// rectangle with no width, draws nothing.
///
/// # Arguments
///
/// * `p` - The painter, targeting the `Widget`.
/// * `bounds` - The rectangle to draw within, in `Widget` coordinates.
/// * `style` - The font and alignment.
/// * `color` - The text color (overrides `style.font_color`, so a `Widget` can invert it).
/// * `msg` - The text to draw.
/// * `truncate` - How to shorten text that doesn't fit.
///
/// # Returns
///
/// The size of the text drawn (`0` by `0` if nothing was drawn) and whether it was shortened.
pub fn text(
    p: &mut dyn Painter,
    bounds: Rect,
    style: &TextStyle,
    color: Color,
    msg: &str,
    truncate: Truncate,
) -> TextDrawn {
    if msg.is_empty() || bounds.w == 0 {
        return TextDrawn {
            size: Size::new(0, 0),
            truncated: false,
        };
    }

    let shortened = fit_text(msg, bounds.w, truncate, &mut |s| p.text_width(style, s));
    let truncated = shortened.is_some();
    let drawn = shortened.as_deref().unwrap_or(msg);

    if drawn.is_empty() {
        return TextDrawn {
            size: Size::new(0, 0),
            truncated,
        };
    }

    TextDrawn {
        size: p.text(bounds, style, color, drawn),
        truncated,
    }
}

/// Shortens text so it fits a width, cutting on grapheme boundaries and marking the cut with an
/// ellipsis.  The ellipsis is measured with the same `measure` function as the text, so the result
/// never overflows `max_width`.
///
/// # Arguments
///
/// * `msg` - The text.
/// * `max_width` - The width in pixels the text must fit.
/// * `mode` - Which part of the text to cut.
/// * `measure` - Returns the width in pixels of a string in the font being drawn.
///
/// # Returns
///
/// `None` if the text fits as is, or if `mode` is [`Truncate::None`] or the text has a newline.
/// Otherwise the shortened text, which is empty if not even the ellipsis fits.
pub fn fit_text(
    msg: &str,
    max_width: u32,
    mode: Truncate,
    measure: &mut dyn FnMut(&str) -> u32,
) -> Option<String> {
    if mode == Truncate::None || msg.contains('\n') || measure(msg) <= max_width {
        return None;
    }

    let graphemes: Vec<&str> = msg.graphemes(true).collect();
    let build = |keep: usize| -> String {
        match mode {
            Truncate::Start => {
                format!("{ELLIPSIS}{}", graphemes[graphemes.len() - keep..].concat())
            }
            Truncate::Middle => {
                let head = keep.div_ceil(2);
                let tail = keep - head;

                format!(
                    "{}{ELLIPSIS}{}",
                    graphemes[..head].concat(),
                    graphemes[graphemes.len() - tail..].concat()
                )
            }
            _ => format!("{}{ELLIPSIS}", graphemes[..keep].concat()),
        }
    };

    // Width grows with the number of kept graphemes, so binary search for the most that fit.
    let (mut low, mut high) = (0, graphemes.len());
    let mut best: Option<String> = None;

    while low <= high {
        let mid = (low + high) / 2;
        let candidate = build(mid);

        if measure(&candidate) <= max_width {
            best = Some(candidate);
            low = mid + 1;
        } else if mid == 0 {
            break;
        } else {
            high = mid - 1;
        }
    }

    Some(best.unwrap_or_default())
}

/// Breaks text into lines no wider than a width.
///
/// Explicit `\n` always starts a new line (an empty line stays empty).  With [`Wrap::Word`] lines
/// break at spaces (runs of spaces collapse to one) and a word wider than the width is broken
/// between graphemes; with [`Wrap::Anywhere`] they break between graphemes.  [`Wrap::None`] only
/// splits at `\n`.  If `max_lines` is set and the text needs more lines, the last line is cut
/// and ends in an ellipsis.
///
/// # Arguments
///
/// * `msg` - The text.
/// * `max_width` - The width in pixels each line must fit (a single grapheme wider than this is
///   still put on its own line).
/// * `wrap` - How to break lines.
/// * `max_lines` - The most lines to return (at least one is returned), or `None` for no limit.
/// * `measure` - Returns the width in pixels of a string in the font being drawn.
///
/// # Returns
///
/// The lines, top to bottom, and whether the last line was cut to honor `max_lines`.
pub fn wrap_text(
    msg: &str,
    max_width: u32,
    wrap: Wrap,
    max_lines: Option<usize>,
    measure: &mut dyn FnMut(&str) -> u32,
) -> (Vec<String>, bool) {
    let mut lines: Vec<String> = Vec::new();

    for paragraph in msg.split('\n') {
        match wrap {
            Wrap::None => lines.push(paragraph.to_string()),
            Wrap::Anywhere => break_graphemes(paragraph, max_width, measure, &mut lines),
            Wrap::Word => break_words(paragraph, max_width, measure, &mut lines),
        }
    }

    let Some(limit) = max_lines.map(|n| n.max(1)) else {
        return (lines, false);
    };

    if lines.len() <= limit {
        return (lines, false);
    }

    lines.truncate(limit);

    // Dropping the trailing ellipsis from the candidate is what fit_text does when it must cut, so
    // the result always ends in exactly one ellipsis.
    let last = format!("{}{ELLIPSIS}", lines[limit - 1]);
    let cut = fit_text(&last, max_width, Truncate::End, measure).unwrap_or(last);

    lines[limit - 1] = cut;

    (lines, true)
}

/// Breaks one paragraph between graphemes, as late as possible, appending the lines to `out`.
///
/// # Arguments
///
/// * `text` - The paragraph (no newlines).
/// * `max_width` - The width each line must fit.
/// * `measure` - Measures a string.
/// * `out` - Receives the lines; an empty paragraph adds one empty line.
fn break_graphemes(
    text: &str,
    max_width: u32,
    measure: &mut dyn FnMut(&str) -> u32,
    out: &mut Vec<String>,
) {
    let mut current = String::new();

    for g in text.graphemes(true) {
        let candidate = format!("{current}{g}");

        if current.is_empty() || measure(&candidate) <= max_width {
            current = candidate;
        } else {
            out.push(std::mem::replace(&mut current, g.to_string()));
        }
    }

    out.push(current);
}

/// Breaks one paragraph at spaces, appending the lines to `out`.
///
/// # Arguments
///
/// * `text` - The paragraph (no newlines).
/// * `max_width` - The width each line must fit.
/// * `measure` - Measures a string.
/// * `out` - Receives the lines; an empty paragraph adds one empty line.
fn break_words(
    text: &str,
    max_width: u32,
    measure: &mut dyn FnMut(&str) -> u32,
    out: &mut Vec<String>,
) {
    let mut current = String::new();

    for word in text.split(' ').filter(|w| !w.is_empty()) {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        if measure(&candidate) <= max_width {
            current = candidate;
            continue;
        }

        if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }

        if measure(word) <= max_width {
            current = word.to_string();
        } else {
            // A word wider than the width: break it between graphemes; the last piece may take
            // the next word.
            let mut pieces = Vec::new();

            break_graphemes(word, max_width, measure, &mut pieces);
            current = pieces.pop().unwrap_or_default();
            out.extend(pieces);
        }
    }

    out.push(current);
}

/// Measures the height of wrapped text for a given width, so a layout can give a label the height
/// it needs.
///
/// # Arguments
///
/// * `p` - The painter (it measures).
/// * `style` - The font.
/// * `msg` - The text.
/// * `width` - The width in pixels the text will be drawn at.
/// * `wrap` - How lines break; [`Wrap::None`] is one line per `\n`.
/// * `max_lines` - The most lines, or `None`.
///
/// # Returns
///
/// The height in pixels: the number of lines times the line height.
pub fn height_for_width(
    p: &mut dyn Painter,
    style: &TextStyle,
    msg: &str,
    width: u32,
    wrap: Wrap,
    max_lines: Option<usize>,
) -> u32 {
    if msg.is_empty() {
        return 0;
    }

    let (lines, _) = wrap_text(msg, width, wrap, max_lines, &mut |s| p.text_width(style, s));

    lines.len() as u32 * p.line_height(style)
}

/// Draws wrapped text: a block of lines, each aligned by `style.justification` within the
/// rectangle, centered vertically when the block fits (top-aligned when it doesn't).  Lines that
/// start below the rectangle are not drawn.
///
/// # Arguments
///
/// * `p` - The painter, targeting the `Widget`.
/// * `bounds` - The rectangle to draw within, in `Widget` coordinates.
/// * `style` - The font and alignment.
/// * `color` - The text color.
/// * `msg` - The text; `\n` starts a new line.
/// * `wrap` - How lines break (see [`wrap_text`]).
/// * `max_lines` - The most lines; the last is cut with an ellipsis if the text needs more.
///
/// # Returns
///
/// The size of the block drawn (`0` by `0` if nothing was drawn) and whether the text was cut to
/// honor `max_lines`.
pub fn paragraph(
    p: &mut dyn Painter,
    bounds: Rect,
    style: &TextStyle,
    color: Color,
    msg: &str,
    wrap: Wrap,
    max_lines: Option<usize>,
) -> TextDrawn {
    if msg.is_empty() || bounds.w == 0 {
        return TextDrawn {
            size: Size::new(0, 0),
            truncated: false,
        };
    }

    let (lines, truncated) = wrap_text(msg, bounds.w, wrap, max_lines, &mut |s| {
        p.text_width(style, s)
    });
    let line_h = p.line_height(style);
    let block_h = lines.len() as u32 * line_h;
    let top = if block_h < bounds.h {
        bounds.y + (bounds.h - block_h) as i32 / 2
    } else {
        bounds.y
    };
    let bottom = bounds.y + bounds.h as i32;
    let mut widest = 0;

    for (i, line) in lines.iter().enumerate() {
        let y = top + i as i32 * line_h as i32;

        if y >= bottom {
            break;
        }

        if !line.is_empty() {
            let drawn = p.text(Rect::new(bounds.x, y, bounds.w, line_h), style, color, line);

            widest = widest.max(drawn.w);
        }
    }

    TextDrawn {
        size: Size::new(widest, block_h),
        truncated,
    }
}

/// Computes where text of a given size is drawn within a rectangle.
///
/// Backends use this so every backend aligns text the same way: horizontally by `justification`,
/// and vertically centered when the text fits (top-aligned when it doesn't).
///
/// # Arguments
///
/// * `bounds` - The rectangle to draw within.
/// * `text_size` - The size of the rendered text.
/// * `justification` - The horizontal alignment.
///
/// # Returns
///
/// The rectangle the text occupies.
pub fn align_text(bounds: Rect, text_size: Size, justification: TextJustify) -> Rect {
    let bounds_width = bounds.w as i32;
    let x = bounds.x
        + match justification {
            TextJustify::Left => 0,
            TextJustify::Right => bounds_width - text_size.w as i32,
            TextJustify::Center => (bounds_width - text_size.w as i32) / 2,
        };
    let y = if text_size.h < bounds.h {
        bounds.y + (bounds.h - text_size.h) as i32 / 2
    } else {
        bounds.y
    };

    Rect::new(x, y, text_size.w, text_size.h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::headless::{PaintCall, RecordingPainter};
    use crate::color::FontStyle;

    fn style(justification: TextJustify) -> TextStyle {
        TextStyle::new(
            Font::new("font", 10, FontStyle::NORMAL),
            Color::BLACK,
            justification,
        )
    }

    #[test]
    fn state_overlay_draws_nothing_for_the_normal_state() {
        let mut p = RecordingPainter::default();
        state_overlay(&mut p, Size::new(40, 20), VisualState::empty());
        state_overlay(
            &mut p,
            Size::new(40, 20),
            VisualState::CHECKED | VisualState::HOVERED,
        );
        assert!(p.calls.is_empty());
    }

    #[test]
    fn state_overlay_dims_disabled_widgets_last() {
        let mut p = RecordingPainter::default();
        state_overlay(
            &mut p,
            Size::new(40, 20),
            VisualState::DISABLED | VisualState::FOCUSED,
        );

        assert_eq!(
            p.calls.last(),
            Some(&PaintCall::DitherOverlay(
                Rect::new(0, 0, 40, 20),
                Color::WHITE
            ))
        );
        assert_eq!(p.calls.len(), 5, "four focus edges, then the dim");
    }

    #[test]
    fn state_overlay_draws_a_focus_outline_and_an_indeterminate_bar() {
        let mut p = RecordingPainter::default();
        state_overlay(&mut p, Size::new(40, 20), VisualState::FOCUSED);
        assert_eq!(p.calls.len(), 4);

        let mut p = RecordingPainter::default();
        state_overlay(&mut p, Size::new(40, 20), VisualState::INDETERMINATE);
        assert_eq!(
            p.calls,
            vec![PaintCall::DitherOverlay(
                Rect::new(1, 10, 38, 1),
                Color::BLACK
            )]
        );
    }

    #[test]
    fn state_overlay_ignores_empty_and_tiny_surfaces() {
        let mut p = RecordingPainter::default();
        state_overlay(&mut p, Size::new(0, 10), VisualState::DISABLED);
        state_overlay(
            &mut p,
            Size::new(2, 2),
            VisualState::FOCUSED | VisualState::INDETERMINATE,
        );
        assert!(p.calls.is_empty());
    }

    #[test]
    fn fill_records_one_call() {
        let mut p = RecordingPainter::default();
        fill(&mut p, Color::RED);
        assert_eq!(p.calls, vec![PaintCall::Fill(Color::RED)]);
    }

    #[test]
    fn border_insets_each_ring() {
        let mut p = RecordingPainter::default();
        border(&mut p, Size::new(10, 10), Color::BLUE, 2);
        assert_eq!(
            p.calls,
            vec![
                PaintCall::Rect(Rect::new(0, 0, 10, 10), Color::BLUE),
                PaintCall::Rect(Rect::new(1, 1, 8, 8), Color::BLUE),
            ]
        );
    }

    #[test]
    fn border_zero_width_draws_nothing() {
        let mut p = RecordingPainter::default();
        border(&mut p, Size::new(10, 10), Color::BLUE, 0);
        assert!(p.calls.is_empty());
    }

    #[test]
    fn border_stops_when_it_covers_the_surface() {
        let mut p = RecordingPainter::default();
        border(&mut p, Size::new(4, 10), Color::BLUE, 9);
        assert_eq!(p.calls.len(), 2);
    }

    #[test]
    fn empty_text_or_zero_width_draws_nothing() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);
        assert_eq!(
            text(
                &mut p,
                Rect::new(0, 0, 50, 20),
                &s,
                Color::BLACK,
                "",
                Truncate::End
            )
            .size,
            Size::new(0, 0)
        );
        assert_eq!(
            text(
                &mut p,
                Rect::new(0, 0, 0, 20),
                &s,
                Color::BLACK,
                "hi",
                Truncate::End
            )
            .size,
            Size::new(0, 0)
        );
        assert!(p.calls.is_empty());
    }

    #[test]
    fn text_is_recorded() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);
        let drawn = text(
            &mut p,
            Rect::new(0, 0, 100, 20),
            &s,
            Color::WHITE,
            "hi",
            Truncate::End,
        );
        assert_ne!(drawn.size, Size::new(0, 0));
        assert!(!drawn.truncated, "text that fits is drawn unchanged");
        assert_eq!(
            p.calls,
            vec![PaintCall::Text(
                Rect::new(0, 0, 100, 20),
                String::from("hi"),
                Color::WHITE
            )]
        );
    }

    #[test]
    fn align_text_justifies_and_centers() {
        let bounds = Rect::new(10, 20, 100, 40);
        let t = Size::new(30, 10);
        assert_eq!(
            align_text(bounds, t, TextJustify::Left),
            Rect::new(10, 35, 30, 10)
        );
        assert_eq!(
            align_text(bounds, t, TextJustify::Right),
            Rect::new(80, 35, 30, 10)
        );
        assert_eq!(
            align_text(bounds, t, TextJustify::Center),
            Rect::new(45, 35, 30, 10)
        );
    }

    #[test]
    fn align_text_top_aligns_when_too_tall() {
        let bounds = Rect::new(0, 5, 100, 10);
        assert_eq!(
            align_text(bounds, Size::new(20, 30), TextJustify::Left),
            Rect::new(0, 5, 20, 30)
        );
    }

    #[test]
    fn cache_policy_defaults_to_texture_cache() {
        assert_eq!(CachePolicy::default(), CachePolicy::TextureCache);
    }

    /// Draws `msg` in a 50-pixel-wide box (ten characters at 5 pixels each) and returns what the
    /// painter recorded, plus whether the text was shortened.
    fn drawn_text(msg: &str, mode: Truncate) -> (String, bool) {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);
        let drawn = text(&mut p, Rect::new(0, 0, 50, 20), &s, Color::BLACK, msg, mode);
        let recorded = match p.calls.first() {
            Some(PaintCall::Text(_, t, _)) => t.clone(),
            _ => String::new(),
        };

        (recorded, drawn.truncated)
    }

    #[test]
    fn text_that_fits_is_unchanged_in_every_mode() {
        for mode in [
            Truncate::None,
            Truncate::End,
            Truncate::Start,
            Truncate::Middle,
        ] {
            assert_eq!(
                drawn_text("0123456789", mode),
                (String::from("0123456789"), false)
            );
        }
    }

    #[test]
    fn end_start_and_middle_cut_with_an_ellipsis() {
        let msg = "Quarterly report.txt";

        assert_eq!(
            drawn_text(msg, Truncate::End),
            (String::from("Quarterly\u{2026}"), true)
        );
        assert_eq!(
            drawn_text(msg, Truncate::Start),
            (String::from("\u{2026}eport.txt"), true)
        );
        assert_eq!(
            drawn_text(msg, Truncate::Middle),
            (String::from("Quart\u{2026}.txt"), true)
        );
    }

    #[test]
    fn none_leaves_long_text_for_the_backend_to_clip() {
        assert_eq!(
            drawn_text("Quarterly report.txt", Truncate::None),
            (String::from("Quarterly report.txt"), false)
        );
    }

    #[test]
    fn truncated_text_never_overflows() {
        let mut measure = |s: &str| s.chars().count() as u32 * 5;

        for mode in [Truncate::End, Truncate::Start, Truncate::Middle] {
            for width in 0..60 {
                let cut = fit_text("The quick brown fox jumps", width, mode, &mut measure).unwrap();
                assert!(measure(&cut) <= width, "{mode:?} at {width}: {cut:?}");
            }
        }
    }

    #[test]
    fn a_width_too_small_for_the_ellipsis_draws_nothing() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);
        let drawn = text(
            &mut p,
            Rect::new(0, 0, 4, 20),
            &s,
            Color::BLACK,
            "hello",
            Truncate::End,
        );

        assert!(drawn.truncated);
        assert_eq!(drawn.size, Size::new(0, 0));
        assert!(p.calls.is_empty());
    }

    #[test]
    fn cuts_fall_on_grapheme_boundaries() {
        // "e" + combining acute, a flag (two scalars), and a family emoji (a ZWJ sequence).
        let msg = "e\u{301}e\u{301}e\u{301}\u{1F1FA}\u{1F1F8}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}ab";
        let mut by_graphemes = |s: &str| s.graphemes(true).count() as u32;

        for mode in [Truncate::End, Truncate::Start, Truncate::Middle] {
            let cut = fit_text(msg, 4, mode, &mut by_graphemes).unwrap();
            assert_eq!(cut.graphemes(true).count(), 4, "{mode:?}: {cut:?}");
        }

        let end = fit_text(msg, 4, Truncate::End, &mut by_graphemes).unwrap();
        assert_eq!(end, "e\u{301}e\u{301}e\u{301}\u{2026}");
    }

    #[test]
    fn text_with_a_newline_is_not_truncated() {
        let mut measure = |s: &str| s.chars().count() as u32 * 5;

        assert_eq!(
            fit_text("a long first line\nb", 10, Truncate::End, &mut measure),
            None
        );
    }

    #[test]
    fn end_is_the_default_mode() {
        assert_eq!(Truncate::default(), Truncate::End);
    }

    /// Five pixels per character, like the recording painter at font size 10.
    fn five(s: &str) -> u32 {
        s.chars().count() as u32 * 5
    }

    #[test]
    fn word_wrap_breaks_at_spaces_and_reflows() {
        let msg = "the quick brown fox jumps";
        let (wide, _) = wrap_text(msg, 100, Wrap::Word, None, &mut five);
        let (narrow, _) = wrap_text(msg, 50, Wrap::Word, None, &mut five);

        assert_eq!(wide, vec!["the quick brown fox", "jumps"]);
        assert_eq!(narrow, vec!["the quick", "brown fox", "jumps"]);
        assert!(narrow.iter().all(|l| five(l) <= 50));
    }

    #[test]
    fn explicit_newlines_are_kept_and_blank_lines_stay() {
        let (lines, _) = wrap_text("a\n\nb c", 100, Wrap::Word, None, &mut five);

        assert_eq!(lines, vec!["a", "", "b c"]);
    }

    #[test]
    fn a_word_wider_than_the_width_is_broken() {
        let (lines, _) = wrap_text("abcdefghij kl", 20, Wrap::Word, None, &mut five);

        assert_eq!(lines, vec!["abcd", "efgh", "ij", "kl"]);
    }

    #[test]
    fn anywhere_breaks_between_graphemes() {
        let (lines, _) = wrap_text("abcdefghij", 25, Wrap::Anywhere, None, &mut five);

        assert_eq!(lines, vec!["abcde", "fghij"]);

        let (lines, _) = wrap_text(
            "e\u{301}e\u{301}e\u{301}",
            5,
            Wrap::Anywhere,
            None,
            &mut |s| s.graphemes(true).count() as u32 * 5,
        );

        assert_eq!(
            lines,
            vec!["e\u{301}", "e\u{301}", "e\u{301}"],
            "accents stay with their letter"
        );
    }

    #[test]
    fn max_lines_cuts_the_last_line_with_an_ellipsis() {
        let (lines, cut) = wrap_text(
            "the quick brown fox jumps",
            50,
            Wrap::Word,
            Some(2),
            &mut five,
        );

        assert!(cut);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "the quick");
        assert_eq!(lines[1], "brown fox\u{2026}");
        assert!(lines.iter().all(|l| five(l) <= 50));

        let (lines, cut) = wrap_text("a\nb", 100, Wrap::Word, Some(1), &mut five);

        assert!(cut, "an explicit break past the limit also cuts");
        assert_eq!(lines, vec!["a\u{2026}"]);

        let (_, cut) = wrap_text("a b", 100, Wrap::Word, Some(2), &mut five);

        assert!(!cut, "text that fits is not cut");
    }

    #[test]
    fn paragraph_aligns_each_line_and_reports_height() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Right);
        let drawn = paragraph(
            &mut p,
            Rect::new(0, 0, 50, 40),
            &s,
            Color::BLACK,
            "the quick brown fox",
            Wrap::Word,
            None,
        );

        assert_eq!(drawn.size.h, 20, "two lines of ten pixels");
        assert!(!drawn.truncated);
        assert_eq!(
            p.calls,
            vec![
                PaintCall::Text(
                    Rect::new(0, 10, 50, 10),
                    String::from("the quick"),
                    Color::BLACK
                ),
                PaintCall::Text(
                    Rect::new(0, 20, 50, 10),
                    String::from("brown fox"),
                    Color::BLACK
                ),
            ],
            "each line gets its own rectangle, so the justification applies per line"
        );
    }

    #[test]
    fn paragraph_skips_lines_below_the_bounds() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);

        paragraph(
            &mut p,
            Rect::new(0, 0, 50, 15),
            &s,
            Color::BLACK,
            "a\nb\nc",
            Wrap::Word,
            None,
        );

        assert_eq!(
            p.calls.len(),
            2,
            "lines at y 0 and 10 start inside; y 20 does not"
        );
    }

    #[test]
    fn height_for_width_grows_as_the_width_shrinks() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);
        let msg = "the quick brown fox jumps";

        assert_eq!(height_for_width(&mut p, &s, msg, 100, Wrap::Word, None), 20);
        assert_eq!(height_for_width(&mut p, &s, msg, 50, Wrap::Word, None), 30);
        assert_eq!(
            height_for_width(&mut p, &s, msg, 50, Wrap::Word, Some(2)),
            20
        );
        assert_eq!(height_for_width(&mut p, &s, "", 50, Wrap::Word, None), 0);
    }
}
