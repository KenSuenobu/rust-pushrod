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
//! A form file stores one or more forms (widget trees) in a compact, versioned, little-endian
//! binary format, so a program refers to every object by its number, as classic resource files
//! did.  The format is specified in `docs/FORM_FORMAT.md`; the [`mod@format`] module reads and writes
//! it using only the standard library.
//!
//! ```
//! use pushrod::form::{FormFile, ObjectRecord};
//!
//! let mut file = FormFile::new();
//! let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
//! file.add_form("Settings", root);
//!
//! let bytes = file.to_bytes().unwrap();
//! assert_eq!(FormFile::from_bytes(&bytes).unwrap(), file);
//! ```

pub mod format;

pub use format::{
    ChildLayout, FORMAT_VERSION, FormEntry, FormError, FormFile, FormValue, ImageEntry, LayoutSpec,
    MAGIC, NO_OBJECT, ObjectRecord, Property, Symbol, SymbolKind,
};
