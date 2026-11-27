// GroupBox Widget
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

//! # GroupBox
//!
//! A frame with a title in a gap at the left of its top border, grouping related controls.
//! Children are added to it like to any other widget (`Engine::add_child_widget`); it has an
//! empty event mask, so it receives no input and emits no events.

use pushrod::accessibility::{AccessInfo, Role};
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

/// Distance from the left edge to the start of the title's gap, in pixels.
const TITLE_INSET: u32 = 10;

/// Space kept clear on each side of the title, in pixels.
const TITLE_PADDING: u32 = 4;

/// A titled frame.  The border runs along the vertical middle of the title's line, and breaks
/// where the title is drawn.  A title too long for the box is shortened with an ellipsis.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::group_box::GroupBox;
///
/// let group = GroupBox::new("Size").at(20, 20).size(200, 120);
/// ```
#[derive(Debug)]
pub struct GroupBox {
    core: WidgetCore,
    style: TextStyle,
    title: String,
    border_color: Color,
    border_width: u8,
}

impl Widget for GroupBox {
    /// Returns the group box's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the group box's shared state for modification.
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

        list.push(Property::text("title", self.get_title()));
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
            "title" => {
                let t = expect_text(name, value)?.to_string();

                track(self, |w| w.set_title(t))
            }
            "border_color" => {
                let c = expect_color(name, value)?;

                track(self, |w| w.set_border_color(c))
            }
            _ => set_core_property(self, name, value),
        }
    }

    /// Describes the widget to a screen reader.
    ///
    /// # Returns
    ///
    /// The widget's role, name, value, state, and actions.
    fn accessibility(&self) -> AccessInfo {
        AccessInfo::of(self, Role::Group).with_name(&self.title)
    }

    /// Fills the background, draws the title, then the border with a gap behind the title.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        let (size, style) = (self.get_size(), self.style());
        let color = style.border_color.unwrap_or(self.border_color);
        let width = u32::from(style.border_width.unwrap_or(self.border_width));
        let font = style.font.as_ref().unwrap_or(&self.style);
        let line = p.line_height(font);

        paint::fill(p, style.background.unwrap_or(self.get_color()));

        // The gap, as (start, end) x positions; empty when there is no title or no room.
        let mut gap = (0, 0);
        let room = size.w.saturating_sub(2 * (TITLE_INSET + TITLE_PADDING));

        if !self.title.is_empty() && room > 0 {
            let drawn = paint::text(
                p,
                Rect::new((TITLE_INSET + TITLE_PADDING) as i32, 0, room, line),
                font,
                style.foreground.unwrap_or(self.style.font_color),
                &self.title,
                Truncate::End,
            );

            if drawn.size.w > 0 {
                gap = (TITLE_INSET, TITLE_INSET + drawn.size.w + 2 * TITLE_PADDING);
            }
        }

        if width == 0 || size.w == 0 || size.h == 0 {
            return;
        }

        let top = (line / 2).min(size.h.saturating_sub(1));
        let (w, h) = (width.min(size.w), width.min(size.h - top));
        let rect = |x: u32, y: u32, rw: u32, rh: u32| {
            Rect::new(x as i32, y as i32, rw.min(size.w - x), rh.min(size.h - y))
        };

        // Top edge: whole when there is no title, otherwise in two pieces around the gap.
        if gap == (0, 0) {
            p.fill_rect(rect(0, top, size.w, h), color);
        } else {
            p.fill_rect(rect(0, top, gap.0, h), color);
            p.fill_rect(rect(gap.1, top, size.w.saturating_sub(gap.1), h), color);
        }

        // Left, right, and bottom edges.
        let side = size.h - top;
        p.fill_rect(rect(0, top, w, side), color);
        p.fill_rect(rect(size.w - w, top, w, side), color);
        p.fill_rect(rect(0, size.h - h, size.w, h), color);
    }
}

impl GroupBox {
    /// Creates a group box, 200 by 100 pixels at the upper left-hand corner of its parent, with
    /// the default font, black title, and a black one pixel border.  The fill is transparent
    /// until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `title` - The title shown in the top border.
    ///
    /// # Returns
    ///
    /// The new `GroupBox`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(200, 100)),
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            title: title.into(),
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
    /// The group box, for chaining.
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
    /// The group box, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets the title font.
    ///
    /// # Arguments
    ///
    /// * `font` - The font.
    ///
    /// # Returns
    ///
    /// The group box, for chaining.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Sets the title color.
    ///
    /// # Arguments
    ///
    /// * `color` - The title color.
    ///
    /// # Returns
    ///
    /// The group box, for chaining.
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
    /// The group box, for chaining.
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
    /// The group box, for chaining.
    #[must_use]
    pub fn border_width(mut self, width: u8) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the title, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `title` - The new title; empty draws an unbroken border.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.set_invalidated(true);
    }

    /// Returns the title.
    ///
    /// # Returns
    ///
    /// A copy of the title.
    pub fn get_title(&self) -> String {
        self.title.clone()
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
}
