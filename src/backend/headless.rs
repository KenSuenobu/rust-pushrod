// Headless Backend
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

//! # Headless backend
//!
//! A [`Painter`] that draws nothing and records every call, so a `Widget`'s `paint` can be unit
//! tested without a window or display.
//!
//! ```rust
//! use pushrod::backend::headless::{PaintCall, RecordingPainter};
//! use pushrod::color::Color;
//! use pushrod::paint;
//!
//! let mut p = RecordingPainter::default();
//! paint::fill(&mut p, Color::WHITE);
//! assert_eq!(p.calls, vec![PaintCall::Fill(Color::WHITE)]);
//! ```

use crate::color::Color;
use crate::geometry::{Point, Rect, Size};
use crate::paint::{CachePolicy, Painter, TextStyle};

/// One call made on a [`RecordingPainter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaintCall {
    /// `fill(color)`
    Fill(Color),
    /// `fill_rect(bounds, color)`
    FillRect(Rect, Color),
    /// `rect(bounds, color)`
    Rect(Rect, Color),
    /// `line(from, to, color)`
    Line(Point, Point, Color),
    /// `text(bounds, msg, color)`
    Text(Rect, String, Color),
    /// `image(path, bounds)`
    Image(String, Rect),
    /// `dither(bounds, first, second)`
    Dither(Rect, Color, Color),
    /// `dither_overlay(bounds, color)`
    DitherOverlay(Rect, Color),
    /// `clip(bounds)`
    Clip(Option<Rect>),
}

/// A [`Painter`] that records its calls in [`calls`](RecordingPainter::calls).
#[derive(Debug, Default)]
pub struct RecordingPainter {
    /// Every call made, in order.
    pub calls: Vec<PaintCall>,
}

impl Painter for RecordingPainter {
    /// Returns the policy of a recording painter.
    ///
    /// # Returns
    ///
    /// [`CachePolicy::DirectDraw`], since nothing is cached.
    fn cache_policy(&self) -> CachePolicy {
        CachePolicy::DirectDraw
    }

    /// Records a fill.
    ///
    /// # Arguments
    ///
    /// * `color` - The fill color.
    fn fill(&mut self, color: Color) {
        self.calls.push(PaintCall::Fill(color));
    }

    /// Records a rectangle fill.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The fill color.
    fn fill_rect(&mut self, bounds: Rect, color: Color) {
        self.calls.push(PaintCall::FillRect(bounds, color));
    }

    /// Records a rectangle outline.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The line color.
    fn rect(&mut self, bounds: Rect, color: Color) {
        self.calls.push(PaintCall::Rect(bounds, color));
    }

    /// Records a line.
    ///
    /// # Arguments
    ///
    /// * `from` - The start point.
    /// * `to` - The end point.
    /// * `color` - The line color.
    fn line(&mut self, from: Point, to: Point, color: Color) {
        self.calls.push(PaintCall::Line(from, to, color));
    }

    /// Records text.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to draw within.
    /// * `style` - The font (its size sets the recorded text height).
    /// * `color` - The text color.
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// A deterministic size: half the font size per character wide (at most the width of `bounds`),
    /// one font size tall.
    fn text(&mut self, bounds: Rect, style: &TextStyle, color: Color, msg: &str) -> Size {
        self.calls
            .push(PaintCall::Text(bounds, msg.to_string(), color));

        let width = (msg.chars().count() as u32 * (style.font_size as u32 / 2)).min(bounds.w);

        Size::new(width, style.font_size as u32)
    }

    /// Measures text the way [`text`](Painter::text) sizes it, without recording a call.
    ///
    /// # Arguments
    ///
    /// * `style` - The font (its size sets the width of each character).
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// Half the font size per character.
    fn text_width(&mut self, style: &TextStyle, msg: &str) -> u32 {
        msg.chars().count() as u32 * (style.font_size as u32 / 2)
    }

    /// Records an image.
    ///
    /// # Arguments
    ///
    /// * `path` - The image path.
    /// * `bounds` - The rectangle.
    ///
    /// # Returns
    ///
    /// `Ok(())`, always.
    ///
    /// # Errors
    ///
    /// Never fails.
    fn image(&mut self, path: &str, bounds: Rect) -> Result<(), String> {
        self.calls.push(PaintCall::Image(path.to_string(), bounds));
        Ok(())
    }

    /// Records a dither.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `first` - The corner pixel color.
    /// * `second` - The other color.
    fn dither(&mut self, bounds: Rect, first: Color, second: Color) {
        self.calls.push(PaintCall::Dither(bounds, first, second));
    }

    /// Records a dither overlay.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The color to blend.
    fn dither_overlay(&mut self, bounds: Rect, color: Color) {
        self.calls.push(PaintCall::DitherOverlay(bounds, color));
    }

    /// Records a clip change.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The clip rectangle, or `None`.
    fn clip(&mut self, bounds: Option<Rect>) {
        self.calls.push(PaintCall::Clip(bounds));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::FontStyle;
    use crate::paint::TextJustify;

    #[test]
    fn records_every_call_in_order() {
        let mut p = RecordingPainter::default();
        let r = Rect::new(1, 2, 3, 4);

        p.fill(Color::BLACK);
        p.fill_rect(r, Color::RED);
        p.line(Point::new(0, 0), Point::new(5, 5), Color::BLUE);
        p.dither(r, Color::BLACK, Color::WHITE);
        p.clip(Some(r));
        p.clip(None);
        assert!(p.image("a.png", r).is_ok());

        assert_eq!(p.calls.len(), 7);
        assert_eq!(p.calls[0], PaintCall::Fill(Color::BLACK));
        assert_eq!(p.calls[5], PaintCall::Clip(None));
        assert_eq!(p.calls[6], PaintCall::Image(String::from("a.png"), r));
    }

    #[test]
    fn text_size_is_deterministic_and_clamped() {
        let mut p = RecordingPainter::default();
        let style = TextStyle::new(
            String::from("f.ttf"),
            FontStyle::NORMAL,
            10,
            Color::BLACK,
            TextJustify::Left,
        );

        assert_eq!(
            p.text(Rect::new(0, 0, 100, 20), &style, Color::BLACK, "abcd"),
            Size::new(20, 10)
        );
        assert_eq!(
            p.text(Rect::new(0, 0, 8, 20), &style, Color::BLACK, "abcd"),
            Size::new(8, 10)
        );
    }

    #[test]
    fn policy_is_direct_draw() {
        assert_eq!(
            RecordingPainter::default().cache_policy(),
            CachePolicy::DirectDraw
        );
    }
}
