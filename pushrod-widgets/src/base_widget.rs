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

//! # BaseWidget
//!
//! This is a default widget that can be used as a top-level canvas.  It is a basic `Widget`
//! implementation that fills its background with the desired color.

use pushrod::font::FontCache;
use pushrod::geometry::{Point, Size};
use pushrod::paint;
use pushrod::widget::{Widget, WidgetCore};
use sdl2::render::Canvas;
use sdl2::video::Window;

/// A rectangle filled with one color.
///
/// Use it as a background, or as a panel that holds other `Widget`s (add them with
/// `Engine::add_child_widget`).
///
/// # Examples
///
/// ```rust
/// use pushrod::geometry::{point, size};
/// use pushrod::widget::Widget;
/// use pushrod_widgets::base_widget::BaseWidget;
/// use sdl2::pixels::Color;
///
/// let mut panel = BaseWidget::new(point(10, 10), size(200, 100));
/// panel.set_color(Color::RGB(220, 220, 220));
/// ```
pub struct BaseWidget {
    core: WidgetCore,
}

impl Widget for BaseWidget {
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
    /// * `c` - The canvas, targeting the panel's texture.
    /// * `_fc` - The font cache (unused).
    fn paint(&self, c: &mut Canvas<Window>, _fc: &mut FontCache) {
        paint::fill(c, self.get_color());
    }
}

impl BaseWidget {
    /// Creates a panel.  Its color is transparent until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent `Widget`.
    /// * `size` - The width and height in pixels.
    ///
    /// # Returns
    ///
    /// The new `BaseWidget`.
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
        }
    }
}
