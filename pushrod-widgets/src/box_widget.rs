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

//! # BoxWidget
//!
//! This is a simple system `Widget` that fills its background like the `BaseWidget`, and draws a
//! border of a specific width and color on top of it.

use pushrod::color::Color;
use pushrod::geometry::{Point, Size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};

/// A filled rectangle with a border of any width and color.
///
/// # Examples
///
/// ```rust
/// use pushrod::geometry::{point, size};
/// use pushrod_widgets::box_widget::BoxWidget;
/// use pushrod::color::Color;
///
/// let frame = BoxWidget::new(point(20, 40), size(100, 100), Color::BLUE, 3);
/// ```
pub struct BoxWidget {
    core: WidgetCore,
    border_color: Color,
    border_width: u8,
}

impl Widget for BoxWidget {
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
        paint::fill(p, self.get_color());
        paint::border(p, self.get_size(), self.border_color, self.border_width);
    }
}

impl BoxWidget {
    /// Creates a box.  Its fill color is transparent until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent `Widget`.
    /// * `size` - The width and height in pixels.
    /// * `border_color` - The border color.
    /// * `border_width` - The border width in pixels; `0` draws no border.
    ///
    /// # Returns
    ///
    /// The new `BoxWidget`.
    pub fn new(origin: Point, size: Size, border_color: Color, border_width: u8) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            border_color,
            border_width,
        }
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
