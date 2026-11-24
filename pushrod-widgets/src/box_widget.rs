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

use pushrod::font::FontCache;
use pushrod::geometry::{Point, Size};
use pushrod::paint;
use pushrod::widget::{Widget, WidgetCore};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// A filled rectangle with a border of any width and color.
pub struct BoxWidget {
    core: WidgetCore,
    border_color: Color,
    border_width: u8,
}

impl Widget for BoxWidget {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the background, then draws the border.
    fn paint(&self, c: &mut Canvas<Window>, _fc: &mut FontCache) {
        paint::fill(c, self.get_color());
        paint::border(c, self.get_size(), self.border_color, self.border_width);
    }
}

impl BoxWidget {
    /// Creates a new `BoxWidget` based on its point of origin, size, border color, and border width.
    /// Any borders with a width of 0 will not be drawn.
    pub fn new(origin: Point, size: Size, border_color: Color, border_width: u8) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            border_color,
            border_width,
        }
    }

    /// Sets the border width in pixels.
    pub fn set_border_width(&mut self, width: u8) {
        self.border_width = width;
        self.set_invalidated(true);
    }

    /// Sets the border color, which can be a `Color::RGB` or `Color::RGBA`.
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
        self.set_invalidated(true);
    }

    /// Retrieves the border width in pixels.
    pub fn get_border_width(&self) -> u8 {
        self.border_width
    }

    /// Retrieves the border color.
    pub fn get_border_color(&self) -> Color {
        self.border_color
    }
}
