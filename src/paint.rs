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
/// centered vertically when it fits (top-aligned when it doesn't).  An empty string, or a
/// rectangle with no width, draws nothing.
///
/// # Arguments
///
/// * `p` - The painter, targeting the `Widget`.
/// * `bounds` - The rectangle to draw within, in `Widget` coordinates.
/// * `style` - The font and alignment.
/// * `color` - The text color (overrides `style.font_color`, so a `Widget` can invert it).
/// * `msg` - The text to draw.
///
/// # Returns
///
/// The size of the text drawn; `0` by `0` if nothing was drawn.
pub fn text(p: &mut dyn Painter, bounds: Rect, style: &TextStyle, color: Color, msg: &str) -> Size {
    if msg.is_empty() || bounds.w == 0 {
        return Size::new(0, 0);
    }

    p.text(bounds, style, color, msg)
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
            text(&mut p, Rect::new(0, 0, 50, 20), &s, Color::BLACK, ""),
            Size::new(0, 0)
        );
        assert_eq!(
            text(&mut p, Rect::new(0, 0, 0, 20), &s, Color::BLACK, "hi"),
            Size::new(0, 0)
        );
        assert!(p.calls.is_empty());
    }

    #[test]
    fn text_is_recorded() {
        let mut p = RecordingPainter::default();
        let s = style(TextJustify::Left);
        let drawn = text(&mut p, Rect::new(0, 0, 100, 20), &s, Color::WHITE, "hi");
        assert_ne!(drawn, Size::new(0, 0));
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
}
