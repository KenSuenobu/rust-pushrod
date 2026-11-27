// Widget Registry
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

//! # Widget registry
//!
//! A form file stores "a button with these properties".  Loading it has to build a `Widget` from a
//! type name, including widgets the application or another widget library defined.  A
//! [`WidgetRegistry`] maps stable type names (`"pushrod.button"`) to constructors that build a
//! `Widget` from [`Properties`].
//!
//! Each widget library registers its own types under its own prefix
//! (`pushrod_widgets::register(&mut registry)` adds the standard set).  Registering a name that
//! already exists replaces its constructor, so a library can give existing forms a new look.
//! Asking for a name that was never registered is an [`Error::UnknownWidgetType`], not a panic.
//!
//! ```rust
//! use pushrod::prelude::*;
//! use pushrod::registry::{Properties, WidgetRegistry};
//! use pushrod::root_widget::RootWidget;
//!
//! let mut registry = WidgetRegistry::new();
//!
//! registry.register("acme.blank", |props| {
//!     let w = props.get_int("width").unwrap_or(10) as u32;
//!
//!     Ok(Box::new(RootWidget::new(size(w, 10))))
//! });
//!
//! let widget = registry.create("acme.blank", &Properties::new().with_int("width", 40)).unwrap();
//!
//! assert_eq!(widget.get_size().w, 40);
//! assert!(registry.create("acme.missing", &Properties::new()).is_err());
//! ```

use crate::color::Color;
use crate::error::{Error, Result};
use crate::geometry::{point, size};
use crate::widget::Widget;
use std::collections::BTreeMap;
use std::fmt;

/// One property value: the kinds a form file can store.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// A true or false value.
    Bool(bool),
    /// A whole number.
    Int(i64),
    /// A number with a fractional part.
    Float(f64),
    /// Text.
    Text(String),
    /// A color.
    Color(Color),
}

/// The named values a constructor builds a `Widget` from.
///
/// The names `x`, `y`, `width`, `height`, and `hidden` are applied to every created `Widget` by
/// [`WidgetRegistry::create`], so a constructor only reads the properties of its own widget.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Properties {
    values: BTreeMap<String, PropertyValue>,
}

impl Properties {
    /// Creates an empty property set.
    ///
    /// # Returns
    ///
    /// The empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value, replacing any earlier value for the name.
    pub fn set(&mut self, name: impl Into<String>, value: PropertyValue) {
        self.values.insert(name.into(), value);
    }

    /// Returns the set with a boolean property added.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The updated set.
    pub fn with_bool(mut self, name: &str, value: bool) -> Self {
        self.set(name, PropertyValue::Bool(value));
        self
    }

    /// Returns the set with a whole-number property added.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The updated set.
    pub fn with_int(mut self, name: &str, value: i64) -> Self {
        self.set(name, PropertyValue::Int(value));
        self
    }

    /// Returns the set with a fractional-number property added.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The updated set.
    pub fn with_float(mut self, name: &str, value: f64) -> Self {
        self.set(name, PropertyValue::Float(value));
        self
    }

    /// Returns the set with a text property added.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The updated set.
    pub fn with_text(mut self, name: &str, value: impl Into<String>) -> Self {
        self.set(name, PropertyValue::Text(value.into()));
        self
    }

    /// Returns the set with a color property added.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The updated set.
    pub fn with_color(mut self, name: &str, value: Color) -> Self {
        self.set(name, PropertyValue::Color(value));
        self
    }

    /// Returns a property's value.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    ///
    /// # Returns
    ///
    /// The value, or `None` if the property is not set.
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.values.get(name)
    }

    /// Returns a boolean property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    ///
    /// # Returns
    ///
    /// The value, or `None` if it is not set or is not a boolean.
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.get(name) {
            Some(PropertyValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    /// Returns a whole-number property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    ///
    /// # Returns
    ///
    /// The value, or `None` if it is not set or is not a whole number.
    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.get(name) {
            Some(PropertyValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    /// Returns a number property; a whole number is converted.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    ///
    /// # Returns
    ///
    /// The value, or `None` if it is not set or is not a number.
    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.get(name) {
            Some(PropertyValue::Float(v)) => Some(*v),
            Some(PropertyValue::Int(v)) => Some(*v as f64),
            _ => None,
        }
    }

    /// Returns a text property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    ///
    /// # Returns
    ///
    /// The value, or `None` if it is not set or is not text.
    pub fn get_text(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(PropertyValue::Text(v)) => Some(v),
            _ => None,
        }
    }

    /// Returns a color property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    ///
    /// # Returns
    ///
    /// The value, or `None` if it is not set or is not a color.
    pub fn get_color(&self, name: &str) -> Option<Color> {
        match self.get(name) {
            Some(PropertyValue::Color(v)) => Some(*v),
            _ => None,
        }
    }
}

/// A function that builds a `Widget` from properties.
///
/// It returns an error, rather than panicking, when a property it needs is missing or invalid.
pub type Constructor = Box<dyn Fn(&Properties) -> Result<Box<dyn Widget>>>;

/// Maps stable widget type names to constructors.
///
/// See the [module documentation](self) for an example.
#[derive(Default)]
pub struct WidgetRegistry {
    constructors: BTreeMap<String, Constructor>,
}

impl fmt::Debug for WidgetRegistry {
    /// Lists the registered type names.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetRegistry")
            .field("types", &self.constructors.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl WidgetRegistry {
    /// Creates an empty registry.  Add the standard widgets with `pushrod_widgets::register`.
    ///
    /// # Returns
    ///
    /// The empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a constructor under a type name, replacing any constructor already there.
    ///
    /// # Arguments
    ///
    /// * `name` - The stable type name, such as `"acme.tree_view"`.
    /// * `constructor` - Builds the `Widget` from its properties.
    ///
    /// # Returns
    ///
    /// `true` if an existing constructor was replaced.
    pub fn register<F>(&mut self, name: impl Into<String>, constructor: F) -> bool
    where
        F: Fn(&Properties) -> Result<Box<dyn Widget>> + 'static,
    {
        self.constructors
            .insert(name.into(), Box::new(constructor))
            .is_some()
    }

    /// Returns whether a type name is registered.
    ///
    /// # Arguments
    ///
    /// * `name` - The type name.
    ///
    /// # Returns
    ///
    /// `true` if the name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.constructors.contains_key(name)
    }

    /// Lists the registered type names, in alphabetical order (for a palette).
    ///
    /// # Returns
    ///
    /// The names.
    pub fn names(&self) -> Vec<&str> {
        self.constructors.keys().map(String::as_str).collect()
    }

    /// Builds a `Widget` from its type name and properties, then applies the common properties
    /// `x`, `y`, `width`, `height`, `hidden`, and `color`.
    ///
    /// # Arguments
    ///
    /// * `name` - The registered type name.
    /// * `properties` - The values to build the `Widget` from.
    ///
    /// # Returns
    ///
    /// The new `Widget`, not yet in a tree.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownWidgetType`] if `name` is not registered, or whatever error the
    /// constructor reports for missing or invalid properties.
    pub fn create(&self, name: &str, properties: &Properties) -> Result<Box<dyn Widget>> {
        let constructor = self
            .constructors
            .get(name)
            .ok_or_else(|| Error::unknown_widget_type(name))?;
        let mut widget = constructor(properties)?;

        if properties.get_int("x").is_some() || properties.get_int("y").is_some() {
            let origin = widget.get_origin();
            let x = properties.get_int("x").map_or(origin.x, |v| v as i32);
            let y = properties.get_int("y").map_or(origin.y, |v| v as i32);

            widget.set_origin(point(x, y));
        }

        if properties.get_int("width").is_some() || properties.get_int("height").is_some() {
            let current = widget.get_size();
            let w = properties
                .get_int("width")
                .map_or(current.w, |v| v.max(0) as u32);
            let h = properties
                .get_int("height")
                .map_or(current.h, |v| v.max(0) as u32);

            widget.set_size(size(w, h));
        }

        if let Some(color) = properties.get_color("color") {
            widget.set_color(color);
        }

        if let Some(hidden) = properties.get_bool("hidden") {
            widget.set_hidden(hidden);
        }

        Ok(widget)
    }
}
