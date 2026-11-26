// Text Widget
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

//! # Text
//!
//! This is a simple system `Widget` that fills its background like the `Panel`, and draws a
//! text message with a given font name, style, color, and message on top of it.

use pushrod::color::Color;
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::paint::{self, Painter, TextStyle, Truncate, Wrap};
use pushrod::widget::{Widget, WidgetCore};
use std::cell::Cell;

pub use pushrod::paint::TextJustify;

/// A string of text on a filled background.
///
/// The text is aligned horizontally by its justification and centered vertically when it fits.
/// Text wider than the widget is shortened with an ellipsis as set by [`Text::set_truncate`]
/// (the end, by default), or, with [`Text::set_wrap`], broken into lines (each aligned by
/// the justification) that reflow when the width changes; [`Text::set_max_lines`] limits
/// them.  It doesn't respond to input.
#[derive(Debug)]
pub struct Text {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
    truncate: Truncate,
    wrap: Wrap,
    max_lines: Option<usize>,
    truncated: Cell<bool>,
}

impl Widget for Text {
    /// Returns the text widget's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the text widget's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the background with the widget's color, then draws the text.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let style = self.style();

        paint::fill(p, style.background.unwrap_or(self.get_color()));
        let bounds = Rect::new(0, 0, size.w, size.h);
        let font = style.font.as_ref().unwrap_or(&self.style);
        let color = style.foreground.unwrap_or(self.style.font_color);
        let drawn = if self.wrap == Wrap::None {
            paint::text(p, bounds, font, color, &self.msg, self.truncate)
        } else {
            paint::paragraph(p, bounds, font, color, &self.msg, self.wrap, self.max_lines)
        };

        self.truncated.set(drawn.truncated);
    }
}

impl Text {
    /// Creates a text widget with sensible defaults.
    ///
    /// It is 200 by 20 pixels at the upper left-hand corner of its parent, with the engine's
    /// default font, black left-justified text, and no wrapping.  Its background is transparent
    /// until set with `set_color`.  Change the defaults with the chained setters:
    ///
    /// ```rust
    /// use pushrod_widgets::text::Text;
    ///
    /// let label = Text::new("Not clicked").at(20, 20).size(200, 20);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `msg` - The text to display.
    ///
    /// # Returns
    ///
    /// The new `Text`.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(200, 20)),
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            msg: msg.into(),
            truncate: Truncate::End,
            wrap: Wrap::None,
            max_lines: None,
            truncated: Cell::new(false),
        }
    }

    /// Sets the upper left-hand corner, relative to the parent `Widget`.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The text widget, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The text widget, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets the font.
    ///
    /// # Arguments
    ///
    /// * `font` - Sets the font.
    ///
    /// # Returns
    ///
    /// The text widget, for chaining.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Sets the text color.
    ///
    /// # Arguments
    ///
    /// * `color` - Sets the text color.
    ///
    /// # Returns
    ///
    /// The text widget, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
        self
    }

    /// Sets the horizontal alignment.
    ///
    /// # Arguments
    ///
    /// * `justification` - Sets the horizontal alignment.
    ///
    /// # Returns
    ///
    /// The text widget, for chaining.
    #[must_use]
    pub fn justify(mut self, justification: TextJustify) -> Self {
        self.style.justification = justification;
        self
    }

    /// Sets the text color, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `color` - The text color.
    pub fn set_font_color(&mut self, color: Color) {
        self.style.font_color = color;
        self.set_invalidated(true);
    }

    /// Sets the text to display, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `msg` - The new text.
    pub fn set_text(&mut self, msg: String) {
        self.msg = msg;
        self.set_invalidated(true);
    }

    /// Returns the text color.
    ///
    /// # Returns
    ///
    /// The text color.
    pub fn get_font_color(&self) -> Color {
        self.style.font_color
    }

    /// Returns the text being displayed.
    ///
    /// # Returns
    ///
    /// A copy of the text.
    pub fn get_text(&self) -> String {
        self.msg.clone()
    }

    /// Sets how text that doesn't fit is shortened, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `truncate` - The mode; the default is [`Truncate::End`].
    pub fn set_truncate(&mut self, truncate: Truncate) {
        if self.truncate != truncate {
            self.truncate = truncate;
            self.set_invalidated(true);
        }
    }

    /// Returns how text that doesn't fit is shortened.
    ///
    /// # Returns
    ///
    /// The truncation mode.
    pub fn get_truncate(&self) -> Truncate {
        self.truncate
    }

    /// Returns whether the text was shortened the last time the widget painted, so a tooltip can
    /// show the full text.
    ///
    /// # Returns
    ///
    /// `true` if the last paint drew a shortened text; `false` before the first paint.
    pub fn is_truncated(&self) -> bool {
        self.truncated.get()
    }

    /// Sets how text is broken into lines, and invalidates the widget.  With wrapping on,
    /// `truncate` is not used: only `max_lines` shortens the text.
    ///
    /// # Arguments
    ///
    /// * `wrap` - The mode; the default is [`Wrap::None`] (one line).
    pub fn set_wrap(&mut self, wrap: Wrap) {
        if self.wrap != wrap {
            self.wrap = wrap;
            self.set_invalidated(true);
        }
    }

    /// Returns how text is broken into lines.
    ///
    /// # Returns
    ///
    /// The wrap mode.
    pub fn get_wrap(&self) -> Wrap {
        self.wrap
    }

    /// Sets the most lines a wrapped text may use, and invalidates the widget.  If the text needs
    /// more, the last line ends in an ellipsis.  Has no effect while the wrap mode is
    /// [`Wrap::None`].
    ///
    /// # Arguments
    ///
    /// * `max_lines` - The limit (at least one), or `None` for no limit.
    pub fn set_max_lines(&mut self, max_lines: Option<usize>) {
        if self.max_lines != max_lines {
            self.max_lines = max_lines;
            self.set_invalidated(true);
        }
    }

    /// Returns the line limit.
    ///
    /// # Returns
    ///
    /// The most lines, or `None` for no limit.
    pub fn get_max_lines(&self) -> Option<usize> {
        self.max_lines
    }

    /// Returns the height the text needs at a width, for layout managers so a wrapped label can
    /// push the widgets below it down.
    ///
    /// # Arguments
    ///
    /// * `p` - A painter, used to measure the text.
    /// * `width` - The width in pixels.
    ///
    /// # Returns
    ///
    /// The height in pixels: the number of lines times the line height.
    pub fn height_for_width(&self, p: &mut dyn Painter, width: u32) -> u32 {
        paint::height_for_width(p, &self.style, &self.msg, width, self.wrap, self.max_lines)
    }
}
