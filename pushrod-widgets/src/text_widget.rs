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

use pushrod::color::{Color, FontStyle};
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::{self, Painter, TextStyle};
use pushrod::widget::{Widget, WidgetCore};

pub use pushrod::paint::TextJustify;

/// A string of text on a filled background.
///
/// The text is aligned horizontally by its justification, wraps at the widget's width, and is
/// centered vertically when it fits.  It doesn't respond to input.
pub struct TextWidget {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
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

        paint::fill(p, self.get_color());
        paint::text(
            p,
            Rect::new(0, 0, size.w, size.h),
            &self.style,
            self.style.font_color,
            &self.msg,
        );
    }
}

impl TextWidget {
    /// Creates a text widget.  Its background is transparent until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent `Widget`.
    /// * `size` - The width and height in pixels.
    /// * `font_name` - Path to the TrueType font file.
    /// * `font_style` - The font style (normal, bold, italic, ...).
    /// * `font_size` - The font size in points.
    /// * `font_color` - The text color.
    /// * `justification` - The horizontal alignment of the text.
    /// * `msg` - The text to display.
    ///
    /// # Returns
    ///
    /// The new `TextWidget`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: Point,
        size: Size,
        font_name: String,
        font_style: FontStyle,
        font_size: u16,
        font_color: Color,
        justification: TextJustify,
        msg: String,
    ) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            style: TextStyle::new(font_name, font_style, font_size, font_color, justification),
            msg,
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
}
