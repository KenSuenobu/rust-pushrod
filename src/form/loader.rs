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

//! # Form files
//!

//! # Form loader
//!
//! Builds the widgets of a form stored in a [`FormFile`] through the
//! [`WidgetRegistry`], applies each object's
//! properties, and returns a [`FormHandle`] that maps the form's stable object numbers (and the
//! names in its symbol table) to the [`WidgetId`]s of the new widgets.  The application loads the
//! form and attaches callbacks; it does not rebuild the UI in code.
//!
//! Loading is all or nothing: every widget is built before any is added, so a failure leaves the
//! window unchanged.  Each error names the object number, its type, and the property.
//!
//! ## Layout kind and alignment codes
//!
//! | Layout kind | Meaning | Parameters |
//! | --- | --- | --- |
//! | `0` | none: children keep their positions | |
//! | `1` | [`Column`]: top to bottom | spacing, margin on every side (both optional) |
//! | `2` | [`Row`]: left to right | spacing, margin on every side (both optional) |
//!
//! Alignment codes: `0` start, `1` center, `2` end, `3` fill.  A grid cell in a child layout is
//! stored but ignored until a grid layout exists.
//!
//! ## Flags and sizes
//!
//! A nonzero behavior or state value replaces the widget's flags; zero keeps what the widget's
//! constructor set.  A zero width or height keeps the widget's own size.  Anchors are stored in
//! the file and not applied yet.
//!
//! After the constructor runs, each named property is also offered to the widget's
//! [`set_property`](Widget::set_property), so values the constructor does not read (such as
//! `checked`) are restored; a property the widget does not accept was consumed by the constructor.
//!
//! An image property becomes a text property holding the image's name in the image pool.

use super::format::{
    ChildLayout, FormFile, FormValue, LayoutSpec, NO_OBJECT, ObjectRecord, Property, SymbolKind,
};
use crate::error::{Error, Result};
use crate::event::{EngineEvent, Event};
use crate::geometry::{point, size};
use crate::layout::{Align, Alignment, Column, LayoutParams, Margin, Row};
use crate::registry::{Properties, PropertyValue, WidgetRegistry};
use crate::widget::{Behavior, VisualState, Widget, WidgetId};
use std::collections::HashMap;

/// The widgets a loaded form created, by object number and by symbol name.
///
/// Returned by `Engine::load_form`.  Use [`id`](FormHandle::id) with a generated ID constant, or
/// [`named`](FormHandle::named) with a name from the form's symbol table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormHandle {
    name: String,
    root: WidgetId,
    ids: HashMap<u32, WidgetId>,
    names: HashMap<String, u32>,
    actions: HashMap<String, u32>,
}

impl FormHandle {
    /// Returns the form's name.
    ///
    /// # Returns
    ///
    /// The name the form has in its file.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the form's root widget.
    ///
    /// # Returns
    ///
    /// The ID of the widget made from the form's root object.
    pub fn root(&self) -> WidgetId {
        self.root
    }

    /// Returns the widget made from an object number.
    ///
    /// # Arguments
    ///
    /// * `number` - The form-local object number, such as a generated ID constant.
    ///
    /// # Returns
    ///
    /// The widget's ID, or `None` if the object is not part of this form.
    pub fn id(&self, number: u32) -> Option<WidgetId> {
        self.ids.get(&number).copied()
    }

    /// Returns the widget a symbol name refers to.
    ///
    /// # Arguments
    ///
    /// * `name` - A name from the file's symbol table, such as `OK_BUTTON`.
    ///
    /// # Returns
    ///
    /// The widget's ID, or `None` if the name is unknown or its object is not in this form.
    pub fn named(&self, name: &str) -> Option<WidgetId> {
        self.names.get(name).and_then(|n| self.id(*n))
    }

    /// Returns the number of an action symbol.
    ///
    /// # Arguments
    ///
    /// * `name` - An action name from the file's symbol table, such as `save`.
    ///
    /// # Returns
    ///
    /// The action number, or `None` if there is no such action.
    pub fn action(&self, name: &str) -> Option<u32> {
        self.actions.get(name).copied()
    }

    /// Returns how many widgets the form created.
    ///
    /// # Returns
    ///
    /// The widget count, including the root.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns whether the form created no widgets.
    ///
    /// # Returns
    ///
    /// `false` for any loaded form, which always has a root.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// Builds an error that names an object.
///
/// # Arguments
///
/// * `form` - The form's name.
/// * `number` - The object number.
/// * `record` - The object.
/// * `message` - What is wrong.
///
/// # Returns
///
/// A [`Error::FormFile`] naming the form, object number, and type.
fn object_error(form: &str, number: u32, record: &ObjectRecord, message: &str) -> Error {
    Error::form_file(
        form,
        format!("object {number} ({}): {message}", record.type_name),
    )
}

/// Converts a stored value to a registry property value.
///
/// # Arguments
///
/// * `file` - The form file, for the image pool.
/// * `value` - The stored value.
///
/// # Returns
///
/// The value, or `None` if an image number is outside the pool.
fn convert(file: &FormFile, value: &FormValue) -> Option<PropertyValue> {
    Some(match value {
        FormValue::Bool(v) => PropertyValue::Bool(*v),
        FormValue::Int(v) => PropertyValue::Int(*v),
        FormValue::Float(v) => PropertyValue::Float(*v),
        FormValue::Text(v) => PropertyValue::Text(v.clone()),
        FormValue::Color(v) => PropertyValue::Color(*v),
        FormValue::Image(index) => {
            PropertyValue::Text(file.images.get(*index as usize)?.name.clone())
        }
    })
}

/// Converts an alignment code.
///
/// # Arguments
///
/// * `code` - `0` start, `1` center, `2` end, `3` fill.
///
/// # Returns
///
/// The alignment, or `None` for an unknown code.
fn align(code: u8) -> Option<Align> {
    match code {
        0 => Some(Align::Start),
        1 => Some(Align::Center),
        2 => Some(Align::End),
        3 => Some(Align::Fill),
        _ => None,
    }
}

/// Converts a child layout record to layout parameters.
///
/// # Arguments
///
/// * `child` - The stored child layout.
///
/// # Returns
///
/// The parameters, or `None` if an alignment code is unknown.
fn child_params(child: &ChildLayout) -> Option<LayoutParams> {
    Some(LayoutParams {
        stretch: u32::from(child.stretch),
        alignment: Alignment::new(align(child.horizontal)?, align(child.vertical)?),
        margin: Margin {
            left: child.margins[0],
            top: child.margins[1],
            right: child.margins[2],
            bottom: child.margins[3],
        },
    })
}

/// Gives a widget the layout manager a stored layout names.
///
/// # Arguments
///
/// * `widget` - The container.
/// * `spec` - The stored layout.
///
/// # Returns
///
/// `Ok(())`, or a message when the kind is unknown or a parameter is negative.
fn apply_layout(widget: &mut dyn Widget, spec: &LayoutSpec) -> std::result::Result<(), String> {
    let spacing = spec.params.first().copied().unwrap_or(0);
    let margin = spec.params.get(1).copied().unwrap_or(0);

    if spacing < 0 || margin < 0 {
        return Err("layout spacing and margin must not be negative".to_string());
    }

    let (spacing, margin) = (spacing as u32, Margin::all(margin as u32));

    match spec.kind {
        0 => {}
        1 => widget.set_layout(Box::new(Column::new().spacing(spacing).margins(margin))),
        2 => widget.set_layout(Box::new(Row::new().spacing(spacing).margins(margin))),
        other => return Err(format!("unknown layout kind {other}")),
    }

    Ok(())
}

/// Builds one object's widget, without adding it to the tree.
///
/// # Arguments
///
/// * `registry` - The widget registry.
/// * `file` - The form file.
/// * `form` - The form's name, for errors.
/// * `number` - The object number.
///
/// # Returns
///
/// The widget, with its properties, geometry, flags, and layout applied.
///
/// # Errors
///
/// Returns an error naming the object (and the property when there is one).
fn build(
    registry: &WidgetRegistry,
    file: &FormFile,
    form: &str,
    number: u32,
) -> Result<Box<dyn Widget>> {
    let record = &file.objects[number as usize];
    let mut props = Properties::new();
    let mut named = Vec::new();

    for property in &record.properties {
        if let Property::Named { name, value } = property {
            let value = convert(file, value).ok_or_else(|| {
                Error::property(
                    name.clone(),
                    format!(
                        "object {number} ({}): image number is not in the image pool",
                        record.type_name
                    ),
                )
            })?;

            named.push((name.clone(), value.clone()));
            props.set(name.clone(), value);
        }
    }

    let mut widget = registry
        .create(&record.type_name, &props)
        .map_err(|e| match e {
            Error::Property { name, message } => Error::property(
                name,
                format!("object {number} ({}): {message}", record.type_name),
            ),
            Error::UnknownWidgetType { name } => object_error(
                form,
                number,
                record,
                &format!("widget type \"{name}\" is not registered"),
            ),
            other => object_error(form, number, record, &other.to_string()),
        })?;

    // A constructor may read only some properties; give the widget the rest, such as `checked`.
    // A property the widget does not accept by name was already consumed by its constructor.
    for (name, value) in &named {
        let _ = widget.set_property(name, value);
    }

    widget.set_origin(point(record.x, record.y));

    if record.width != 0 || record.height != 0 {
        let current = widget.get_size();

        widget.set_size(size(
            if record.width != 0 {
                record.width
            } else {
                current.w
            },
            if record.height != 0 {
                record.height
            } else {
                current.h
            },
        ));
    }

    if record.behavior != 0 {
        let bits = u16::try_from(record.behavior)
            .map_err(|_| object_error(form, number, record, "behavior flags do not fit"))?;

        widget.set_behavior(Behavior::from_bits_truncate(bits));
    }

    if record.state != 0 {
        let bits = u16::try_from(record.state)
            .map_err(|_| object_error(form, number, record, "state flags do not fit"))?;

        widget.set_state(VisualState::from_bits_truncate(bits));
    }

    for property in &record.properties {
        match property {
            Property::Layout(spec) => apply_layout(widget.as_mut(), spec)
                .map_err(|m| object_error(form, number, record, &m))?,
            Property::ChildLayout(child) => {
                let params = child_params(child).ok_or_else(|| {
                    object_error(
                        form,
                        number,
                        record,
                        "unknown alignment code in child layout",
                    )
                })?;

                widget.set_layout_params(params);
            }
            Property::Named { .. } => {}
        }
    }

    Ok(widget)
}

/// Loads a form into the widget tree.
///
/// # Arguments
///
/// * `cache` - The widget tree to add to.
/// * `registry` - The widget registry that creates each object.
/// * `parent` - The widget the form's root becomes a child of.
/// * `file` - The form file.
/// * `name` - The form's name.
///
/// # Returns
///
/// The [`FormHandle`] for the new widgets.  Queues [`EngineEvent::FormLoaded`].
///
/// # Errors
///
/// Returns an error if the file is inconsistent, the form or a widget type is unknown, a property
/// is missing or invalid, or `parent` does not exist.  The tree is unchanged on error.
pub(crate) fn load(
    cache: &mut crate::cache::WidgetCache,
    registry: &WidgetRegistry,
    parent: WidgetId,
    file: &FormFile,
    name: &str,
) -> Result<FormHandle> {
    file.validate()
        .map_err(|e| Error::form_file(name, e.to_string()))?;

    let entry = file
        .form(name)
        .ok_or_else(|| Error::form_file(name, "no form with this name in the file"))?;

    // Walk the tree parents first, so a parent is added before its children.
    let mut order = Vec::new();
    let mut stack = vec![entry.root];

    while let Some(number) = stack.pop() {
        order.push(number);
        stack.extend(file.children(number).into_iter().rev());
    }

    let mut built = Vec::with_capacity(order.len());

    for &number in &order {
        built.push(build(registry, file, name, number)?);
    }

    if cache.get(parent).is_none() {
        return Err(Error::UnknownWidget { id: parent });
    }

    let mut ids = HashMap::new();

    for (&number, widget) in order.iter().zip(built) {
        let record = &file.objects[number as usize];
        let owner = if record.parent == NO_OBJECT || number == entry.root {
            parent
        } else {
            ids[&record.parent]
        };

        ids.insert(number, cache.try_add_child(owner, widget)?);
    }

    let mut names = HashMap::new();
    let mut actions = HashMap::new();

    for symbol in &file.symbols {
        match symbol.kind {
            SymbolKind::Id => {
                names.entry(symbol.name.clone()).or_insert(symbol.id);
            }
            SymbolKind::Action => {
                actions.entry(symbol.name.clone()).or_insert(symbol.id);
            }
        }
    }

    let handle = FormHandle {
        name: name.to_string(),
        root: ids[&entry.root],
        ids,
        names,
        actions,
    };

    cache.queue_event(Event::Engine(EngineEvent::FormLoaded {
        form: name.to_string(),
        root: handle.root,
        widgets: handle.len(),
    }));

    Ok(handle)
}
