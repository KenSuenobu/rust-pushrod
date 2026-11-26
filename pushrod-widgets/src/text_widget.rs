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

//! # TextWidget
//!
//! This is a simple system `Widget` that fills its background like the `BaseWidget`, and draws a
//! text message with a given font name, style, color, and message on top of it.

use pushrod::color::Color;
use pushrod::font::Font;
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::{self, Painter, TextStyle, Truncate, Wrap};
use pushrod::widget::{Widget, WidgetCore};
use std::cell::Cell;

pub use pushrod::paint::TextJustify;

/// A string of text on a filled background.
///
/// The text is aligned horizontally by its justification and centered vertically when it fits.
/// Text wider than the widget is shortened with an ellipsis as set by [`TextWidget::set_truncate`]
/// (the end, by default), or, with [`TextWidget::set_wrap`], broken into lines (each aligned by
/// the justification) that reflow when the width changes; [`TextWidget::set_max_lines`] limits
/// them.  It doesn't respond to input.
pub struct TextWidget {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
    truncate: Truncate,
    wrap: Wrap,
    max_lines: Option<usize>,
    truncated: Cell<bool>,
}

impl Widget for TextWidget {
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

impl TextWidget {
    /// Creates a text widget.  Its background is transparent until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent `Widget`.
    /// * `size` - The width and height in pixels.
    /// * `font` - The font handle: a registered family, size, and style.  `Font::default()` is
    ///   the engine's default font, which needs no files.
    /// * `font_color` - The text color.
    /// * `justification` - The horizontal alignment of the text.
    /// * `msg` - The text to display.
    ///
    /// # Returns
    ///
    /// The new `TextWidget`.
    pub fn new(
        origin: Point,
        size: Size,
        font: Font,
        font_color: Color,
        justification: TextJustify,
        msg: String,
    ) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            style: TextStyle::new(font, font_color, justification),
            msg,
            truncate: Truncate::End,
            wrap: Wrap::None,
            max_lines: None,
            truncated: Cell::new(false),
        }
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
