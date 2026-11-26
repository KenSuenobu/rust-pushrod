// Box Widget
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

//! # Frame
//!
//! This is a simple system `Widget` that fills its background like the `Panel`, and draws a
//! border of a specific width and color on top of it.

use pushrod::color::Color;
use pushrod::geometry::{point, size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};

/// A filled rectangle with a border of any width and color.
///
/// # Examples
///
/// ```rust
/// use pushrod::color::Color;
/// use pushrod_widgets::frame::Frame;
///
/// let frame = Frame::new()
///     .at(20, 40)
///     .size(100, 100)
///     .border_color(Color::BLUE)
///     .border_width(3);
/// ```
#[derive(Debug)]
pub struct Frame {
    core: WidgetCore,
    border_color: Color,
    border_width: u8,
}

impl Widget for Frame {
    /// Returns the box's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the box's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the box with its color, then draws the border inside its edges.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the box.
    fn paint(&self, p: &mut dyn Painter) {
        let style = self.style();

        paint::fill(p, style.background.unwrap_or(self.get_color()));
        paint::border(
            p,
            self.get_size(),
            style.border_color.unwrap_or(self.border_color),
            style.border_width.unwrap_or(self.border_width),
        );
    }
}

impl Frame {
    /// Creates a frame, 100 by 100 pixels at the upper left-hand corner of its parent, with a
    /// black one pixel border.  Its fill color is transparent until set with `set_color`.
    ///
    /// # Returns
    ///
    /// The new `Frame`.
    pub fn new() -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(100, 100)),
            border_color: Color::BLACK,
            border_width: 1,
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
    /// The frame, for chaining.
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
    /// The frame, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets the border color.
    ///
    /// # Arguments
    ///
    /// * `color` - The border color.
    ///
    /// # Returns
    ///
    /// The frame, for chaining.
    #[must_use]
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Sets the border width.
    ///
    /// # Arguments
    ///
    /// * `width` - The border width in pixels; `0` draws no border.
    ///
    /// # Returns
    ///
    /// The frame, for chaining.
    #[must_use]
    pub fn border_width(mut self, width: u8) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the border width, and invalidates the box.
    ///
    /// # Arguments
    ///
    /// * `width` - The border width in pixels; `0` draws no border.
    pub fn set_border_width(&mut self, width: u8) {
        self.border_width = width;
        self.set_invalidated(true);
    }

    /// Sets the border color, and invalidates the box.
    ///
    /// # Arguments
    ///
    /// * `color` - The border color.
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
        self.set_invalidated(true);
    }

    /// Returns the border width.
    ///
    /// # Returns
    ///
    /// The border width in pixels.
    pub fn get_border_width(&self) -> u8 {
        self.border_width
    }

    /// Returns the border color.
    ///
    /// # Returns
    ///
    /// The border color.
    pub fn get_border_color(&self) -> Color {
        self.border_color
    }
}

impl Default for Frame {
    /// Creates a frame with [`Frame::new`].
    ///
    /// # Returns
    ///
    /// The new `Frame`.
    fn default() -> Self {
        Self::new()
    }
}
