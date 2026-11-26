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
//!   BaseWidget   = fill
//!   BoxWidget    = fill + border
//!   TextWidget   = fill + text
//!   ButtonWidget = fill + text + border   (+ pressed state)
//! ```
//!
//! The default backend is [`Sdl2Painter`](crate::backend::sdl2::Sdl2Painter): each `Widget` paints
//! into its own GPU texture only when invalidated, and frames are blits of cached textures.  A
//! backend declares its [`CachePolicy`].  The [`RecordingPainter`](crate::backend::headless::RecordingPainter)
//! records calls for unit tests, without a window.

use crate::color::{Color, FontStyle};
use crate::geometry::{Point, Rect, Size};
use crate::widget::VisualState;
use unicode_segmentation::UnicodeSegmentation;

/// Horizontal alignment of text within its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// The ellipsis drawn where text is cut.
pub const ELLIPSIS: &str = "\u{2026}";

/// The result of [`text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextDrawn {
    /// The size of the text drawn; `0` by `0` if nothing was drawn.
    pub size: Size,
    /// Whether the text was shortened to fit (see [`Truncate`]), so a tooltip can show it in full.
    pub truncated: bool,
}

/// How a backend keeps painted `Widget`s between frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    /// Path to the TrueType font file.
    pub font_name: String,

    /// Font style (normal, bold, italic, ...)
    pub font_style: FontStyle,

    /// Font size in points.
    pub font_size: u16,

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
    /// * `font_name` - Path to the TrueType font file.
    /// * `font_style` - The font style (normal, bold, italic, ...).
    /// * `font_size` - The font size in points.
    /// * `font_color` - The text color.
    /// * `justification` - The horizontal alignment.
    ///
    /// # Returns
    ///
    /// The new `TextStyle`.
    pub fn new(
        font_name: String,
        font_style: FontStyle,
        font_size: u16,
        font_color: Color,
        justification: TextJustify,
    ) -> Self {
        Self {
            font_name,
            font_style,
            font_size,
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
    /// Returns a message if the image cannot be loaded or drawn.
    fn image(&mut self, path: &str, bounds: Rect) -> Result<(), String>;

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

    fn style(justification: TextJustify) -> TextStyle {
        TextStyle::new(
            String::from("font.ttf"),
            FontStyle::NORMAL,
            10,
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
}
