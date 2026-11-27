// BoxedLabel Widget
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

//! # BoxedLabel
//!
//! A display-only label: a fill, centered text, and a border, composed from the core's
//! `paint::fill`, `paint::text`, and `paint::border` helpers.  It is the reference example of
//! composing paint helpers into a widget.

use pushrod::color::Color;
use pushrod::error::Result;
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::properties::{
    Property, core_properties, expect_color, expect_text, set_core_property, track,
};
use pushrod::registry::PropertyValue;
use pushrod::widget::{Widget, WidgetCore};

/// Text centered in a filled box with a border.  Text wider than the box is shortened with an
/// ellipsis.  It has an empty event mask, so it receives no input and emits no events.
///
/// # Examples
///
/// ```rust
/// use pushrod::color::Color;
/// use pushrod_widgets::boxed_label::BoxedLabel;
///
/// let label = BoxedLabel::new("Ready").at(20, 20).size(120, 24).border_color(Color::BLUE);
/// ```
#[derive(Debug)]
pub struct BoxedLabel {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
    border_color: Color,
    border_width: u8,
}

impl Widget for BoxedLabel {
    /// Returns the label's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the label's shared state for modification.
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

        list.push(Property::text("text", self.get_text()));
        list.push(Property::color("border_color", self.border_color));

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
            "text" => {
                let t = expect_text(name, value)?.to_string();

                track(self, |w| w.set_text(t))
            }
            "border_color" => {
                let c = expect_color(name, value)?;

                track(self, |w| w.set_border_color(c))
            }
            _ => set_core_property(self, name, value),
        }
    }

    /// Fills the background, draws the centered text inside the border, then the border.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        let (size, style) = (self.get_size(), self.style());
        let width = style.border_width.unwrap_or(self.border_width);
        let inset = i32::from(width);
        let inner = Rect::new(
            inset,
            inset,
            size.w.saturating_sub(2 * inset as u32),
            size.h.saturating_sub(2 * inset as u32),
        );

        paint::fill(p, style.background.unwrap_or(self.get_color()));
        let font = style.font.as_ref().unwrap_or(&self.style);
        paint::text(
            p,
            inner,
            font,
            style.foreground.unwrap_or(self.style.font_color),
            &self.msg,
            Truncate::End,
        );
        paint::border(
            p,
            size,
            style.border_color.unwrap_or(self.border_color),
            width,
        );
    }
}

impl BoxedLabel {
    /// Creates a label, 120 by 24 pixels at the upper left-hand corner of its parent, with the
    /// default font, black text, and a black one pixel border.  The fill is transparent until
    /// set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The text to display.
    ///
    /// # Returns
    ///
    /// The new `BoxedLabel`.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(120, 24)),
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Center),
            msg: msg.into(),
            border_color: Color::BLACK,
            border_width: 1,
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
    /// The label, for chaining.
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
    /// The label, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets the font.
    ///
    /// # Arguments
    ///
    /// * `font` - The font.
    ///
    /// # Returns
    ///
    /// The label, for chaining.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Sets the text color.
    ///
    /// # Arguments
    ///
    /// * `color` - The text color.
    ///
    /// # Returns
    ///
    /// The label, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
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
    /// The label, for chaining.
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
    /// The label, for chaining.
    #[must_use]
    pub fn border_width(mut self, width: u8) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the text, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `msg` - The new text.
    pub fn set_text(&mut self, msg: impl Into<String>) {
        self.msg = msg.into();
        self.set_invalidated(true);
    }

    /// Returns the text being displayed.
    ///
    /// # Returns
    ///
    /// A copy of the text.
    pub fn get_text(&self) -> String {
        self.msg.clone()
    }

    /// Sets the border color, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `color` - The border color.
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
        self.set_invalidated(true);
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
}
