// Property Introspection
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

//! # Property introspection
//!
//! Every [`Widget`] can list its properties with [`Widget::properties`], and change one by name
//! with [`Widget::set_property`], without the caller knowing the concrete type.  Saving and
//! loading forms, property bindings, and a visual designer's inspector are built on this.
//!
//! A [`Property`] is a name, a [`PropertyKind`], and a [`PropertyValue`] (the same value type a
//! form file stores; see [`registry`](crate::registry)).  The properties every `Widget` has
//! come by default from [`core_properties`]; a widget with more adds its own and falls back to
//! [`set_core_property`] for the rest.
//!
//! | Name | Kind | Meaning |
//! | --- | --- | --- |
//! | `x`, `y` | Int | Origin, relative to the parent. |
//! | `width`, `height` | Int | Size in pixels. |
//! | `color` | Color | Background color. |
//! | `hidden` | Bool | Hidden. |
//! | `enabled` | Bool | Enabled. |
//! | `tooltip` | Text | Tooltip text; empty for none. |
//! | `behavior` | Int | The [`Behavior`] bits. |
//! | `state` | Int | The [`VisualState`] bits. |
//! | `stretch` | Int | Layout stretch factor (see [`LayoutParams`](crate::layout::LayoutParams)). |
//! | `margin_left`, `margin_top`, `margin_right`, `margin_bottom` | Int | Layout margin. |
//! | `align_horizontal`, `align_vertical` | Text | `start`, `center`, `end`, or `fill`. |
//!
//! Changing a property through [`Engine::set_widget_property`](crate::engine::Engine::set_widget_property)
//! emits [`EngineEvent::PropertyChanged`](crate::event::EngineEvent::PropertyChanged).
//!
//! ```rust
//! use pushrod::prelude::*;
//! use pushrod::properties::{PropertyKind, core_properties};
//! use pushrod::registry::PropertyValue;
//!
//! let mut engine: Engine = Engine::new(size(200, 100), 30);
//! let id = engine.add_widget(pushrod::root_widget::RootWidget::new(size(200, 100)));
//!
//! assert!(engine.widgets().get(id).unwrap().properties().iter().any(|p| p.name == "width"));
//!
//! // Changing a value reports `true`; setting the same value again reports `false`.
//! assert_eq!(engine.set_widget_property(id, "x", &PropertyValue::Int(5)), Ok(true));
//! assert_eq!(engine.set_widget_property(id, "x", &PropertyValue::Int(5)), Ok(false));
//! assert_eq!(PropertyKind::of(&PropertyValue::Int(5)), PropertyKind::Int);
//! # let _ = core_properties::<pushrod::root_widget::RootWidget>;
//! ```

use crate::color::Color;
use crate::error::{Error, Result};
use crate::geometry::{Point, Size};
use crate::layout::Align;
use crate::registry::PropertyValue;
use crate::widget::{Behavior, VisualState, Widget};

/// The kind of value a [`Property`] holds.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKind {
    /// A true or false value.
    Bool,
    /// A whole number.
    Int,
    /// A number with a fractional part.
    Float,
    /// Text.
    Text,
    /// A color.
    Color,
}

impl PropertyKind {
    /// Returns the kind of a value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The value's kind.
    pub fn of(value: &PropertyValue) -> Self {
        match value {
            PropertyValue::Bool(_) => PropertyKind::Bool,
            PropertyValue::Int(_) => PropertyKind::Int,
            PropertyValue::Float(_) => PropertyKind::Float,
            PropertyValue::Text(_) => PropertyKind::Text,
            PropertyValue::Color(_) => PropertyKind::Color,
        }
    }
}

/// One named, typed value of a `Widget`.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The property's name, such as `width`.
    pub name: String,
    /// The kind of value it holds.
    pub kind: PropertyKind,
    /// The current value.
    pub value: PropertyValue,
}

impl Property {
    /// Creates a property; its kind comes from the value.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The current value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn new(name: &str, value: PropertyValue) -> Self {
        Self {
            name: name.to_string(),
            kind: PropertyKind::of(&value),
            value,
        }
    }

    /// Creates a boolean property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The current value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn bool(name: &str, value: bool) -> Self {
        Self::new(name, PropertyValue::Bool(value))
    }

    /// Creates a whole-number property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The current value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn int(name: &str, value: i64) -> Self {
        Self::new(name, PropertyValue::Int(value))
    }

    /// Creates a fractional-number property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The current value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn float(name: &str, value: f64) -> Self {
        Self::new(name, PropertyValue::Float(value))
    }

    /// Creates a text property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The current value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn text(name: &str, value: impl Into<String>) -> Self {
        Self::new(name, PropertyValue::Text(value.into()))
    }

    /// Creates a color property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The current value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn color(name: &str, value: Color) -> Self {
        Self::new(name, PropertyValue::Color(value))
    }
}

/// Reads a boolean value.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The boolean.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not a boolean.
pub fn expect_bool(name: &str, value: &PropertyValue) -> Result<bool> {
    match value {
        PropertyValue::Bool(b) => Ok(*b),
        _ => Err(Error::property(name, "expected a boolean")),
    }
}

/// Reads a whole-number value.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The number.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not a whole number.
pub fn expect_int(name: &str, value: &PropertyValue) -> Result<i64> {
    match value {
        PropertyValue::Int(i) => Ok(*i),
        _ => Err(Error::property(name, "expected a whole number")),
    }
}

/// Reads a whole-number value that must fit a `u32`.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The number.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not a whole number from 0 to `u32::MAX`.
pub fn expect_u32(name: &str, value: &PropertyValue) -> Result<u32> {
    u32::try_from(expect_int(name, value)?)
        .map_err(|_| Error::property(name, "expected a whole number of 0 or more"))
}

/// Reads a number; a whole number is accepted and converted.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The number.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not a number.
pub fn expect_float(name: &str, value: &PropertyValue) -> Result<f64> {
    match value {
        PropertyValue::Float(f) => Ok(*f),
        PropertyValue::Int(i) => Ok(*i as f64),
        _ => Err(Error::property(name, "expected a number")),
    }
}

/// Reads a text value.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The text.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not text.
pub fn expect_text<'a>(name: &str, value: &'a PropertyValue) -> Result<&'a str> {
    match value {
        PropertyValue::Text(t) => Ok(t),
        _ => Err(Error::property(name, "expected text")),
    }
}

/// Reads a color value.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The color.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not a color.
pub fn expect_color(name: &str, value: &PropertyValue) -> Result<Color> {
    match value {
        PropertyValue::Color(c) => Ok(*c),
        _ => Err(Error::property(name, "expected a color")),
    }
}

/// Creates the error for a property that can be listed but not changed.
///
/// # Arguments
///
/// * `name` - The property name.
///
/// # Returns
///
/// The error.
pub fn read_only(name: &str) -> Error {
    Error::property(name, "the property is read only")
}

/// Returns the text for an alignment.
///
/// # Arguments
///
/// * `align` - The alignment along one axis.
///
/// # Returns
///
/// `start`, `center`, `end`, or `fill`.
fn align_name(align: Align) -> &'static str {
    match align {
        Align::Start => "start",
        Align::Center => "center",
        Align::End => "end",
        Align::Fill => "fill",
    }
}

/// Reads an alignment from text.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `text` - `start`, `center`, `end`, or `fill`.
///
/// # Returns
///
/// The alignment.
///
/// # Errors
///
/// Returns [`Error::Property`] for any other text.
fn parse_align(name: &str, text: &str) -> Result<Align> {
    match text {
        "start" => Ok(Align::Start),
        "center" => Ok(Align::Center),
        "end" => Ok(Align::End),
        "fill" => Ok(Align::Fill),
        _ => Err(Error::property(
            name,
            "expected start, center, end, or fill",
        )),
    }
}

/// Applies a change to a `Widget` and reports whether its properties differ afterward.
///
/// A widget setter may clamp or ignore a value, so comparing the property lists before and after
/// is the reliable way to know whether a [`Widget::set_property`] really changed anything.
///
/// # Arguments
///
/// * `w` - The `Widget`.
/// * `apply` - The change to make.
///
/// # Returns
///
/// `Ok(true)` if any property differs after the change, `Ok(false)` otherwise.
///
/// # Errors
///
/// Never fails; it returns a `Result` so a `set_property` can return it directly.
pub fn track<W: Widget + ?Sized>(w: &mut W, apply: impl FnOnce(&mut W)) -> Result<bool> {
    let before = w.properties();

    apply(w);

    Ok(w.properties() != before)
}

/// Returns the properties every `Widget` has.
///
/// # Arguments
///
/// * `w` - The `Widget`.
///
/// # Returns
///
/// The core properties, in the order of the table in the [module documentation](self).
pub fn core_properties<W: Widget + ?Sized>(w: &W) -> Vec<Property> {
    let origin = w.get_origin();
    let size = w.get_size();
    let params = w.layout_params();

    vec![
        Property::int("x", i64::from(origin.x)),
        Property::int("y", i64::from(origin.y)),
        Property::int("width", i64::from(size.w)),
        Property::int("height", i64::from(size.h)),
        Property::color("color", w.get_color()),
        Property::bool("hidden", w.is_hidden()),
        Property::bool("enabled", w.is_enabled()),
        Property::text("tooltip", w.tooltip()),
        Property::int("behavior", i64::from(w.behavior().bits())),
        Property::int("state", i64::from(w.state().bits())),
        Property::int("stretch", i64::from(params.stretch)),
        Property::int("margin_left", i64::from(params.margin.left)),
        Property::int("margin_top", i64::from(params.margin.top)),
        Property::int("margin_right", i64::from(params.margin.right)),
        Property::int("margin_bottom", i64::from(params.margin.bottom)),
        Property::text("align_horizontal", align_name(params.alignment.horizontal)),
        Property::text("align_vertical", align_name(params.alignment.vertical)),
    ]
}

/// Converts a number to a coordinate.
///
/// # Arguments
///
/// * `name` - The property name, for the error.
/// * `value` - The value to read.
///
/// # Returns
///
/// The coordinate.
///
/// # Errors
///
/// Returns [`Error::Property`] if the value is not a whole number that fits.
fn coordinate(name: &str, value: &PropertyValue) -> Result<i32> {
    i32::try_from(expect_int(name, value)?).map_err(|_| Error::property(name, "out of range"))
}

/// Sets one of the properties every `Widget` has.
///
/// # Arguments
///
/// * `w` - The `Widget`.
/// * `name` - The property name.
/// * `value` - The new value.
///
/// # Returns
///
/// `true` if the value changed, `false` if the `Widget` already had it.
///
/// # Errors
///
/// Returns [`Error::Property`] if the name is not a core property, or the value has the wrong
/// kind or is out of range.
pub fn set_core_property<W: Widget + ?Sized>(
    w: &mut W,
    name: &str,
    value: &PropertyValue,
) -> Result<bool> {
    let origin = w.get_origin();
    let size = w.get_size();
    let mut params = w.layout_params();

    match name {
        "x" | "y" => {
            let n = coordinate(name, value)?;
            let new = if name == "x" {
                Point::new(n, origin.y)
            } else {
                Point::new(origin.x, n)
            };

            if new == origin {
                return Ok(false);
            }
            w.set_origin(new);
        }
        "width" | "height" => {
            let n = expect_u32(name, value)?;
            let new = if name == "width" {
                Size::new(n, size.h)
            } else {
                Size::new(size.w, n)
            };

            if new == size {
                return Ok(false);
            }
            w.set_size(new);
        }
        "color" => {
            let c = expect_color(name, value)?;

            if c == w.get_color() {
                return Ok(false);
            }
            w.set_color(c);
        }
        "hidden" => {
            let b = expect_bool(name, value)?;

            if b == w.is_hidden() {
                return Ok(false);
            }
            w.set_hidden(b);
        }
        "enabled" => {
            let b = expect_bool(name, value)?;

            if b == w.is_enabled() {
                return Ok(false);
            }
            w.set_enabled(b);
        }
        "tooltip" => {
            let t = expect_text(name, value)?;

            if t == w.tooltip() {
                return Ok(false);
            }
            w.set_tooltip(t);
        }
        "behavior" => {
            let b = Behavior::from_bits(
                u16::try_from(expect_int(name, value)?)
                    .map_err(|_| Error::property(name, "out of range"))?,
            )
            .ok_or_else(|| Error::property(name, "contains unknown behavior bits"))?;

            if b == w.behavior() {
                return Ok(false);
            }
            w.set_behavior(b);
        }
        "state" => {
            let s = VisualState::from_bits(
                u16::try_from(expect_int(name, value)?)
                    .map_err(|_| Error::property(name, "out of range"))?,
            )
            .ok_or_else(|| Error::property(name, "contains unknown state bits"))?;

            if s == w.state() {
                return Ok(false);
            }
            w.set_state(s);
        }
        "stretch" | "margin_left" | "margin_top" | "margin_right" | "margin_bottom"
        | "align_horizontal" | "align_vertical" => {
            let before = params;

            match name {
                "stretch" => params.stretch = expect_u32(name, value)?,
                "margin_left" => params.margin.left = expect_u32(name, value)?,
                "margin_top" => params.margin.top = expect_u32(name, value)?,
                "margin_right" => params.margin.right = expect_u32(name, value)?,
                "margin_bottom" => params.margin.bottom = expect_u32(name, value)?,
                "align_horizontal" => {
                    params.alignment.horizontal = parse_align(name, expect_text(name, value)?)?
                }
                _ => params.alignment.vertical = parse_align(name, expect_text(name, value)?)?,
            }

            if params == before {
                return Ok(false);
            }
            w.set_layout_params(params);
        }
        _ => return Err(Error::property(name, "no such property")),
    }

    Ok(true)
}
