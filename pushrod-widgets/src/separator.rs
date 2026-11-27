// Separator Widget
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

//! # Separator
//!
//! A display-only divider: a one pixel horizontal or vertical line, optionally etched as two
//! lines (a dark line beside a white one) so it reads as a groove.

use pushrod::color::Color;
use pushrod::error::Result;
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::properties::{
    Property, core_properties, expect_bool, expect_color, read_only, set_core_property, track,
};
use pushrod::registry::PropertyValue;
use pushrod::widget::{Widget, WidgetCore};

/// Which way the line runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
}

/// A thin divider line.  It has an empty event mask, so it receives no input and emits no
/// events.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::separator::Separator;
///
/// let rule = Separator::horizontal(200).at(20, 60);
/// let groove = Separator::vertical(80).etched().at(240, 20);
/// ```
#[derive(Debug)]
pub struct Separator {
    core: WidgetCore,
    orientation: Orientation,
    color: Color,
    etched: bool,
}

impl Widget for Separator {
    /// Returns the separator's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the separator's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Lists the widget's properties: the core properties, then its own.
    ///
    /// # Returns
    ///
    /// The properties.
    fn properties(&self) -> Vec<Property> {
        let mut list = core_properties(self);

        list.push(Property::bool("etched", self.is_etched()));
        list.push(Property::color("line_color", self.color));
        list.push(Property::bool("vertical", !self.is_horizontal()));

        list
    }

    /// Sets a property by name; the widget's own, or else a core property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed, `false` if the widget already had it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Property`](pushrod::error::Error::Property) if there is no such
    /// property, it is read only, or the value is the wrong kind or out of range.
    fn set_property(&mut self, name: &str, value: &PropertyValue) -> Result<bool> {
        match name {
            "etched" => {
                let b = expect_bool(name, value)?;

                track(self, |w| w.set_etched(b))
            }
            "line_color" => {
                let c = expect_color(name, value)?;

                track(self, |w| w.set_line_color(c))
            }
            "vertical" => Err(read_only(name)),
            _ => set_core_property(self, name, value),
        }
    }

    /// Draws the line (and, when etched, the white line beside it).
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let style = self.style();
        let color = style.foreground.unwrap_or(self.color);
        let last_x = size.w.saturating_sub(1) as i32;
        let last_y = size.h.saturating_sub(1) as i32;

        if size.w == 0 || size.h == 0 {
            return;
        }

        match self.orientation {
            Orientation::Horizontal => {
                p.line(point(0, 0), point(last_x, 0), color);

                if self.etched && size.h > 1 {
                    p.line(point(0, 1), point(last_x, 1), Color::WHITE);
                }
            }
            Orientation::Vertical => {
                p.line(point(0, 0), point(0, last_y), color);

                if self.etched && size.w > 1 {
                    p.line(point(1, 0), point(1, last_y), Color::WHITE);
                }
            }
        }
    }
}

impl Separator {
    /// Creates a one pixel high horizontal line at the upper left-hand corner of its parent,
    /// drawn in gray.
    ///
    /// # Arguments
    ///
    /// * `width` - The length of the line in pixels.
    ///
    /// # Returns
    ///
    /// The new `Separator`.
    pub fn horizontal(width: u32) -> Self {
        Self::build(Orientation::Horizontal, size(width, 1))
    }

    /// Creates a one pixel wide vertical line at the upper left-hand corner of its parent,
    /// drawn in gray.
    ///
    /// # Arguments
    ///
    /// * `height` - The length of the line in pixels.
    ///
    /// # Returns
    ///
    /// The new `Separator`.
    pub fn vertical(height: u32) -> Self {
        Self::build(Orientation::Vertical, size(1, height))
    }

    /// Builds a separator of the given orientation and size.
    fn build(orientation: Orientation, size: pushrod::geometry::Size) -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size),
            orientation,
            color: Color::RGBA(64, 64, 64, 255),
            etched: false,
        }
    }

    /// Sets the upper left-hand corner, relative to the parent.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The separator, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the line color.
    ///
    /// # Arguments
    ///
    /// * `color` - The line color.
    ///
    /// # Returns
    ///
    /// The separator, for chaining.
    #[must_use]
    pub fn line_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Etches the separator: the widget grows to two pixels thick, drawing the line beside a
    /// white line.
    ///
    /// # Returns
    ///
    /// The separator, for chaining.
    #[must_use]
    pub fn etched(mut self) -> Self {
        self.set_etched(true);
        self
    }

    /// Turns the etched look on or off, resizes the widget to match (one pixel thick, or two
    /// when etched), and invalidates it.
    ///
    /// # Arguments
    ///
    /// * `etched` - `true` for two lines, `false` for one.
    pub fn set_etched(&mut self, etched: bool) {
        let thickness = if etched { 2 } else { 1 };
        let current = self.get_size();

        self.etched = etched;
        self.set_size(match self.orientation {
            Orientation::Horizontal => size(current.w, thickness),
            Orientation::Vertical => size(thickness, current.h),
        });
        self.set_invalidated(true);
    }

    /// Returns whether the separator is etched.
    ///
    /// # Returns
    ///
    /// `true` when drawn as two lines.
    pub fn is_etched(&self) -> bool {
        self.etched
    }

    /// Returns whether the separator runs horizontally.
    ///
    /// # Returns
    ///
    /// `true` for a horizontal line, `false` for a vertical one.
    pub fn is_horizontal(&self) -> bool {
        self.orientation == Orientation::Horizontal
    }

    /// Sets the line color, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `color` - The line color.
    pub fn set_line_color(&mut self, color: Color) {
        self.color = color;
        self.set_invalidated(true);
    }
}
