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

//! # The form file format
//!
//! [`FormFile`] is the in-memory form of a form file: a table of forms, a table of objects (the
//! widgets, linked into trees by object number), an image pool, and a symbol table.  Strings are
//! plain `String`s here; the writer builds the string pool and the reader resolves it, so the
//! pool is only a detail of the bytes.
//!
//! The byte layout is specified in `docs/FORM_FORMAT.md`.  In short, everything is little-endian,
//! the header holds the magic `PRSC`, the version, and the offset and size of each section, and
//! each object's properties are a block of tag-length-value entries so a reader skips tags it
//! does not know and old readers keep working when new properties are added.
//!
//! Reading never panics: every offset, length, and object number is checked, and a damaged file
//! is a [`FormError`].
//!
//! ```
//! use pushrod::form::{FormFile, FormValue, ObjectRecord, Property};
//!
//! let mut file = FormFile::new();
//! let mut button = ObjectRecord::new("pushrod.button");
//! button.properties.push(Property::named("text", FormValue::Text("OK".to_string())));
//! let root = file.add_object(None, button);
//! file.add_form("Sample", root);
//!
//! let bytes = file.to_bytes().unwrap();
//! assert_eq!(&bytes[0..4], b"PRSC");
//! assert_eq!(FormFile::from_bytes(&bytes).unwrap(), file);
//! ```

use crate::color::Color;
use std::collections::HashMap;
use std::fmt;

/// The four bytes every form file starts with.
pub const MAGIC: [u8; 4] = *b"PRSC";

/// The format version this library writes and the newest it reads.
pub const FORMAT_VERSION: u16 = 1;

/// The object number that means "no object" (no parent, no child, no sibling).
pub const NO_OBJECT: u32 = u32::MAX;

/// The property-block offset that means "this object has no properties".
const NO_PROPERTIES: u32 = u32::MAX;

/// The size of the header in bytes.
const HEADER_LEN: usize = 56;

/// The size of one object record in bytes.
const OBJECT_LEN: usize = 48;

/// The property tags defined by version 1.  Tags are `u16`; readers skip any other tag.
mod tag {
    /// A property stored by name: name (string index), value kind, value.
    pub const NAMED: u16 = 1;
    /// The layout a container applies to its children.
    pub const LAYOUT: u16 = 2;
    /// The layout parameters of a child inside its parent's layout.
    pub const CHILD_LAYOUT: u16 = 3;
}

/// The value kinds of a named property.
mod kind {
    /// A true or false value: one byte.
    pub const BOOL: u8 = 0;
    /// A whole number: `i64`.
    pub const INT: u8 = 1;
    /// A number with a fractional part: `f64`.
    pub const FLOAT: u8 = 2;
    /// Text: a string index.
    pub const TEXT: u8 = 3;
    /// A color: red, green, blue, alpha bytes.
    pub const COLOR: u8 = 4;
    /// An image: an index into the image pool.
    pub const IMAGE: u8 = 5;
}

/// A failure reading or writing a form file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormError {
    /// The bytes do not start with the `PRSC` magic.
    BadMagic,
    /// The file is a newer version than this library reads.
    UnsupportedVersion {
        /// The version the file declares.
        version: u16,
    },
    /// The file ends before something it declares.
    Truncated {
        /// What was being read.
        what: &'static str,
    },
    /// The file is internally inconsistent (a bad index, a cycle, a broken link, bad UTF-8).
    Invalid {
        /// What is wrong.
        message: String,
    },
    /// A value is too large for the format (more than `u32::MAX` entries or bytes).
    TooLarge {
        /// What was too large.
        what: &'static str,
    },
}

impl FormError {
    /// Builds a [`FormError::Invalid`].
    ///
    /// # Arguments
    ///
    /// * `message` - What is wrong.
    ///
    /// # Returns
    ///
    /// The error.
    fn invalid(message: impl Into<String>) -> Self {
        FormError::Invalid {
            message: message.into(),
        }
    }
}

impl fmt::Display for FormError {
    /// Writes a one-line, human-readable description of the error.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormError::BadMagic => write!(f, "not a form file (missing PRSC magic)"),
            FormError::UnsupportedVersion { version } => write!(
                f,
                "form file version {version} is newer than the supported version {FORMAT_VERSION}"
            ),
            FormError::Truncated { what } => {
                write!(f, "form file is truncated while reading {what}")
            }
            FormError::Invalid { message } => write!(f, "invalid form file: {message}"),
            FormError::TooLarge { what } => {
                write!(f, "{what} is too large for the form file format")
            }
        }
    }
}

impl std::error::Error for FormError {}

/// A typed value of a named property.
#[derive(Debug, Clone, PartialEq)]
pub enum FormValue {
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
    /// An image, by its number in the image pool.
    Image(u32),
}

/// The layout a container applies to its children: a layout kind and its parameters.
///
/// The kind codes are defined in `docs/FORM_FORMAT.md`; the format stores them without
/// interpreting them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayoutSpec {
    /// The layout kind code (`0` is none).
    pub kind: u8,
    /// The kind's parameters (spacing, columns, and so on).
    pub params: Vec<i32>,
}

/// A child's parameters inside its parent's layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChildLayout {
    /// The share of spare space the child takes (`0` takes none).
    pub stretch: u16,
    /// The horizontal alignment code.
    pub horizontal: u8,
    /// The vertical alignment code.
    pub vertical: u8,
    /// The margins: left, top, right, bottom.
    pub margins: [u32; 4],
    /// The grid cell: column, row, column span, row span.
    pub cell: [u16; 4],
}

/// One entry of an object's property block.
#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    /// A property stored by name, such as `text` or `value`.
    Named {
        /// The property name.
        name: String,
        /// The value.
        value: FormValue,
    },
    /// The layout this container applies to its children.
    Layout(LayoutSpec),
    /// This child's layout parameters.
    ChildLayout(ChildLayout),
}

impl Property {
    /// Builds a named property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The property.
    pub fn named(name: &str, value: FormValue) -> Self {
        Property::Named {
            name: name.to_string(),
            value,
        }
    }
}

/// One widget in the object table.  Its number is its index in [`FormFile::objects`].
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectRecord {
    /// The registered type name, such as `pushrod.button`.
    pub type_name: String,
    /// The parent's object number, or [`NO_OBJECT`] for a root.
    pub parent: u32,
    /// The first child's object number, or [`NO_OBJECT`].
    pub first_child: u32,
    /// The next sibling's object number, or [`NO_OBJECT`].
    pub next_sibling: u32,
    /// The left edge, relative to the parent.
    pub x: i32,
    /// The top edge, relative to the parent.
    pub y: i32,
    /// The width.
    pub width: u32,
    /// The height.
    pub height: u32,
    /// The behavior flags.
    pub behavior: u32,
    /// The state flags.
    pub state: u32,
    /// The anchors (which edges follow the parent).
    pub anchors: u32,
    /// The properties, in order.
    pub properties: Vec<Property>,
}

impl ObjectRecord {
    /// Creates an object with no links, no size, and no properties.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The registered type name.
    ///
    /// # Returns
    ///
    /// The record.
    pub fn new(type_name: &str) -> Self {
        ObjectRecord {
            type_name: type_name.to_string(),
            parent: NO_OBJECT,
            first_child: NO_OBJECT,
            next_sibling: NO_OBJECT,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            behavior: 0,
            state: 0,
            anchors: 0,
            properties: Vec::new(),
        }
    }
}

/// One entry of the form table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormEntry {
    /// The form's name.
    pub name: String,
    /// The object number of the form's root.
    pub root: u32,
}

/// One entry of the image pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageEntry {
    /// The image's name.
    pub name: String,
    /// The encoded image bytes (for example a PNG file), stored untouched.
    pub data: Vec<u8>,
}

/// What a [`Symbol`] names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// The name of an object number (an ID constant).
    Id,
    /// The name of an action a program binds to.
    Action,
}

/// One entry of the symbol table: a stable number and its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// What the symbol names.
    pub kind: SymbolKind,
    /// The number: an object number for [`SymbolKind::Id`], an action number otherwise.
    pub id: u32,
    /// The name, such as `OK_BUTTON` or `save`.
    pub name: String,
}

/// A whole form file in memory.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormFile {
    /// The forms: a name and the root object of each.
    pub forms: Vec<FormEntry>,
    /// The object table; an object's number is its index.
    pub objects: Vec<ObjectRecord>,
    /// The image pool; an image's number is its index.
    pub images: Vec<ImageEntry>,
    /// The symbol table.
    pub symbols: Vec<Symbol>,
}

impl FormFile {
    /// Creates an empty form file.
    ///
    /// # Returns
    ///
    /// The empty file.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an object, linking it as the last child of `parent`.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent's object number, or `None` for a root.  A number that is not an
    ///   object leaves the record unlinked; [`FormFile::validate`] reports it.
    /// * `record` - The object.  Its `parent`, `first_child`, and `next_sibling` are replaced.
    ///
    /// # Returns
    ///
    /// The new object's number.
    pub fn add_object(&mut self, parent: Option<u32>, mut record: ObjectRecord) -> u32 {
        let number = self.objects.len() as u32;
        record.first_child = NO_OBJECT;
        record.next_sibling = NO_OBJECT;
        record.parent = parent.unwrap_or(NO_OBJECT);
        let last = parent.and_then(|p| self.children(p).last().copied());
        self.objects.push(record);
        if let Some(parent) = parent {
            if let Some(last) = last {
                self.objects[last as usize].next_sibling = number;
            } else if let Some(p) = self.objects.get_mut(parent as usize) {
                p.first_child = number;
            }
        }
        number
    }

    /// Adds a form to the form table.
    ///
    /// # Arguments
    ///
    /// * `name` - The form's name.
    /// * `root` - The object number of the form's root.
    pub fn add_form(&mut self, name: &str, root: u32) {
        self.forms.push(FormEntry {
            name: name.to_string(),
            root,
        });
    }

    /// Adds an image to the pool.
    ///
    /// # Arguments
    ///
    /// * `name` - The image's name.
    /// * `data` - The encoded image bytes.
    ///
    /// # Returns
    ///
    /// The image's number in the pool.
    pub fn add_image(&mut self, name: &str, data: Vec<u8>) -> u32 {
        self.images.push(ImageEntry {
            name: name.to_string(),
            data,
        });
        (self.images.len() - 1) as u32
    }

    /// Adds a symbol.
    ///
    /// # Arguments
    ///
    /// * `kind` - What the symbol names.
    /// * `id` - The number.
    /// * `name` - The name.
    pub fn add_symbol(&mut self, kind: SymbolKind, id: u32, name: &str) {
        self.symbols.push(Symbol {
            kind,
            id,
            name: name.to_string(),
        });
    }

    /// Returns the form with a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The form's name.
    ///
    /// # Returns
    ///
    /// The entry, or `None`.
    pub fn form(&self, name: &str) -> Option<&FormEntry> {
        self.forms.iter().find(|f| f.name == name)
    }

    /// Returns the object numbers of an object's children, in order.
    ///
    /// # Arguments
    ///
    /// * `number` - The parent's object number.
    ///
    /// # Returns
    ///
    /// The children (empty for a leaf, an unknown number, or a damaged chain).
    pub fn children(&self, number: u32) -> Vec<u32> {
        let mut out = Vec::new();
        let mut next = self
            .objects
            .get(number as usize)
            .map_or(NO_OBJECT, |o| o.first_child);
        while next != NO_OBJECT && out.len() <= self.objects.len() {
            out.push(next);
            next = self
                .objects
                .get(next as usize)
                .map_or(NO_OBJECT, |o| o.next_sibling);
        }
        out
    }

    /// Returns the object number of a symbol name.
    ///
    /// # Arguments
    ///
    /// * `name` - The symbol name.
    ///
    /// # Returns
    ///
    /// The number of the first [`SymbolKind::Id`] symbol with the name, or `None`.
    pub fn id_of(&self, name: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|s| s.kind == SymbolKind::Id && s.name == name)
            .map(|s| s.id)
    }

    /// Checks that the file is consistent: every number is in range, every link agrees with its
    /// parent, no object is reached twice, and every image reference exists.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the file is consistent.
    ///
    /// # Errors
    ///
    /// [`FormError::Invalid`] naming the first problem.
    pub fn validate(&self) -> Result<(), FormError> {
        let count = self.objects.len();
        let in_range = |n: u32| n == NO_OBJECT || (n as usize) < count;
        for (i, o) in self.objects.iter().enumerate() {
            if !in_range(o.parent) || !in_range(o.first_child) || !in_range(o.next_sibling) {
                return Err(FormError::invalid(format!(
                    "object {i} links to a missing object"
                )));
            }
            for p in &o.properties {
                if let Property::Named {
                    value: FormValue::Image(n),
                    ..
                } = p
                    && (*n as usize) >= self.images.len()
                {
                    return Err(FormError::invalid(format!(
                        "object {i} uses missing image {n}"
                    )));
                }
            }
        }
        // Walk every child chain: each child must name its parent and be reached once.
        let mut seen = vec![false; count];
        for (i, o) in self.objects.iter().enumerate() {
            let mut next = o.first_child;
            while next != NO_OBJECT {
                let n = next as usize;
                if seen[n] {
                    return Err(FormError::invalid(format!(
                        "object {n} is linked more than once"
                    )));
                }
                seen[n] = true;
                if self.objects[n].parent != i as u32 {
                    return Err(FormError::invalid(format!(
                        "object {n} is a child of {i} but names parent {}",
                        self.objects[n].parent
                    )));
                }
                next = self.objects[n].next_sibling;
            }
        }
        for (i, o) in self.objects.iter().enumerate() {
            if o.parent == NO_OBJECT {
                if seen[i] {
                    return Err(FormError::invalid(format!(
                        "root object {i} is also a child"
                    )));
                }
            } else if !seen[i] {
                return Err(FormError::invalid(format!(
                    "object {i} is not in its parent's child list"
                )));
            }
        }
        // Every object must reach a root: walk up with a bound to catch parent cycles.
        for i in 0..count {
            let mut at = i;
            let mut steps = 0;
            while self.objects[at].parent != NO_OBJECT {
                at = self.objects[at].parent as usize;
                steps += 1;
                if steps > count {
                    return Err(FormError::invalid(format!(
                        "object {i} is in a parent cycle"
                    )));
                }
            }
        }
        for f in &self.forms {
            if (f.root as usize) >= count {
                return Err(FormError::invalid(format!(
                    "form \"{}\" has missing root {}",
                    f.name, f.root
                )));
            }
        }
        Ok(())
    }

    /// Writes the file as bytes in the current format version.
    ///
    /// # Returns
    ///
    /// The bytes.
    ///
    /// # Errors
    ///
    /// [`FormError::Invalid`] when the file is inconsistent ([`FormFile::validate`]), and
    /// [`FormError::TooLarge`] when it exceeds the format's limits.
    pub fn to_bytes(&self) -> Result<Vec<u8>, FormError> {
        self.validate()?;
        let mut pool = StringPool::default();

        // Property blocks first, so the object table can hold their offsets.
        let mut props = Vec::new();
        let mut offsets = Vec::with_capacity(self.objects.len());
        for o in &self.objects {
            if o.properties.is_empty() {
                offsets.push(NO_PROPERTIES);
                continue;
            }
            offsets.push(to_u32(props.len(), "property section")?);
            let mut block = Vec::new();
            for p in &o.properties {
                write_property(&mut block, p, &mut pool)?;
            }
            put_u32(&mut props, to_u32(block.len(), "property block")?);
            props.extend_from_slice(&block);
        }

        let mut forms = Vec::new();
        put_u32(&mut forms, to_u32(self.forms.len(), "form table")?);
        for f in &self.forms {
            put_u32(&mut forms, pool.add(&f.name)?);
            put_u32(&mut forms, f.root);
        }

        let mut objects = Vec::new();
        put_u32(&mut objects, to_u32(self.objects.len(), "object table")?);
        for (o, offset) in self.objects.iter().zip(&offsets) {
            put_u32(&mut objects, pool.add(&o.type_name)?);
            put_u32(&mut objects, o.parent);
            put_u32(&mut objects, o.first_child);
            put_u32(&mut objects, o.next_sibling);
            put_i32(&mut objects, o.x);
            put_i32(&mut objects, o.y);
            put_u32(&mut objects, o.width);
            put_u32(&mut objects, o.height);
            put_u32(&mut objects, o.behavior);
            put_u32(&mut objects, o.state);
            put_u32(&mut objects, o.anchors);
            put_u32(&mut objects, *offset);
        }

        let mut images = Vec::new();
        put_u32(&mut images, to_u32(self.images.len(), "image pool")?);
        for img in &self.images {
            put_u32(&mut images, pool.add(&img.name)?);
            put_u32(&mut images, to_u32(img.data.len(), "image")?);
            images.extend_from_slice(&img.data);
        }

        let mut symbols = Vec::new();
        put_u32(&mut symbols, to_u32(self.symbols.len(), "symbol table")?);
        for s in &self.symbols {
            symbols.push(match s.kind {
                SymbolKind::Id => 0,
                SymbolKind::Action => 1,
            });
            put_u32(&mut symbols, s.id);
            put_u32(&mut symbols, pool.add(&s.name)?);
        }

        let strings = pool.into_bytes()?;
        let sections = [&forms, &objects, &props, &strings, &images, &symbols];
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&(HEADER_LEN as u16).to_le_bytes());
        let mut offset = HEADER_LEN;
        for s in sections {
            put_u32(&mut out, to_u32(offset, "file")?);
            put_u32(&mut out, to_u32(s.len(), "section")?);
            offset += s.len();
        }
        to_u32(offset, "file")?;
        for s in sections {
            out.extend_from_slice(s);
        }
        Ok(out)
    }

    /// Reads a form file.
    ///
    /// Property tags and value kinds this version does not define are skipped, so a file written
    /// by a newer library with extra properties still loads.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The file's bytes.
    ///
    /// # Returns
    ///
    /// The file.
    ///
    /// # Errors
    ///
    /// A [`FormError`] describing why the bytes are not a valid form file: a wrong magic, a newer
    /// version, a truncated section, or an inconsistent table.
    pub fn from_bytes(bytes: &[u8]) -> Result<FormFile, FormError> {
        if bytes.len() < 4 || bytes[0..4] != MAGIC {
            return Err(FormError::BadMagic);
        }
        let mut head = Reader::new(bytes, "header");
        head.skip(4)?;
        let version = head.u16()?;
        if version > FORMAT_VERSION {
            return Err(FormError::UnsupportedVersion { version });
        }
        let header_len = head.u16()? as usize;
        if header_len < HEADER_LEN {
            return Err(FormError::invalid("header is too short"));
        }
        let mut section = |what: &'static str| -> Result<&[u8], FormError> {
            let offset = head.u32()? as usize;
            let length = head.u32()? as usize;
            let end = offset
                .checked_add(length)
                .ok_or(FormError::Truncated { what })?;
            bytes.get(offset..end).ok_or(FormError::Truncated { what })
        };
        let forms = section("form table")?;
        let objects = section("object table")?;
        let props = section("property section")?;
        let strings = section("string pool")?;
        let images = section("image pool")?;
        let symbols = section("symbol table")?;

        let pool = read_strings(strings)?;
        let mut file = FormFile::new();

        let mut r = Reader::new(forms, "form table");
        for _ in 0..r.count(8)? {
            let name = pool.get(r.u32()?)?;
            file.forms.push(FormEntry {
                name,
                root: r.u32()?,
            });
        }

        let mut r = Reader::new(objects, "object table");
        for _ in 0..r.count(OBJECT_LEN)? {
            let type_name = pool.get(r.u32()?)?;
            let mut o = ObjectRecord::new(&type_name);
            o.parent = r.u32()?;
            o.first_child = r.u32()?;
            o.next_sibling = r.u32()?;
            o.x = r.i32()?;
            o.y = r.i32()?;
            o.width = r.u32()?;
            o.height = r.u32()?;
            o.behavior = r.u32()?;
            o.state = r.u32()?;
            o.anchors = r.u32()?;
            let offset = r.u32()?;
            if offset != NO_PROPERTIES {
                o.properties = read_property_block(props, offset as usize, &pool)?;
            }
            file.objects.push(o);
        }

        let mut r = Reader::new(images, "image pool");
        for _ in 0..r.count(8)? {
            let name = pool.get(r.u32()?)?;
            let len = r.u32()? as usize;
            file.images.push(ImageEntry {
                name,
                data: r.take(len)?.to_vec(),
            });
        }

        let mut r = Reader::new(symbols, "symbol table");
        for _ in 0..r.count(9)? {
            let kind = match r.u8()? {
                0 => SymbolKind::Id,
                1 => SymbolKind::Action,
                other => return Err(FormError::invalid(format!("unknown symbol kind {other}"))),
            };
            let id = r.u32()?;
            let name = pool.get(r.u32()?)?;
            file.symbols.push(Symbol { kind, id, name });
        }

        file.validate()?;
        Ok(file)
    }
}

/// Converts a length to `u32`, or reports the format limit.
fn to_u32(n: usize, what: &'static str) -> Result<u32, FormError> {
    u32::try_from(n).map_err(|_| FormError::TooLarge { what })
}

/// Appends a little-endian `u32`.
fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Appends a little-endian `i32`.
fn put_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// The string pool the writer builds: each distinct string is stored once.
#[derive(Default)]
struct StringPool {
    strings: Vec<String>,
    index: HashMap<String, u32>,
}

impl StringPool {
    /// Adds a string (or finds it) and returns its number.
    fn add(&mut self, s: &str) -> Result<u32, FormError> {
        if let Some(n) = self.index.get(s) {
            return Ok(*n);
        }
        let n = to_u32(self.strings.len(), "string pool")?;
        self.strings.push(s.to_string());
        self.index.insert(s.to_string(), n);
        Ok(n)
    }

    /// Encodes the pool: a count, then each string as a length and UTF-8 bytes.
    fn into_bytes(self) -> Result<Vec<u8>, FormError> {
        let mut out = Vec::new();
        put_u32(&mut out, to_u32(self.strings.len(), "string pool")?);
        for s in &self.strings {
            put_u32(&mut out, to_u32(s.len(), "string")?);
            out.extend_from_slice(s.as_bytes());
        }
        Ok(out)
    }
}

/// The strings a reader decoded.
struct ReadPool(Vec<String>);

impl ReadPool {
    /// Returns string `n`, or an error when the number is out of range.
    fn get(&self, n: u32) -> Result<String, FormError> {
        self.0
            .get(n as usize)
            .cloned()
            .ok_or_else(|| FormError::invalid(format!("string {n} does not exist")))
    }
}

/// Decodes the string pool section.
fn read_strings(bytes: &[u8]) -> Result<ReadPool, FormError> {
    let mut r = Reader::new(bytes, "string pool");
    let count = r.count(4)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let len = r.u32()? as usize;
        let text = std::str::from_utf8(r.take(len)?)
            .map_err(|_| FormError::invalid("a string is not valid UTF-8"))?;
        out.push(text.to_string());
    }
    Ok(ReadPool(out))
}

/// Appends one property as a tag-length-value entry.
fn write_property(out: &mut Vec<u8>, p: &Property, pool: &mut StringPool) -> Result<(), FormError> {
    let mut body = Vec::new();
    let tag = match p {
        Property::Named { name, value } => {
            put_u32(&mut body, pool.add(name)?);
            match value {
                FormValue::Bool(b) => {
                    body.push(kind::BOOL);
                    body.push(u8::from(*b));
                }
                FormValue::Int(i) => {
                    body.push(kind::INT);
                    body.extend_from_slice(&i.to_le_bytes());
                }
                FormValue::Float(f) => {
                    body.push(kind::FLOAT);
                    body.extend_from_slice(&f.to_le_bytes());
                }
                FormValue::Text(t) => {
                    body.push(kind::TEXT);
                    put_u32(&mut body, pool.add(t)?);
                }
                FormValue::Color(c) => {
                    body.push(kind::COLOR);
                    body.extend_from_slice(&[c.r, c.g, c.b, c.a]);
                }
                FormValue::Image(n) => {
                    body.push(kind::IMAGE);
                    put_u32(&mut body, *n);
                }
            }
            tag::NAMED
        }
        Property::Layout(l) => {
            body.push(l.kind);
            put_u32(&mut body, to_u32(l.params.len(), "layout parameters")?);
            for v in &l.params {
                put_i32(&mut body, *v);
            }
            tag::LAYOUT
        }
        Property::ChildLayout(c) => {
            body.extend_from_slice(&c.stretch.to_le_bytes());
            body.push(c.horizontal);
            body.push(c.vertical);
            for m in c.margins {
                put_u32(&mut body, m);
            }
            for v in c.cell {
                body.extend_from_slice(&v.to_le_bytes());
            }
            tag::CHILD_LAYOUT
        }
    };
    out.extend_from_slice(&tag.to_le_bytes());
    put_u32(out, to_u32(body.len(), "property")?);
    out.extend_from_slice(&body);
    Ok(())
}

/// Reads the property block at `offset` in the property section.
fn read_property_block(
    section: &[u8],
    offset: usize,
    pool: &ReadPool,
) -> Result<Vec<Property>, FormError> {
    let tail = section.get(offset..).ok_or(FormError::Truncated {
        what: "property block",
    })?;
    let mut r = Reader::new(tail, "property block");
    let len = r.u32()? as usize;
    let mut r = Reader::new(r.take(len)?, "property block");
    let mut out = Vec::new();
    while !r.is_empty() {
        let tag = r.u16()?;
        let len = r.u32()? as usize;
        let mut body = Reader::new(r.take(len)?, "property");
        let property = match tag {
            tag::NAMED => read_named(&mut body, pool)?,
            tag::LAYOUT => {
                let kind = body.u8()?;
                let count = body.count(4)?;
                let mut params = Vec::with_capacity(count);
                for _ in 0..count {
                    params.push(body.i32()?);
                }
                Some(Property::Layout(LayoutSpec { kind, params }))
            }
            tag::CHILD_LAYOUT => {
                let stretch = body.u16()?;
                let horizontal = body.u8()?;
                let vertical = body.u8()?;
                let margins = [body.u32()?, body.u32()?, body.u32()?, body.u32()?];
                let cell = [body.u16()?, body.u16()?, body.u16()?, body.u16()?];
                Some(Property::ChildLayout(ChildLayout {
                    stretch,
                    horizontal,
                    vertical,
                    margins,
                    cell,
                }))
            }
            _ => None, // An unknown tag: its length was consumed, so it is skipped.
        };
        out.extend(property);
    }
    Ok(out)
}

/// Reads a named property; `None` when its value kind is not defined in this version.
fn read_named(body: &mut Reader<'_>, pool: &ReadPool) -> Result<Option<Property>, FormError> {
    let name = pool.get(body.u32()?)?;
    let value = match body.u8()? {
        kind::BOOL => FormValue::Bool(body.u8()? != 0),
        kind::INT => FormValue::Int(i64::from_le_bytes(body.array()?)),
        kind::FLOAT => FormValue::Float(f64::from_le_bytes(body.array()?)),
        kind::TEXT => FormValue::Text(pool.get(body.u32()?)?),
        kind::COLOR => {
            let [r, g, b, a] = body.array()?;
            FormValue::Color(Color { r, g, b, a })
        }
        kind::IMAGE => FormValue::Image(body.u32()?),
        _ => return Ok(None),
    };
    Ok(Some(Property::Named { name, value }))
}

/// A bounds-checked cursor over bytes.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    what: &'static str,
}

impl<'a> Reader<'a> {
    /// Creates a reader; `what` names the data in errors.
    fn new(bytes: &'a [u8], what: &'static str) -> Self {
        Reader { bytes, at: 0, what }
    }

    /// Whether every byte has been read.
    fn is_empty(&self) -> bool {
        self.at >= self.bytes.len()
    }

    /// Takes the next `n` bytes.
    fn take(&mut self, n: usize) -> Result<&'a [u8], FormError> {
        let end = self
            .at
            .checked_add(n)
            .ok_or(FormError::Truncated { what: self.what })?;
        let out = self
            .bytes
            .get(self.at..end)
            .ok_or(FormError::Truncated { what: self.what })?;
        self.at = end;
        Ok(out)
    }

    /// Skips `n` bytes.
    fn skip(&mut self, n: usize) -> Result<(), FormError> {
        self.take(n).map(|_| ())
    }

    /// Takes the next `N` bytes as an array.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], FormError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N)?);
        Ok(out)
    }

    /// Reads one byte.
    fn u8(&mut self) -> Result<u8, FormError> {
        Ok(self.array::<1>()?[0])
    }

    /// Reads a little-endian `u16`.
    fn u16(&mut self) -> Result<u16, FormError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    /// Reads a little-endian `u32`.
    fn u32(&mut self) -> Result<u32, FormError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    /// Reads a little-endian `i32`.
    fn i32(&mut self) -> Result<i32, FormError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    /// Reads an entry count and checks the rest of the data could hold that many entries of at
    /// least `min_size` bytes, so a lying count cannot force a huge allocation.
    fn count(&mut self, min_size: usize) -> Result<usize, FormError> {
        let n = self.u32()? as usize;
        let remaining = self.bytes.len() - self.at;
        if n.checked_mul(min_size).is_none_or(|need| need > remaining) {
            return Err(FormError::Truncated { what: self.what });
        }
        Ok(n)
    }
}
