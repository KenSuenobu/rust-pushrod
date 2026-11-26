// Base Widget
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

//! # Panel
//!
//! This is a default widget that can be used as a top-level canvas.  It is a basic `Widget`
//! implementation that fills its background with the desired color.

use pushrod::geometry::{point, size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};

/// A rectangle filled with one color.
///
/// Use it as a background, or as a panel that holds other `Widget`s (add them with
/// `Engine::add_child_widget`).
///
/// # Examples
///
/// ```rust
/// use pushrod::widget::Widget;
/// use pushrod_widgets::panel::Panel;
/// use pushrod::color::Color;
///
/// let mut panel = Panel::new().at(10, 10).size(200, 100);
/// panel.set_color(Color::RGB(220, 220, 220));
/// ```
#[derive(Debug)]
pub struct Panel {
    core: WidgetCore,
}

impl Widget for Panel {
    /// Returns the panel's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the panel's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the panel with its color.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the panel.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.style().background.unwrap_or(self.get_color()));
    }
}

impl Panel {
    /// Creates a panel, 100 by 100 pixels at the upper left-hand corner of its parent.  Its color
    /// is transparent until set with `set_color`.
    ///
    /// # Returns
    ///
    /// The new `Panel`.
    pub fn new() -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(100, 100)),
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
    /// The panel, for chaining.
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
    /// The panel, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }
}

impl Default for Panel {
    /// Creates a panel with [`Panel::new`].
    ///
    /// # Returns
    ///
    /// The new `Panel`.
    fn default() -> Self {
        Self::new()
    }
}
