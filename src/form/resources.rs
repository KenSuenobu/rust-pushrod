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

//! # Resource kinds
//!
//! Besides forms, a form file carries **resources**: strings, images, icons, files, menus, and
//! alerts.  Each kind is a [`ResourceTable`] whose entries have a stable numeric ID and a name.
//! An ID is assigned once, starting at 1, and is never reused or renumbered: removing or
//! reordering entries leaves every other ID alone, so code and saved settings that hold an ID
//! stay valid.
//!
//! On disk the resources are one section of tag-length-value tables (see `docs/FORM_FORMAT.md`),
//! so a reader skips any table it does not know.  Reading never panics: every length, count, and
//! reference is checked and a damaged section is a [`FormError`].
//!
//! ```
//! use pushrod::form::{FormFile, Resources};
//!
//! let mut file = FormFile::new();
//! let hello = file.resources.strings.add("HELLO", "Hello".to_string()).unwrap();
//! let bye = file.resources.strings.add("BYE", "Goodbye".to_string()).unwrap();
//! file.resources.strings.remove(hello);
//!
//! let back = FormFile::from_bytes(&file.to_bytes().unwrap()).unwrap();
//! assert_eq!(back.resources.strings.get(bye).unwrap().value, "Goodbye");
//! assert!(back.resources.strings.get(hello).is_none());
//! let _: &Resources = &back.resources;
//! ```

use super::format::{FormError, Reader, put_u32, to_u32};

/// The resource table tags.  A reader skips any other tag.
pub(super) mod table_tag {
    /// The string table.
    pub const STRINGS: u16 = 1;
    /// The image table.
    pub const IMAGES: u16 = 2;
    /// The icon table.
    pub const ICONS: u16 = 3;
    /// The file table.
    pub const FILES: u16 = 4;
    /// The menu table.
    pub const MENUS: u16 = 5;
    /// The alert table.
    pub const ALERTS: u16 = 6;
}

/// The deepest a menu may nest submenus; deeper menus are rejected when read or written.
pub const MAX_MENU_DEPTH: usize = 16;

/// One resource: its stable ID, its name, and its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry<T> {
    /// The ID, assigned once and never reused.  IDs start at 1.
    pub id: u32,
    /// The name, unique within its table, such as `SAVE_FAILED`.
    pub name: String,
    /// The value.
    pub value: T,
}

/// A table of resources of one kind with stable IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTable<T> {
    entries: Vec<ResourceEntry<T>>,
    next_id: u32,
}

impl<T> Default for ResourceTable<T> {
    /// Creates an empty table whose first ID will be 1.
    ///
    /// # Returns
    ///
    /// The empty table.
    fn default() -> Self {
        ResourceTable {
            entries: Vec::new(),
            next_id: 1,
        }
    }
}

impl<T> ResourceTable<T> {
    /// Creates an empty table.
    ///
    /// # Returns
    ///
    /// The table; its first ID will be 1.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a resource under the next unused ID.
    ///
    /// # Arguments
    ///
    /// * `name` - The name; it must be unique in the table.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The new resource's ID.
    ///
    /// # Errors
    ///
    /// [`FormError::Invalid`] when the name is already used, and [`FormError::TooLarge`] when the
    /// IDs are exhausted.
    pub fn add(&mut self, name: &str, value: T) -> Result<u32, FormError> {
        let id = self.next_id;
        self.insert_with_id(id, name, value)?;
        Ok(id)
    }

    /// Adds a resource under a chosen ID, for importing a table saved elsewhere.
    ///
    /// The next ID moves past `id` when needed, so a later [`ResourceTable::add`] never collides.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID; it must be nonzero and unused.
    /// * `name` - The name; it must be unique in the table.
    /// * `value` - The value.
    ///
    /// # Errors
    ///
    /// [`FormError::Invalid`] for a zero, used, or reserved ID or a used name, and
    /// [`FormError::TooLarge`] when no ID follows `id`.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the resource was added.
    pub fn insert_with_id(&mut self, id: u32, name: &str, value: T) -> Result<(), FormError> {
        if id == 0 {
            return Err(FormError::invalid("resource ID 0 is reserved"));
        }
        if self.get(id).is_some() {
            return Err(FormError::invalid(format!(
                "resource ID {id} is already used"
            )));
        }
        if self.by_name(name).is_some() {
            return Err(FormError::invalid(format!(
                "resource name \"{name}\" is already used"
            )));
        }
        let following = id.checked_add(1).ok_or(FormError::TooLarge {
            what: "resource IDs",
        })?;
        self.next_id = self.next_id.max(following);
        self.entries.push(ResourceEntry {
            id,
            name: name.to_string(),
            value,
        });
        Ok(())
    }

    /// Removes a resource.  Its ID is never handed out again.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to remove.
    ///
    /// # Returns
    ///
    /// The removed entry, or `None` when there was none.
    pub fn remove(&mut self, id: u32) -> Option<ResourceEntry<T>> {
        let at = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.remove(at))
    }

    /// Moves a resource to another position; no ID changes.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to move.
    /// * `index` - The new position (clamped to the end).
    ///
    /// # Returns
    ///
    /// `true` when the resource exists.
    pub fn move_to(&mut self, id: u32, index: usize) -> bool {
        let Some(at) = self.entries.iter().position(|e| e.id == id) else {
            return false;
        };
        let entry = self.entries.remove(at);
        self.entries.insert(index.min(self.entries.len()), entry);
        true
    }

    /// Returns a resource by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID.
    ///
    /// # Returns
    ///
    /// The entry, or `None`.
    pub fn get(&self, id: u32) -> Option<&ResourceEntry<T>> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Returns a resource's value for editing.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID.
    ///
    /// # Returns
    ///
    /// The value, or `None`.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        self.entries
            .iter_mut()
            .find(|e| e.id == id)
            .map(|e| &mut e.value)
    }

    /// Returns a resource by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The entry, or `None`.
    pub fn by_name(&self, name: &str) -> Option<&ResourceEntry<T>> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Returns the entries in order.
    ///
    /// # Returns
    ///
    /// An iterator over the entries.
    pub fn iter(&self) -> std::slice::Iter<'_, ResourceEntry<T>> {
        self.entries.iter()
    }

    /// Returns the number of entries.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the table has no entries.
    ///
    /// # Returns
    ///
    /// `true` when empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the ID the next [`ResourceTable::add`] will use.
    ///
    /// # Returns
    ///
    /// The next ID.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// Checks that IDs are nonzero, unique, and below the next ID, and names are unique.
    fn validate(&self, what: &str) -> Result<(), FormError> {
        for (i, e) in self.entries.iter().enumerate() {
            if e.id == 0 || e.id >= self.next_id {
                return Err(FormError::invalid(format!(
                    "{what} ID {} is out of range",
                    e.id
                )));
            }
            if self.entries[..i]
                .iter()
                .any(|p| p.id == e.id || p.name == e.name)
            {
                return Err(FormError::invalid(format!(
                    "{what} \"{}\" ({}) is used twice",
                    e.name, e.id
                )));
            }
        }
        Ok(())
    }
}

impl<'a, T> IntoIterator for &'a ResourceTable<T> {
    type Item = &'a ResourceEntry<T>;
    type IntoIter = std::slice::Iter<'a, ResourceEntry<T>>;

    /// Iterates over the entries in order.
    ///
    /// # Returns
    ///
    /// The iterator.
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

/// An icon: an image plus optional mask and selected-state image, and a caption.
///
/// The images are IDs in the image table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconResource {
    /// The image drawn normally.
    pub image: u32,
    /// The image used as a mask, if any.
    pub mask: Option<u32>,
    /// The image drawn when the icon is selected, if any.
    pub selected: Option<u32>,
    /// The caption drawn with the icon.
    pub caption: String,
}

/// A named blob of bytes with a MIME type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResource {
    /// The MIME type, such as `text/plain`.
    pub mime: String,
    /// The bytes, stored untouched.
    pub data: Vec<u8>,
}

/// What a menu is used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    /// A menu bar: each top-level item is a title with a submenu.
    Bar,
    /// A context menu (a pop-up menu).
    Context,
}

/// One item of a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    /// An item that triggers an action.
    Command {
        /// The text shown.
        label: String,
        /// The name of the action a program binds to.
        action: String,
        /// The keyboard shortcut, such as `Ctrl+S`.
        shortcut: Option<String>,
        /// Whether the item can be chosen.
        enabled: bool,
    },
    /// A divider line.
    Separator,
    /// An item that opens a submenu.
    Submenu {
        /// The text shown.
        label: String,
        /// The items of the submenu.
        items: Vec<MenuItem>,
    },
}

impl MenuItem {
    /// Builds an enabled command with no shortcut.
    ///
    /// # Arguments
    ///
    /// * `label` - The text shown.
    /// * `action` - The action name.
    ///
    /// # Returns
    ///
    /// The item.
    pub fn command(label: &str, action: &str) -> Self {
        MenuItem::Command {
            label: label.to_string(),
            action: action.to_string(),
            shortcut: None,
            enabled: true,
        }
    }
}

/// A menu bar or context menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuResource {
    /// What the menu is used for.
    pub kind: MenuKind,
    /// The top-level items.
    pub items: Vec<MenuItem>,
}

/// How serious an alert is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    /// Information.
    Info,
    /// A warning.
    Warning,
    /// An error.
    Error,
    /// A question.
    Question,
}

/// An alert: its kind and the strings it shows, as IDs in the string table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertResource {
    /// How serious the alert is.
    pub kind: AlertKind,
    /// The title string.
    pub title: u32,
    /// The message string.
    pub message: u32,
    /// The button strings, in order.
    pub buttons: Vec<u32>,
}

/// Every resource table of a form file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resources {
    /// UTF-8 text.
    pub strings: ResourceTable<String>,
    /// Encoded image data (for example PNG).
    pub images: ResourceTable<Vec<u8>>,
    /// Icons.
    pub icons: ResourceTable<IconResource>,
    /// Opaque files.
    pub files: ResourceTable<FileResource>,
    /// Menu bars and context menus.
    pub menus: ResourceTable<MenuResource>,
    /// Alerts.
    pub alerts: ResourceTable<AlertResource>,
}

impl Resources {
    /// Returns whether every table is empty (no resource section is written).
    ///
    /// # Returns
    ///
    /// `true` when there are no resources.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
            && self.images.is_empty()
            && self.icons.is_empty()
            && self.files.is_empty()
            && self.menus.is_empty()
            && self.alerts.is_empty()
    }

    /// Checks IDs and names, that icons name existing images, that alerts name existing strings,
    /// and that menus nest no deeper than [`MAX_MENU_DEPTH`].
    ///
    /// # Errors
    ///
    /// [`FormError::Invalid`] naming the first problem.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the resources are consistent.
    pub fn validate(&self) -> Result<(), FormError> {
        self.strings.validate("string")?;
        self.images.validate("image")?;
        self.icons.validate("icon")?;
        self.files.validate("file")?;
        self.menus.validate("menu")?;
        self.alerts.validate("alert")?;
        for e in &self.icons {
            let refs = [Some(e.value.image), e.value.mask, e.value.selected];
            if let Some(n) = refs
                .into_iter()
                .flatten()
                .find(|n| self.images.get(*n).is_none())
            {
                return Err(FormError::invalid(format!(
                    "icon \"{}\" uses missing image {n}",
                    e.name
                )));
            }
        }
        for e in &self.alerts {
            let refs = [e.value.title, e.value.message]
                .into_iter()
                .chain(e.value.buttons.iter().copied());
            for n in refs {
                if self.strings.get(n).is_none() {
                    return Err(FormError::invalid(format!(
                        "alert \"{}\" uses missing string {n}",
                        e.name
                    )));
                }
            }
        }
        for e in &self.menus {
            if menu_depth(&e.value.items) > MAX_MENU_DEPTH {
                return Err(FormError::invalid(format!(
                    "menu \"{}\" nests deeper than {MAX_MENU_DEPTH} levels",
                    e.name
                )));
            }
        }
        Ok(())
    }

    /// Encodes the resource section: one tag-length-value entry per non-empty table.
    ///
    /// # Returns
    ///
    /// The section bytes.
    ///
    /// # Errors
    ///
    /// [`FormError::Invalid`] when the resources are inconsistent ([`Resources::validate`]) and
    /// [`FormError::TooLarge`] when a value exceeds the format's limits.
    pub(super) fn to_section(&self) -> Result<Vec<u8>, FormError> {
        self.validate()?;
        let mut out = Vec::new();
        write_table(&mut out, table_tag::STRINGS, &self.strings, |o, v| {
            write_str(o, v)
        })?;
        write_table(&mut out, table_tag::IMAGES, &self.images, |o, v| {
            write_bytes(o, v)
        })?;
        write_table(&mut out, table_tag::ICONS, &self.icons, |o, v| {
            put_u32(o, v.image);
            put_u32(o, v.mask.unwrap_or(0));
            put_u32(o, v.selected.unwrap_or(0));
            write_str(o, &v.caption)
        })?;
        write_table(&mut out, table_tag::FILES, &self.files, |o, v| {
            write_str(o, &v.mime)?;
            write_bytes(o, &v.data)
        })?;
        write_table(&mut out, table_tag::MENUS, &self.menus, |o, v| {
            o.push(match v.kind {
                MenuKind::Bar => 0,
                MenuKind::Context => 1,
            });
            write_items(o, &v.items)
        })?;
        write_table(&mut out, table_tag::ALERTS, &self.alerts, |o, v| {
            o.push(match v.kind {
                AlertKind::Info => 0,
                AlertKind::Warning => 1,
                AlertKind::Error => 2,
                AlertKind::Question => 3,
            });
            put_u32(o, v.title);
            put_u32(o, v.message);
            put_u32(o, to_u32(v.buttons.len(), "alert buttons")?);
            for b in &v.buttons {
                put_u32(o, *b);
            }
            Ok(())
        })?;
        Ok(out)
    }

    /// Decodes a resource section.  A table with a tag this version does not define is skipped.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The section bytes.
    ///
    /// # Returns
    ///
    /// The resources, validated.
    ///
    /// # Errors
    ///
    /// A [`FormError`] when the section is truncated or inconsistent.
    pub(super) fn from_section(bytes: &[u8]) -> Result<Resources, FormError> {
        let mut res = Resources::default();
        let mut seen = Vec::new();
        let mut r = Reader::new(bytes, "resource section");
        while !r.is_empty() {
            let tag = r.u16()?;
            let len = r.u32()? as usize;
            let body = r.take(len)?;
            if (table_tag::STRINGS..=table_tag::ALERTS).contains(&tag) {
                if seen.contains(&tag) {
                    return Err(FormError::invalid(format!(
                        "resource table {tag} appears twice"
                    )));
                }
                seen.push(tag);
            }
            match tag {
                table_tag::STRINGS => res.strings = read_table(body, read_str)?,
                table_tag::IMAGES => res.images = read_table(body, read_bytes)?,
                table_tag::ICONS => {
                    res.icons = read_table(body, |r| {
                        let image = r.u32()?;
                        let mask = r.u32()?;
                        let selected = r.u32()?;
                        Ok(IconResource {
                            image,
                            mask: (mask != 0).then_some(mask),
                            selected: (selected != 0).then_some(selected),
                            caption: read_str(r)?,
                        })
                    })?;
                }
                table_tag::FILES => {
                    res.files = read_table(body, |r| {
                        Ok(FileResource {
                            mime: read_str(r)?,
                            data: read_bytes(r)?,
                        })
                    })?;
                }
                table_tag::MENUS => {
                    res.menus = read_table(body, |r| {
                        let kind = match r.u8()? {
                            0 => MenuKind::Bar,
                            1 => MenuKind::Context,
                            other => {
                                return Err(FormError::invalid(format!(
                                    "unknown menu kind {other}"
                                )));
                            }
                        };
                        Ok(MenuResource {
                            kind,
                            items: read_items(r, 1)?,
                        })
                    })?;
                }
                table_tag::ALERTS => {
                    res.alerts = read_table(body, |r| {
                        let kind = match r.u8()? {
                            0 => AlertKind::Info,
                            1 => AlertKind::Warning,
                            2 => AlertKind::Error,
                            3 => AlertKind::Question,
                            other => {
                                return Err(FormError::invalid(format!(
                                    "unknown alert kind {other}"
                                )));
                            }
                        };
                        let title = r.u32()?;
                        let message = r.u32()?;
                        let count = r.count(4)?;
                        let mut buttons = Vec::with_capacity(count);
                        for _ in 0..count {
                            buttons.push(r.u32()?);
                        }
                        Ok(AlertResource {
                            kind,
                            title,
                            message,
                            buttons,
                        })
                    })?;
                }
                _ => {} // An unknown table: its length was consumed, so it is skipped.
            }
        }
        res.validate()?;
        Ok(res)
    }
}

/// Item kinds inside a menu.
mod item_kind {
    /// A command.
    pub const COMMAND: u8 = 0;
    /// A separator.
    pub const SEPARATOR: u8 = 1;
    /// A submenu.
    pub const SUBMENU: u8 = 2;
    /// Command flag: the item is enabled.
    pub const ENABLED: u8 = 1;
    /// Command flag: a shortcut follows.
    pub const HAS_SHORTCUT: u8 = 2;
}

/// Returns how many levels of items a list nests (a flat list is 1).
fn menu_depth(items: &[MenuItem]) -> usize {
    1 + items
        .iter()
        .map(|i| match i {
            MenuItem::Submenu { items, .. } => menu_depth(items),
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

/// Appends a length-prefixed UTF-8 string.
fn write_str(out: &mut Vec<u8>, s: &str) -> Result<(), FormError> {
    put_u32(out, to_u32(s.len(), "string")?);
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

/// Appends length-prefixed bytes.
fn write_bytes(out: &mut Vec<u8>, b: &[u8]) -> Result<(), FormError> {
    put_u32(out, to_u32(b.len(), "resource data")?);
    out.extend_from_slice(b);
    Ok(())
}

/// Reads a length-prefixed UTF-8 string.
fn read_str(r: &mut Reader<'_>) -> Result<String, FormError> {
    let len = r.u32()? as usize;
    std::str::from_utf8(r.take(len)?)
        .map(str::to_string)
        .map_err(|_| FormError::invalid("a resource string is not valid UTF-8"))
}

/// Reads length-prefixed bytes.
fn read_bytes(r: &mut Reader<'_>) -> Result<Vec<u8>, FormError> {
    let len = r.u32()? as usize;
    Ok(r.take(len)?.to_vec())
}

/// Appends a menu item list: a count, then each item as kind, length, and body.
fn write_items(out: &mut Vec<u8>, items: &[MenuItem]) -> Result<(), FormError> {
    put_u32(out, to_u32(items.len(), "menu items")?);
    for item in items {
        let mut body = Vec::new();
        let kind = match item {
            MenuItem::Command {
                label,
                action,
                shortcut,
                enabled,
            } => {
                write_str(&mut body, label)?;
                write_str(&mut body, action)?;
                body.push(
                    if *enabled { item_kind::ENABLED } else { 0 }
                        | if shortcut.is_some() {
                            item_kind::HAS_SHORTCUT
                        } else {
                            0
                        },
                );
                if let Some(s) = shortcut {
                    write_str(&mut body, s)?;
                }
                item_kind::COMMAND
            }
            MenuItem::Separator => item_kind::SEPARATOR,
            MenuItem::Submenu { label, items } => {
                write_str(&mut body, label)?;
                write_items(&mut body, items)?;
                item_kind::SUBMENU
            }
        };
        out.push(kind);
        put_u32(out, to_u32(body.len(), "menu item")?);
        out.extend_from_slice(&body);
    }
    Ok(())
}

/// Reads a menu item list; an item kind this version does not define is skipped.
fn read_items(r: &mut Reader<'_>, depth: usize) -> Result<Vec<MenuItem>, FormError> {
    if depth > MAX_MENU_DEPTH {
        return Err(FormError::invalid("a menu nests too deeply"));
    }
    let count = r.count(5)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.u8()?;
        let len = r.u32()? as usize;
        let mut body = Reader::new(r.take(len)?, "menu item");
        match kind {
            item_kind::COMMAND => {
                let label = read_str(&mut body)?;
                let action = read_str(&mut body)?;
                let flags = body.u8()?;
                let shortcut = if flags & item_kind::HAS_SHORTCUT != 0 {
                    Some(read_str(&mut body)?)
                } else {
                    None
                };
                out.push(MenuItem::Command {
                    label,
                    action,
                    shortcut,
                    enabled: flags & item_kind::ENABLED != 0,
                });
            }
            item_kind::SEPARATOR => out.push(MenuItem::Separator),
            item_kind::SUBMENU => {
                let label = read_str(&mut body)?;
                out.push(MenuItem::Submenu {
                    label,
                    items: read_items(&mut body, depth + 1)?,
                });
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Appends a table as a tag-length-value entry; an empty table writes nothing.
fn write_table<T>(
    out: &mut Vec<u8>,
    tag: u16,
    table: &ResourceTable<T>,
    mut write_value: impl FnMut(&mut Vec<u8>, &T) -> Result<(), FormError>,
) -> Result<(), FormError> {
    if table.is_empty() {
        return Ok(());
    }
    let mut body = Vec::new();
    put_u32(&mut body, table.next_id);
    put_u32(&mut body, to_u32(table.len(), "resource table")?);
    for e in table {
        put_u32(&mut body, e.id);
        write_str(&mut body, &e.name)?;
        write_value(&mut body, &e.value)?;
    }
    out.extend_from_slice(&tag.to_le_bytes());
    put_u32(out, to_u32(body.len(), "resource table")?);
    out.extend_from_slice(&body);
    Ok(())
}

/// Reads a table body: the next ID, a count, and each entry.
fn read_table<T>(
    body: &[u8],
    mut read_value: impl FnMut(&mut Reader<'_>) -> Result<T, FormError>,
) -> Result<ResourceTable<T>, FormError> {
    let mut r = Reader::new(body, "resource table");
    let next_id = r.u32()?;
    let count = r.count(8)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.u32()?;
        let name = read_str(&mut r)?;
        let value = read_value(&mut r)?;
        entries.push(ResourceEntry { id, name, value });
    }
    Ok(ResourceTable { entries, next_id })
}
