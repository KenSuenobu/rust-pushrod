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

//! # One loaded resource set
//!
//! A [`ResourceSet`] is one resource file, read and checked once.  Strings, files, menus, and
//! alerts are read straight from it; each image is kept as shared bytes behind an
//! [`ImageHandle`], so every widget that asks for the same image gets the same handle and the
//! backend uploads it to a GPU texture once, under the handle's [`key`](ImageHandle::key).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::form::{AlertResource, FormFile, MenuResource};

use super::id::{AlertId, FileId, FormId, IconId, ImageId, MenuId, StringId};

/// Numbers sets as they load, so image keys from different sets never collide.
static NEXT_SET: AtomicU64 = AtomicU64::new(1);

/// An image resource: shared encoded bytes and the key a backend caches its texture under.
///
/// Cloning is cheap and shares the bytes, so two widgets showing one image share one handle.
#[derive(Debug, Clone)]
pub struct ImageHandle {
    key: Arc<str>,
    data: Arc<[u8]>,
}

impl ImageHandle {
    /// Returns the key that names this image in a backend's texture cache.
    ///
    /// # Returns
    ///
    /// A key unique to the image and its set, such as `resource:3/image/1`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the encoded image bytes (for example a PNG file), exactly as stored.
    ///
    /// # Returns
    ///
    /// The bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns whether two handles are the same image, shared rather than copied.
    ///
    /// # Arguments
    ///
    /// * `other` - The other handle.
    ///
    /// # Returns
    ///
    /// `true` if both handles share one set of bytes.
    pub fn is_same(&self, other: &ImageHandle) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

/// An icon resource: its image, optional mask and selected-state image, and caption.
#[derive(Debug, Clone)]
pub struct IconHandle {
    /// The icon's image.
    pub image: ImageHandle,
    /// The mask image, if the icon has one.
    pub mask: Option<ImageHandle>,
    /// The image shown when selected, if the icon has one.
    pub selected: Option<ImageHandle>,
    /// The caption shown with the icon.
    pub caption: String,
}

/// One resource file, loaded and checked.
#[derive(Debug)]
pub struct ResourceSet {
    file: FormFile,
    images: HashMap<u32, ImageHandle>,
    icons: HashMap<u32, IconHandle>,
}

impl ResourceSet {
    /// Reads a resource set from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The resource file.
    ///
    /// # Returns
    ///
    /// The loaded set.
    ///
    /// # Errors
    ///
    /// [`Error::FormFile`] if the file cannot be read or is not a valid resource file.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let name = path.display().to_string();
        let bytes = std::fs::read(path).map_err(|e| Error::form_file(&name, e.to_string()))?;

        Self::from_bytes(&bytes).map_err(|e| match e {
            Error::FormFile { message, .. } => Error::form_file(name, message),
            other => other,
        })
    }

    /// Reads a resource set from bytes, such as `include_bytes!("app.prsc")`.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The contents of a resource file.
    ///
    /// # Returns
    ///
    /// The loaded set.
    ///
    /// # Errors
    ///
    /// [`Error::FormFile`] if the bytes are not a valid resource file.  Nothing panics on
    /// malformed input.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let file =
            FormFile::from_bytes(bytes).map_err(|e| Error::form_file("<bytes>", e.to_string()))?;

        Self::from_file(file)
    }

    /// Builds a set from a form file already in memory.
    ///
    /// # Arguments
    ///
    /// * `file` - The form file; its forms and resource tables become the set.
    ///
    /// # Returns
    ///
    /// The loaded set.
    ///
    /// # Errors
    ///
    /// [`Error::FormFile`] if the file or its resource tables are inconsistent.
    pub fn from_file(file: FormFile) -> Result<Self> {
        file.validate()
            .and_then(|()| file.resources.validate())
            .map_err(|e| Error::form_file("<bytes>", e.to_string()))?;

        let serial = NEXT_SET.fetch_add(1, Ordering::Relaxed);
        let images: HashMap<u32, ImageHandle> = file
            .resources
            .images
            .iter()
            .map(|entry| {
                let handle = ImageHandle {
                    key: format!("resource:{serial}/image/{}", entry.id).into(),
                    data: Arc::from(entry.value.as_slice()),
                };

                (entry.id, handle)
            })
            .collect();
        let icons = file
            .resources
            .icons
            .iter()
            .filter_map(|entry| {
                let icon = &entry.value;
                let handle = IconHandle {
                    image: images.get(&icon.image)?.clone(),
                    mask: icon.mask.and_then(|id| images.get(&id).cloned()),
                    selected: icon.selected.and_then(|id| images.get(&id).cloned()),
                    caption: icon.caption.clone(),
                };

                Some((entry.id, handle))
            })
            .collect();

        Ok(Self {
            file,
            images,
            icons,
        })
    }

    /// Returns the form file behind the set.
    ///
    /// # Returns
    ///
    /// The file.
    pub fn file(&self) -> &FormFile {
        &self.file
    }

    /// Returns a form's name.
    ///
    /// # Arguments
    ///
    /// * `id` - The form's ID (its position in the file, from 1).
    ///
    /// # Returns
    ///
    /// The name, or `None` if the set has no such form.
    pub fn form_name(&self, id: FormId) -> Option<&str> {
        let index = usize::try_from(id.get()).ok()?.checked_sub(1)?;

        self.file.forms.get(index).map(|f| f.name.as_str())
    }

    /// Returns a string.
    ///
    /// # Arguments
    ///
    /// * `id` - The string's ID.
    ///
    /// # Returns
    ///
    /// The text, or `None` if the set has no such string.
    pub fn string(&self, id: StringId) -> Option<&str> {
        self.file
            .resources
            .strings
            .get(id.get())
            .map(|e| e.value.as_str())
    }

    /// Returns an image.
    ///
    /// # Arguments
    ///
    /// * `id` - The image's ID.
    ///
    /// # Returns
    ///
    /// A shared handle, or `None` if the set has no such image.
    pub fn image(&self, id: ImageId) -> Option<ImageHandle> {
        self.images.get(&id.get()).cloned()
    }

    /// Returns an icon.
    ///
    /// # Arguments
    ///
    /// * `id` - The icon's ID.
    ///
    /// # Returns
    ///
    /// The icon, or `None` if the set has no such icon.
    pub fn icon(&self, id: IconId) -> Option<IconHandle> {
        self.icons.get(&id.get()).cloned()
    }

    /// Returns a file's bytes.
    ///
    /// # Arguments
    ///
    /// * `id` - The file's ID.
    ///
    /// # Returns
    ///
    /// The bytes, or `None` if the set has no such file.
    pub fn file_data(&self, id: FileId) -> Option<&[u8]> {
        self.file
            .resources
            .files
            .get(id.get())
            .map(|e| e.value.data.as_slice())
    }

    /// Returns a menu.
    ///
    /// # Arguments
    ///
    /// * `id` - The menu's ID.
    ///
    /// # Returns
    ///
    /// The menu, or `None` if the set has no such menu.
    pub fn menu(&self, id: MenuId) -> Option<&MenuResource> {
        self.file.resources.menus.get(id.get()).map(|e| &e.value)
    }

    /// Returns an alert.
    ///
    /// # Arguments
    ///
    /// * `id` - The alert's ID.
    ///
    /// # Returns
    ///
    /// The alert, or `None` if the set has no such alert.
    pub fn alert(&self, id: AlertId) -> Option<&AlertResource> {
        self.file.resources.alerts.get(id.get()).map(|e| &e.value)
    }
}
