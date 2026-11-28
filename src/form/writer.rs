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

//! # Form writer
//!
//! Saves a widget subtree as a form in a form file, so the form loader can build it again.
//! [`FormWriter::add_form`] walks the tree from a root widget, stores each widget's geometry,
//! flags, layout, and properties (read through [`Widget::properties`]), and returns the object
//! number each widget received.  [`FormWriter::write`] saves the file.
//!
//! A widget has no type name of its own, so tell the writer which Rust type stands for which
//! registry name with [`FormWriter::register_type`] (`pushrod_widgets::register_form_types`
//! does it for the standard widgets).
//!
//! ## What is written
//!
//! * The origin, size, behavior, and state of every widget.  The core properties that the object
//!   record already holds (`x`, `y`, `width`, `height`, `hidden`, `enabled`, `behavior`,
//!   `state`, and the layout parameters) are not repeated as named properties.
//! * Every other property from [`Widget::properties`], so what the widget lists comes back.
//! * A container's layout, and a child's layout parameters when they are not the defaults.
//! * **Not** the parts of a composite: they are rebuilt when the composite is created.
//!
//! Strings are stored once by the file format's string pool.  Images are stored once in the image
//! pool: [`FormWriter::add_image`] returns the same number for the same name and bytes, and
//! [`FormWriter::image_property`] makes a text property that names an added image store as an
//! image reference.

use super::format::{
    ChildLayout, FormFile, FormValue, LayoutSpec, ObjectRecord, Property, SymbolKind,
};
use crate::cache::WidgetCache;
use crate::error::{Error, Result};
use crate::layout::{Align, LayoutParams};
use crate::registry::PropertyValue;
use crate::widget::{Widget, WidgetId};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// The core properties an object record stores in its own fields, so they are not written by name.
const RECORD_PROPERTIES: [&str; 15] = [
    "x",
    "y",
    "width",
    "height",
    "hidden",
    "enabled",
    "behavior",
    "state",
    "stretch",
    "margin_left",
    "margin_top",
    "margin_right",
    "margin_bottom",
    "align_horizontal",
    "align_vertical",
];

/// Builds a [`FormFile`] from widgets.
///
/// ```
/// use pushrod::cache::WidgetCache;
/// use pushrod::form::writer::FormWriter;
/// use pushrod::geometry::size;
/// use pushrod::root_widget::RootWidget;
///
/// let mut cache = WidgetCache::new();
/// let panel = cache.add_child(0, Box::new(RootWidget::new(size(50, 20))));
///
/// let mut writer = FormWriter::new();
///
/// writer.register_type::<RootWidget>("acme.panel");
///
/// let numbers = writer.add_form("main", &cache, panel).unwrap();
///
/// assert_eq!(numbers[&panel], 0);
/// assert_eq!(writer.file().forms[0].name, "main");
/// ```
#[derive(Debug, Default)]
pub struct FormWriter {
    file: FormFile,
    types: HashMap<TypeId, String>,
    image_properties: HashSet<String>,
}

impl FormWriter {
    /// Creates a writer with an empty file and no known widget types.
    ///
    /// # Returns
    ///
    /// The writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Says which registry type name the widgets of a Rust type are saved as.
    ///
    /// # Arguments
    ///
    /// * `name` - The name the loader's registry knows the type by, such as `"pushrod.button"`.
    ///
    /// # Returns
    ///
    /// `true` if the type was already registered and its name was replaced.
    pub fn register_type<T: Widget>(&mut self, name: &str) -> bool {
        self.types
            .insert(TypeId::of::<T>(), name.to_string())
            .is_some()
    }

    /// Makes text properties with this name store as references to the image pool.
    ///
    /// A property is stored as an image only when its text is the name of an image added with
    /// [`add_image`](FormWriter::add_image); the loader turns it back into the same text.
    ///
    /// # Arguments
    ///
    /// * `property` - The property name, such as `"path"`.
    pub fn image_property(&mut self, property: &str) {
        self.image_properties.insert(property.to_string());
    }

    /// Adds an image to the image pool, once.
    ///
    /// # Arguments
    ///
    /// * `name` - The image's name.
    /// * `data` - The encoded image bytes.
    ///
    /// # Returns
    ///
    /// The image's number.  The same name and bytes give the same number.
    pub fn add_image(&mut self, name: &str, data: &[u8]) -> u32 {
        match self
            .file
            .images
            .iter()
            .position(|i| i.name == name && i.data == data)
        {
            Some(index) => index as u32,
            None => self.file.add_image(name, data.to_vec()),
        }
    }

    /// Adds a symbol (an ID name or an action name) to the file.
    ///
    /// # Arguments
    ///
    /// * `kind` - What the symbol names.
    /// * `id` - The object number (from [`add_form`](FormWriter::add_form)) or action number.
    /// * `name` - The symbol's name, such as `OK_BUTTON`.
    pub fn add_symbol(&mut self, kind: SymbolKind, id: u32, name: &str) {
        self.file.add_symbol(kind, id, name);
    }

    /// Saves the subtree under `root` as a form.
    ///
    /// # Arguments
    ///
    /// * `name` - The form's name.
    /// * `cache` - The widget tree.
    /// * `root` - The widget that becomes the form's root.
    ///
    /// # Returns
    ///
    /// The object number given to each widget written, by widget ID.  Nothing is added to the
    /// file on error.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownWidget`] if `root` does not exist, and [`Error::FormFile`] if
    /// the name is already used, a widget's type is not registered with the writer, or a layout
    /// cannot be stored.
    pub fn add_form(
        &mut self,
        name: &str,
        cache: &WidgetCache,
        root: WidgetId,
    ) -> Result<HashMap<WidgetId, u32>> {
        if cache.get(root).is_none() {
            return Err(Error::UnknownWidget { id: root });
        }

        if self.file.form(name).is_some() {
            return Err(Error::form_file(
                name,
                "a form with this name was already added",
            ));
        }

        // Build every record first, parents before children, so an error changes nothing.
        let mut order: Vec<(WidgetId, Option<usize>)> = Vec::new();
        let mut records = Vec::new();
        let mut stack = vec![(root, None)];

        while let Some((id, parent)) = stack.pop() {
            records.push(self.record(name, cache, id, id == root)?);
            order.push((id, parent));

            let index = order.len() - 1;
            let parts = part_ids(cache.get(id));

            for &child in cache.children_of(id).iter().rev() {
                if !parts.contains(&child) {
                    stack.push((child, Some(index)));
                }
            }
        }

        let mut numbers = HashMap::new();
        let mut by_index = Vec::with_capacity(order.len());

        for ((id, parent), record) in order.iter().zip(records) {
            let parent = parent.map(|i| by_index[i]);
            let number = self.file.add_object(parent, record);

            by_index.push(number);
            numbers.insert(*id, number);
        }

        self.file.add_form(name, numbers[&root]);

        Ok(numbers)
    }

    /// Returns the file built so far.
    ///
    /// # Returns
    ///
    /// The form file.
    pub fn file(&self) -> &FormFile {
        &self.file
    }

    /// Returns the file built so far, as bytes.
    ///
    /// # Returns
    ///
    /// The encoded file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FormFile`] if the file is inconsistent or too large for the format.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.file
            .to_bytes()
            .map_err(|e| Error::form_file("(memory)", e.to_string()))
    }

    /// Saves the file.
    ///
    /// # Arguments
    ///
    /// * `path` - Where to write it; an existing file is replaced.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the file is written.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FormFile`] naming the path if the file cannot be encoded or written.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let shown = path.display().to_string();
        let bytes = self
            .file
            .to_bytes()
            .map_err(|e| Error::form_file(shown.clone(), e.to_string()))?;

        std::fs::write(path, bytes).map_err(|e| Error::form_file(shown, e.to_string()))
    }

    /// Builds the object record of one widget, without its links.
    ///
    /// # Arguments
    ///
    /// * `form` - The form's name, for errors.
    /// * `cache` - The widget tree.
    /// * `id` - The widget.
    /// * `is_root` - Whether this is the form's root, which has no layout parameters to keep.
    ///
    /// # Returns
    ///
    /// The record.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FormFile`] if the type is not registered or the layout cannot be stored.
    fn record(
        &self,
        form: &str,
        cache: &WidgetCache,
        id: WidgetId,
        is_root: bool,
    ) -> Result<ObjectRecord> {
        let widget = cache.get(id).ok_or(Error::UnknownWidget { id })?;
        let any: &dyn Any = widget;
        let type_name = self.types.get(&any.type_id()).ok_or_else(|| {
            Error::form_file(
                form,
                format!("widget {id}: its type is not registered with the writer"),
            )
        })?;
        let mut record = ObjectRecord::new(type_name);
        let origin = widget.get_origin();
        let size = widget.get_size();

        record.x = origin.x;
        record.y = origin.y;
        record.width = size.w;
        record.height = size.h;
        record.behavior = u32::from(widget.behavior().bits());
        record.state = u32::from(widget.state().bits());

        for property in widget.properties() {
            if RECORD_PROPERTIES.contains(&property.name.as_str()) {
                continue;
            }

            let value = self.value(&property.name, &property.value);

            record
                .properties
                .push(Property::named(&property.name, value));
        }

        if widget.has_layout() {
            let spec = widget
                .core()
                .layout_ref()
                .and_then(|layout| layout.form_spec())
                .map(|(kind, params)| LayoutSpec { kind, params })
                .ok_or_else(|| {
                    Error::form_file(
                        form,
                        format!(
                            "widget {id} ({type_name}): its layout cannot be stored in a form file"
                        ),
                    )
                })?;

            record.properties.push(Property::Layout(spec));
        }

        let params = widget.layout_params();

        if !is_root && params != LayoutParams::default() {
            record
                .properties
                .push(Property::ChildLayout(child_layout(&params)));
        }

        Ok(record)
    }

    /// Converts a property value, using the image pool for named images.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The stored value.
    fn value(&self, name: &str, value: &PropertyValue) -> FormValue {
        match value {
            PropertyValue::Bool(v) => FormValue::Bool(*v),
            PropertyValue::Int(v) => FormValue::Int(*v),
            PropertyValue::Float(v) => FormValue::Float(*v),
            PropertyValue::Color(v) => FormValue::Color(*v),
            PropertyValue::Text(text) => {
                let image = self
                    .image_properties
                    .contains(name)
                    .then(|| self.file.images.iter().position(|i| &i.name == text))
                    .flatten();

                match image {
                    Some(index) => FormValue::Image(index as u32),
                    None => FormValue::Text(text.clone()),
                }
            }
        }
    }
}

/// Lists a composite's part IDs.
///
/// # Arguments
///
/// * `widget` - The widget, if it exists.
///
/// # Returns
///
/// The IDs of its parts, or none if it is not a composite.
fn part_ids(widget: Option<&dyn Widget>) -> Vec<WidgetId> {
    widget
        .and_then(|w| w.as_composite())
        .map(|c| c.parts().into_iter().map(|(_, id)| id).collect())
        .unwrap_or_default()
}

/// Converts an alignment to its file code.
///
/// # Arguments
///
/// * `align` - The alignment.
///
/// # Returns
///
/// `0` start, `1` center, `2` end, `3` fill.
fn align_code(align: Align) -> u8 {
    match align {
        Align::Start => 0,
        Align::Center => 1,
        Align::End => 2,
        Align::Fill => 3,
    }
}

/// Converts layout parameters to a child layout record.
///
/// # Arguments
///
/// * `params` - The widget's layout parameters.
///
/// # Returns
///
/// The record; a stretch above `u16::MAX` is stored as `u16::MAX`.
fn child_layout(params: &LayoutParams) -> ChildLayout {
    ChildLayout {
        stretch: u16::try_from(params.stretch).unwrap_or(u16::MAX),
        horizontal: align_code(params.alignment.horizontal),
        vertical: align_code(params.alignment.vertical),
        margins: [
            params.margin.left,
            params.margin.top,
            params.margin.right,
            params.margin.bottom,
        ],
        cell: [0; 4],
    }
}
