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

//! # Resources
//!
//! The runtime resource manager.  A [`Resources`] holds one or more loaded resource files
//! ([`ResourceSet`]s) and gets forms, strings, images, icons, files, menus, and alerts out of
//! them by typed ID ([`id`]): an ID of one kind cannot be used as another, so asking for an image
//! with a string's ID does not compile.
//!
//! Sets stack: [`Resources::with_fallback`] adds a lower-priority set, and every lookup searches
//! the sets in order, so an application set overrides defaults shipped by a library set.
//!
//! ```
//! use pushrod::form::FormFile;
//! use pushrod::resource::{Resources, StringId};
//!
//! let mut file = FormFile::new();
//! let hello = file.resources.strings.add("HELLO", "Hello".to_string()).unwrap();
//!
//! let resources = Resources::from_bytes(&file.to_bytes().unwrap()).unwrap();
//! assert_eq!(resources.string(StringId(hello)), Some("Hello"));
//! assert_eq!(resources.find::<StringId>("HELLO"), Some(StringId(hello)));
//! ```
//!
//! An [`Engine`](crate::engine::Engine) owns the set it is given with
//! [`Engine::set_resources`](crate::engine::Engine::set_resources), emits
//! [`EngineEvent::ResourcesLoaded`](crate::event::EngineEvent::ResourcesLoaded) and
//! [`EngineEvent::ResourcesFreed`](crate::event::EngineEvent::ResourcesFreed), and builds forms
//! with [`Engine::load_resource_form`](crate::engine::Engine::load_resource_form).

pub mod id;
pub mod set;

pub use id::{AlertId, FileId, FormId, FormIndex, IconId, ImageId, MenuId, StringId};
pub use set::{IconHandle, ImageHandle, ResourceSet};

use crate::error::Result;
use crate::form::{AlertResource, MenuResource};

/// A resource ID type that can be looked up by name.
pub trait ResourceKind: Sized {
    /// Finds the ID of the resource with a name in one set.
    ///
    /// # Arguments
    ///
    /// * `set` - The set to search.
    /// * `name` - The resource's name.
    ///
    /// # Returns
    ///
    /// The ID, or `None` if the set has no such resource.
    fn find_in(set: &ResourceSet, name: &str) -> Option<Self>;
}

/// Implements [`ResourceKind`] for an ID type from the form file table that holds it.
///
/// # Arguments
///
/// * `$id` - The ID type.
/// * `$table` - The table field of the file's resources.
macro_rules! kind_by_name {
    ($($id:ident => $table:ident),+ $(,)?) => {$(
        impl ResourceKind for $id {
            fn find_in(set: &ResourceSet, name: &str) -> Option<Self> {
                set.file().resources.$table.by_name(name).map(|e| $id(e.id))
            }
        }
    )+};
}

kind_by_name!(
    StringId => strings,
    ImageId => images,
    IconId => icons,
    FileId => files,
    MenuId => menus,
    AlertId => alerts,
);

impl ResourceKind for FormId {
    fn find_in(set: &ResourceSet, name: &str) -> Option<Self> {
        let index = set.file().forms.iter().position(|f| f.name == name)?;

        Some(FormId(u32::try_from(index).ok()? + 1))
    }
}

/// The loaded resource sets of an application, searched in order.
#[derive(Debug, Default)]
pub struct Resources {
    sets: Vec<ResourceSet>,
}

impl Resources {
    /// Creates a manager with no sets.
    ///
    /// # Returns
    ///
    /// An empty manager; every lookup finds nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a resource file.
    ///
    /// # Arguments
    ///
    /// * `path` - The resource file.
    ///
    /// # Returns
    ///
    /// A manager holding the file's set.
    ///
    /// # Errors
    ///
    /// [`Error::FormFile`](crate::error::Error::FormFile) if the file cannot be read or is
    /// malformed.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Ok(Self {
            sets: vec![ResourceSet::load(path)?],
        })
    }

    /// Loads a resource file embedded in the executable.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The file's contents, usually `include_bytes!("app.prsc")`.
    ///
    /// # Returns
    ///
    /// A manager holding the set; it gives the same results as [`load`](Resources::load).
    ///
    /// # Errors
    ///
    /// [`Error::FormFile`](crate::error::Error::FormFile) if the bytes are malformed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            sets: vec![ResourceSet::from_bytes(bytes)?],
        })
    }

    /// Adds every set of `other` after this manager's own, as lower-priority fallbacks.
    ///
    /// # Arguments
    ///
    /// * `other` - The sets to add, searched after the ones already here.
    ///
    /// # Returns
    ///
    /// The combined manager.
    pub fn with_fallback(mut self, other: Resources) -> Self {
        self.sets.extend(other.sets);
        self
    }

    /// Returns how many sets are loaded.
    ///
    /// # Returns
    ///
    /// The number of sets.
    pub fn sets(&self) -> usize {
        self.sets.len()
    }

    /// Returns the sets in search order.
    ///
    /// # Returns
    ///
    /// An iterator over the sets, highest priority first.
    pub fn iter(&self) -> std::slice::Iter<'_, ResourceSet> {
        self.sets.iter()
    }

    /// Finds the first set that has a form, and the form's name.
    ///
    /// # Arguments
    ///
    /// * `id` - The form's ID.
    ///
    /// # Returns
    ///
    /// The set and the form's name, or `None` if no set has the form.
    pub fn form(&self, id: FormId) -> Option<(&ResourceSet, &str)> {
        self.sets
            .iter()
            .find_map(|set| set.form_name(id).map(|name| (set, name)))
    }

    /// Returns a string.
    ///
    /// # Arguments
    ///
    /// * `id` - The string's ID.
    ///
    /// # Returns
    ///
    /// The text from the first set that has it, or `None`.
    pub fn string(&self, id: StringId) -> Option<&str> {
        self.sets.iter().find_map(|set| set.string(id))
    }

    /// Returns an image.
    ///
    /// # Arguments
    ///
    /// * `id` - The image's ID.
    ///
    /// # Returns
    ///
    /// A shared handle from the first set that has it, or `None`.
    pub fn image(&self, id: ImageId) -> Option<ImageHandle> {
        self.sets.iter().find_map(|set| set.image(id))
    }

    /// Returns an icon.
    ///
    /// # Arguments
    ///
    /// * `id` - The icon's ID.
    ///
    /// # Returns
    ///
    /// The icon from the first set that has it, or `None`.
    pub fn icon(&self, id: IconId) -> Option<IconHandle> {
        self.sets.iter().find_map(|set| set.icon(id))
    }

    /// Returns a file's bytes.
    ///
    /// # Arguments
    ///
    /// * `id` - The file's ID.
    ///
    /// # Returns
    ///
    /// The bytes from the first set that has the file, or `None`.
    pub fn file(&self, id: FileId) -> Option<&[u8]> {
        self.sets.iter().find_map(|set| set.file_data(id))
    }

    /// Returns a menu.
    ///
    /// # Arguments
    ///
    /// * `id` - The menu's ID.
    ///
    /// # Returns
    ///
    /// The menu from the first set that has it, or `None`.
    pub fn menu(&self, id: MenuId) -> Option<&MenuResource> {
        self.sets.iter().find_map(|set| set.menu(id))
    }

    /// Returns an alert.
    ///
    /// # Arguments
    ///
    /// * `id` - The alert's ID.
    ///
    /// # Returns
    ///
    /// The alert from the first set that has it, or `None`.
    pub fn alert(&self, id: AlertId) -> Option<&AlertResource> {
        self.sets.iter().find_map(|set| set.alert(id))
    }

    /// Finds a resource's ID by name, for tools and debugging; applications use the generated
    /// constants.
    ///
    /// # Arguments
    ///
    /// * `name` - The resource's name.
    ///
    /// # Returns
    ///
    /// The ID from the first set that has a resource of that kind and name, or `None`.
    pub fn find<K: ResourceKind>(&self, name: &str) -> Option<K> {
        self.sets.iter().find_map(|set| K::find_in(set, name))
    }
}
