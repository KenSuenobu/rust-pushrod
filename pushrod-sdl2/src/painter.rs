// SDL2 Backend
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

//! # SDL2 backend
//!
//! The default [`Painter`]: draws with `SDL2` into the canvas it wraps.  The `WidgetCache` points
//! the canvas at a `Widget`'s own GPU texture while the `Widget` is invalidated, so each `Widget`
//! paints into its own texture only when it changes, and frames are blits of cached textures.
//! This is the only module (with the engine, caches, and font renderer) that touches `SDL2` types
//! for drawing.

use crate::font::FontCache;
use pushrod::color::{Color, FontStyle};
use pushrod::error::Error;
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::{self, CachePolicy, Painter, TextStyle};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::{Point as SdlPoint, Rect as SdlRect};
use sdl2::render::{BlendMode, Canvas};
use sdl2::ttf::FontStyle as SdlFontStyle;
use sdl2::video::Window;

/// Converts a Pushrod [`Color`] to an `SDL2` color.
///
/// # Arguments
///
/// * `c` - The color.
///
/// # Returns
///
/// The `SDL2` color.
pub fn to_sdl_color(c: Color) -> SdlColor {
    SdlColor::RGBA(c.r, c.g, c.b, c.a)
}

/// Converts a Pushrod [`Rect`] to an `SDL2` rectangle.
///
/// # Arguments
///
/// * `r` - The rectangle.
///
/// # Returns
///
/// The `SDL2` rectangle.
pub fn to_sdl_rect(r: Rect) -> SdlRect {
    SdlRect::new(r.x, r.y, r.w, r.h)
}

/// Converts a Pushrod [`FontStyle`] to an `SDL2_ttf` font style.
///
/// # Arguments
///
/// * `s` - The font style.
///
/// # Returns
///
/// The `SDL2_ttf` style; the flag bits are the same.
pub fn to_sdl_font_style(s: FontStyle) -> SdlFontStyle {
    SdlFontStyle::from_bits_truncate(s.bits() as _)
}

/// The `SDL2` [`Painter`], drawing into a canvas (usually targeting a `Widget`'s texture).
pub struct Sdl2Painter<'a> {
    canvas: &'a mut Canvas<Window>,
    fonts: &'a mut FontCache,
    errors: Vec<Error>,
}

impl Sdl2Painter<'_> {
    /// The cache policy of the `SDL2` backend: per-widget GPU textures.
    pub const CACHE_POLICY: CachePolicy = CachePolicy::TextureCache;
}

impl<'a> Sdl2Painter<'a> {
    /// Creates a painter.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas to draw into.
    /// * `fonts` - The font cache used to render text.
    ///
    /// # Returns
    ///
    /// The new `Sdl2Painter`.
    pub fn new(canvas: &'a mut Canvas<Window>, fonts: &'a mut FontCache) -> Self {
        Self {
            canvas,
            fonts,
            errors: Vec::new(),
        }
    }

    /// Removes and returns the drawing failures since the last call.
    ///
    /// The [`Painter`] methods return nothing for shape and text drawing, so a failure is
    /// recorded here and the drawing is skipped.  The backend reports the errors to the engine as
    /// [`EngineEvent::Error`](pushrod::event::EngineEvent::Error).
    ///
    /// # Returns
    ///
    /// The errors, in the order they happened; empty if drawing succeeded.
    pub fn take_errors(&mut self) -> Vec<Error> {
        std::mem::take(&mut self.errors)
    }

    /// Records a failed drawing call as a texture error.
    ///
    /// # Arguments
    ///
    /// * `result` - What `SDL2` returned.
    fn check(&mut self, result: Result<(), String>) {
        if let Err(message) = result {
            self.errors.push(Error::texture(message));
        }
    }
}

impl Painter for Sdl2Painter<'_> {
    /// Returns the `SDL2` backend's cache policy.
    ///
    /// # Returns
    ///
    /// [`CachePolicy::TextureCache`].
    fn cache_policy(&self) -> CachePolicy {
        Self::CACHE_POLICY
    }

    /// Fills the whole canvas.
    ///
    /// # Arguments
    ///
    /// * `color` - The fill color.
    fn fill(&mut self, color: Color) {
        self.canvas.set_draw_color(to_sdl_color(color));
        self.canvas.clear();
    }

    /// Fills a rectangle.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The fill color.
    fn fill_rect(&mut self, bounds: Rect, color: Color) {
        self.canvas.set_draw_color(to_sdl_color(color));
        let result = self.canvas.fill_rect(to_sdl_rect(bounds));

        self.check(result);
    }

    /// Draws a rectangle outline.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The line color.
    fn rect(&mut self, bounds: Rect, color: Color) {
        self.canvas.set_draw_color(to_sdl_color(color));
        let result = self.canvas.draw_rect(to_sdl_rect(bounds));

        self.check(result);
    }

    /// Draws a line.
    ///
    /// # Arguments
    ///
    /// * `from` - The start point.
    /// * `to` - The end point.
    /// * `color` - The line color.
    fn line(&mut self, from: Point, to: Point, color: Color) {
        self.canvas.set_draw_color(to_sdl_color(color));
        let result = self
            .canvas
            .draw_line(SdlPoint::new(from.x, from.y), SdlPoint::new(to.x, to.y));

        self.check(result);
    }

    /// Draws text by copying glyphs from the font cache's atlas into the canvas.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to draw within.
    /// * `style` - The font and alignment.
    /// * `color` - The text color.
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// The size of the text drawn; zero if the font could not be loaded or the text could not be
    /// drawn (the failure is recorded, see [`Sdl2Painter::take_errors`]).
    fn text(&mut self, bounds: Rect, style: &TextStyle, color: Color, msg: &str) -> Size {
        let block = match self.fonts.layout(
            self.canvas,
            &style.font_name,
            style.font_size,
            to_sdl_font_style(style.font_style),
            msg,
            bounds.w,
        ) {
            Ok(block) => block,
            Err(error) => {
                self.errors.push(error);

                return Size::new(0, 0);
            }
        };
        let text_size = block.size;
        let dest = paint::align_text(bounds, text_size, style.justification);

        if let Err(error) = self.fonts.draw(
            self.canvas,
            &block,
            Point::new(dest.x, dest.y),
            to_sdl_color(color),
        ) {
            self.errors.push(error);
        }

        text_size
    }

    /// Returns the height of one line, as the font cache lays out a single line.
    ///
    /// # Arguments
    ///
    /// * `style` - The font.
    ///
    /// # Returns
    ///
    /// The line height in pixels.
    ///
    ///
    /// Zero if the font cannot be loaded (the failure is recorded, see
    /// [`Sdl2Painter::take_errors`]).
    fn line_height(&mut self, style: &TextStyle) -> u32 {
        match self.fonts.layout(
            self.canvas,
            &style.font_name,
            style.font_size,
            to_sdl_font_style(style.font_style),
            "A",
            u32::MAX,
        ) {
            Ok(block) => block.size.h,
            Err(error) => {
                self.errors.push(error);

                0
            }
        }
    }

    /// Measures one line of text with the font cache, without drawing it.
    ///
    /// # Arguments
    ///
    /// * `style` - The font.
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// The width in pixels.
    ///
    ///
    /// Zero if the font cannot be loaded (the failure is recorded, see
    /// [`Sdl2Painter::take_errors`]).
    fn text_width(&mut self, style: &TextStyle, msg: &str) -> u32 {
        match self.fonts.layout(
            self.canvas,
            &style.font_name,
            style.font_size,
            to_sdl_font_style(style.font_style),
            msg,
            u32::MAX,
        ) {
            Ok(block) => block.size.w,
            Err(error) => {
                self.errors.push(error);

                0
            }
        }
    }

    /// Draws an image file scaled to a rectangle.
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
    /// Returns [`Error::Image`] if the image cannot be loaded or copied.
    fn image(&mut self, path: &str, bounds: Rect) -> Result<(), Error> {
        let texture = self
            .canvas
            .texture_creator()
            .load_texture(path)
            .map_err(|message| Error::image(path, message))?;
        let result = self.canvas.copy(&texture, None, to_sdl_rect(bounds));

        unsafe { texture.destroy() };

        result.map_err(|message| Error::image(path, message))
    }

    /// Fills a rectangle with a checkerboard of two colors.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `first` - The corner pixel color.
    /// * `second` - The other color.
    fn dither(&mut self, bounds: Rect, first: Color, second: Color) {
        self.fill_rect(bounds, second);

        let points: Vec<SdlPoint> = (0..bounds.h as i32)
            .flat_map(|y| {
                (0..bounds.w as i32)
                    .filter(move |x| (x + y) % 2 == 0)
                    .map(move |x| SdlPoint::new(bounds.x + x, bounds.y + y))
            })
            .collect();

        self.canvas.set_draw_color(to_sdl_color(first));
        let result = self.canvas.draw_points(points.as_slice());

        self.check(result);
    }

    /// Blends a color over every other pixel of a rectangle, leaving the rest untouched.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The color to blend; its alpha sets the strength.
    fn dither_overlay(&mut self, bounds: Rect, color: Color) {
        let points: Vec<SdlPoint> = (0..bounds.h as i32)
            .flat_map(|y| {
                (0..bounds.w as i32)
                    .filter(move |x| (x + y) % 2 == 0)
                    .map(move |x| SdlPoint::new(bounds.x + x, bounds.y + y))
            })
            .collect();

        self.canvas.set_blend_mode(BlendMode::Blend);
        self.canvas.set_draw_color(to_sdl_color(color));
        let result = self.canvas.draw_points(points.as_slice());

        self.check(result);
        self.canvas.set_blend_mode(BlendMode::None);
    }

    /// Limits later drawing to a rectangle.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle, or `None` to remove the limit.
    fn clip(&mut self, bounds: Option<Rect>) {
        self.canvas.set_clip_rect(bounds.map(to_sdl_rect));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_color() {
        assert_eq!(
            to_sdl_color(Color::RGBA(1, 2, 3, 4)),
            SdlColor::RGBA(1, 2, 3, 4)
        );
    }

    #[test]
    fn converts_rect() {
        let r = to_sdl_rect(Rect::new(-1, 2, 3, 4));
        assert_eq!((r.x(), r.y(), r.width(), r.height()), (-1, 2, 3, 4));
    }

    #[test]
    fn converts_font_style() {
        assert_eq!(to_sdl_font_style(FontStyle::NORMAL), SdlFontStyle::NORMAL);
        assert_eq!(
            to_sdl_font_style(FontStyle::BOLD | FontStyle::UNDERLINE),
            SdlFontStyle::BOLD | SdlFontStyle::UNDERLINE
        );
        assert_eq!(to_sdl_font_style(FontStyle::ITALIC), SdlFontStyle::ITALIC);
        assert_eq!(
            to_sdl_font_style(FontStyle::STRIKETHROUGH),
            SdlFontStyle::STRIKETHROUGH
        );
    }

    #[test]
    fn sdl2_uses_texture_cache() {
        assert_eq!(Sdl2Painter::CACHE_POLICY, CachePolicy::TextureCache);
    }
}
