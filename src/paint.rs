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
//! Reusable drawing building blocks.  `Widget`s are composed by calling these functions in
//! order from their `paint` function, all drawing into the `Widget`'s single texture:
//!
//! ```text
//!   RootWidget   = fill
//!   BaseWidget   = fill
//!   BoxWidget    = fill + border
//!   TextWidget   = fill + text
//!   ButtonWidget = fill + text + border   (+ pressed state)
//! ```
//!
//! This replaces composing `Widget`s out of other `Widget`s, which required every sub-`Widget` to
//! keep its own texture, size, and invalidation state in sync with its parent.

use crate::font::FontCache;
use crate::geometry::Size;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

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

/// The font and alignment used to draw text with [`text`].
#[derive(Clone)]
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

/// Fills the whole texture with a color.
///
/// This is the background of most `Widget`s, and usually the first thing their `paint` draws.
///
/// # Arguments
///
/// * `c` - The canvas, targeting the `Widget`'s texture.
/// * `color` - The fill color.
pub fn fill(c: &mut Canvas<Window>, color: Color) {
    c.set_draw_color(color);
    c.clear();
}

/// Draws a border inside the edges of the texture.
///
/// The border is `width` one-pixel rectangles, each inset one pixel further.  A `width` of `0`
/// draws nothing, and drawing stops early if the border would cover the whole texture.
///
/// # Arguments
///
/// * `c` - The canvas, targeting the `Widget`'s texture.
/// * `size` - The size of the texture (the `Widget`'s size).
/// * `color` - The border color.
/// * `width` - The border width in pixels.
///
/// # Panics
///
/// Panics if `SDL2` fails to draw a rectangle.
pub fn border(c: &mut Canvas<Window>, size: Size, color: Color, width: u8) {
    c.set_draw_color(color);

    for i in 0..width as u32 {
        if i * 2 >= size.w || i * 2 >= size.h {
            break;
        }

        c.draw_rect(Rect::new(
            i as i32,
            i as i32,
            size.w - i * 2,
            size.h - i * 2,
        ))
        .unwrap();
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
/// * `c` - The canvas, targeting the `Widget`'s texture.
/// * `fc` - The font cache used to render the text.
/// * `bounds` - The rectangle to draw within, in texture coordinates.
/// * `style` - The font and alignment.
/// * `color` - The text color (overrides `style.font_color`, so a `Widget` can invert it).
/// * `msg` - The text to draw.
///
/// # Panics
///
/// Panics if the font cannot be loaded or `SDL2` fails to render or copy the text.
pub fn text(
    c: &mut Canvas<Window>,
    fc: &mut FontCache,
    bounds: Rect,
    style: &TextStyle,
    color: Color,
    msg: &str,
) {
    if msg.is_empty() || bounds.width() == 0 {
        return;
    }

    let (font_texture, font_width, font_height) = fc.render_text(
        c,
        &style.font_name,
        style.font_size,
        style.font_style,
        color,
        msg,
        bounds.width(),
    );

    let bounds_width = bounds.width() as i32;
    let text_x = bounds.x()
        + match style.justification {
            TextJustify::Left => 0,
            TextJustify::Right => bounds_width - font_width as i32,
            TextJustify::Center => (bounds_width - font_width as i32) / 2,
        };

    // Text that fits is centered vertically; text taller than the bounds is top-aligned.
    let text_y = if font_height < bounds.height() {
        bounds.y() + (bounds.height() - font_height) as i32 / 2
    } else {
        bounds.y()
    };

    c.copy(
        &font_texture,
        None,
        Rect::new(text_x, text_y, font_width, font_height),
    )
    .unwrap();

    // With `unsafe_textures`, textures must be explicitly destroyed, otherwise every repaint of
    // text leaks a GPU texture.
    unsafe { font_texture.destroy() };
}
