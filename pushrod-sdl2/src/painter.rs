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
//!
//! On a high-DPI window the painter is given the window's scale factor
//! ([`Sdl2Painter::with_scale`]): every logical coordinate is converted to physical pixels, text
//! is rasterized at the scaled font size, and one-pixel lines become whole physical pixels thick
//! (see [`crate::scale`]).  At 1x every call draws exactly as it would without scaling.

use crate::font::FontCache;
use crate::image_cache::ImageCache;
use crate::scale::{diagonal_offsets, dither_cells, outline_bars, scaled_font_size, straight_line};
use pushrod::color::{Color, FontStyle};
use pushrod::error::Error;
use pushrod::geometry::{
    Point, Rect, Size, sanitize_scale, stroke_width, to_logical_length, to_physical,
    to_physical_point,
};
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
    images: Option<&'a mut ImageCache>,
    errors: Vec<Error>,
    /// Physical pixels per logical pixel.
    scale: f32,
}

impl<'a> std::fmt::Debug for Sdl2Painter<'a> {
    /// Formats the value for debugging.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sdl2Painter").finish_non_exhaustive()
    }
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
            images: None,
            errors: Vec::new(),
            scale: 1.0,
        }
    }

    /// Sets the scale factor: how many physical pixels of the canvas make one logical pixel.
    ///
    /// # Arguments
    ///
    /// * `scale` - The window's scale factor; an unusable value (zero, negative, not finite)
    ///   reads as `1.0`.
    ///
    /// # Returns
    ///
    /// The painter, for chaining.
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = sanitize_scale(scale);
        self
    }

    /// Returns whether the painter draws at 1x, where logical and physical pixels are the same.
    ///
    /// # Returns
    ///
    /// `true` at a scale factor of exactly `1.0`.
    fn is_unscaled(&self) -> bool {
        self.scale == 1.0
    }

    /// Returns a text style whose font is rasterized at physical resolution.
    ///
    /// # Arguments
    ///
    /// * `style` - The style the `Widget` asked for.
    ///
    /// # Returns
    ///
    /// The style with its font size multiplied by the scale.
    fn physical_style(&self, style: &TextStyle) -> TextStyle {
        let mut scaled = style.clone();

        scaled.font = style
            .font
            .with_size(scaled_font_size(style.font.size(), self.scale));
        scaled
    }

    /// Fills physical rectangles with a color, recording any failure.
    ///
    /// # Arguments
    ///
    /// * `rects` - The rectangles, in physical pixels.
    /// * `color` - The fill color.
    fn fill_physical(&mut self, rects: &[Rect], color: Color) {
        let rects: Vec<SdlRect> = rects.iter().map(|r| to_sdl_rect(*r)).collect();

        self.canvas.set_draw_color(to_sdl_color(color));
        let result = self.canvas.fill_rects(&rects);

        self.check(result);
    }

    /// Gives the painter the shared image cache used by [`Painter::image_size`] and
    /// [`Painter::image_part`].  Without one those calls fail with [`Error::Image`].
    ///
    /// # Arguments
    ///
    /// * `images` - The cache shared by every `Widget`.
    ///
    /// # Returns
    ///
    /// The painter, for chaining.
    #[must_use]
    pub fn with_images(mut self, images: &'a mut ImageCache) -> Self {
        self.images = Some(images);
        self
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

    /// Returns the scale factor the painter draws at.
    ///
    /// # Returns
    ///
    /// Physical pixels per logical pixel; `1.0` unless [`Sdl2Painter::with_scale`] set one.
    fn scale_factor(&self) -> f32 {
        self.scale
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
        let result = self
            .canvas
            .fill_rect(to_sdl_rect(to_physical(bounds, self.scale)));

        self.check(result);
    }

    /// Draws a rectangle outline.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The line color.
    fn rect(&mut self, bounds: Rect, color: Color) {
        if !self.is_unscaled() {
            let bars = outline_bars(to_physical(bounds, self.scale), stroke_width(self.scale));

            self.fill_physical(&bars, color);

            return;
        }

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
        if !self.is_unscaled() {
            if let Some(bar) = straight_line(from, to, self.scale) {
                self.fill_physical(&[bar], color);

                return;
            }

            let (a, b) = (
                to_physical_point(from, self.scale),
                to_physical_point(to, self.scale),
            );

            self.canvas.set_draw_color(to_sdl_color(color));

            for offset in diagonal_offsets(a, b, self.scale) {
                let result = self.canvas.draw_line(
                    SdlPoint::new(a.x + offset.x, a.y + offset.y),
                    SdlPoint::new(b.x + offset.x, b.y + offset.y),
                );

                self.check(result);
            }

            return;
        }

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
        let scale = self.scale;
        let style = &self.physical_style(style);
        let bounds = to_physical(bounds, scale);
        let block = match self.fonts.layout(self.canvas, &style.font, msg, bounds.w) {
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

        Size::new(
            to_logical_length(text_size.w, scale),
            to_logical_length(text_size.h, scale),
        )
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
        let style = self.physical_style(style);

        match self.fonts.layout(self.canvas, &style.font, "A", u32::MAX) {
            Ok(block) => to_logical_length(block.size.h, self.scale),
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
        let style = self.physical_style(style);

        match self.fonts.layout(self.canvas, &style.font, msg, u32::MAX) {
            Ok(block) => to_logical_length(block.size.w, self.scale),
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
        let result = self
            .canvas
            .copy(&texture, None, to_sdl_rect(to_physical(bounds, self.scale)));

        unsafe { texture.destroy() };

        result.map_err(|message| Error::image(path, message))
    }

    /// Returns an image's size, loading it into the shared image cache on first use.  A failure
    /// is also recorded for the backend to report as an engine error event.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file.
    ///
    /// # Returns
    ///
    /// The image's size in pixels.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] if there is no image cache or the file cannot be loaded.
    fn image_size(&mut self, path: &str) -> Result<Size, Error> {
        let result = match self.images.as_deref_mut() {
            Some(images) => images.load(self.canvas, path),
            None => Err(Error::image(path, "no image cache")),
        };

        if let Err(error) = &result {
            self.errors.push(error.clone());
        }

        result
    }

    /// Copies part of a cached image texture into a rectangle.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file.
    /// * `source` - The part of the image to copy, in image pixels; `None` copies all of it.
    /// * `bounds` - The rectangle to draw into.
    ///
    /// # Returns
    ///
    /// `Ok(())` once drawn.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] if there is no image cache or the file cannot be loaded or drawn.
    fn image_part(&mut self, path: &str, source: Option<Rect>, bounds: Rect) -> Result<(), Error> {
        let Some(images) = self.images.as_deref_mut() else {
            return Err(Error::image(path, "no image cache"));
        };

        images.load(self.canvas, path)?;

        let texture = images
            .texture(path)
            .ok_or_else(|| Error::image(path, "image not cached"))?;

        self.canvas
            .copy(
                texture,
                source.map(to_sdl_rect),
                to_sdl_rect(to_physical(bounds, self.scale)),
            )
            .map_err(|message| Error::image(path, message))
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

        if !self.is_unscaled() {
            self.fill_physical(&dither_cells(bounds, self.scale), first);

            return;
        }

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
        if !self.is_unscaled() {
            self.canvas.set_blend_mode(BlendMode::Blend);
            self.fill_physical(&dither_cells(bounds, self.scale), color);
            self.canvas.set_blend_mode(BlendMode::None);

            return;
        }

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
        let scale = self.scale;

        self.canvas
            .set_clip_rect(bounds.map(|b| to_sdl_rect(to_physical(b, scale))));
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
