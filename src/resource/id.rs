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

//! # Typed resource IDs
//!
//! A resource is named by a number, and a bare number says nothing about what it names.  Each
//! kind of resource has its own ID type, so passing a [`StringId`] where an [`ImageId`] is wanted
//! is a compile error rather than a wrong picture at run time:
//!
//! ```
//! use pushrod::resource::{ImageId, StringId};
//!
//! const SAVE_FAILED: StringId = StringId(3);
//! const LOGO: ImageId = ImageId(1);
//!
//! fn show_image(image: ImageId) -> u32 {
//!     image.get()
//! }
//!
//! assert_eq!(show_image(LOGO), 1);
//! assert_eq!(u32::from(SAVE_FAILED), 3);
//! ```
//!
//! Using one kind as another does not compile:
//!
//! ```compile_fail
//! use pushrod::resource::{ImageId, StringId};
//!
//! fn show_image(image: ImageId) {}
//!
//! show_image(StringId(3));
//! ```
//!
//! The constants are written by [`crate::form::codegen`] from a form file's symbol table, so code
//! says `strings::SAVE_FAILED` and not `3`.  Numbers are the stable IDs of the resource tables
//! (they start at 1 and are never reused); a [`FormId`] is the form's position in the file,
//! counting from 1.  Creating an ID is not an engine action, so these types emit no engine event.

pub use crate::form::FormIndex;

/// Gives an ID newtype its accessor and its conversion to a number.
///
/// # Arguments
///
/// * `$name` - The ID type.
macro_rules! id_impls {
    ($($name:ident),+ $(,)?) => {$(
        impl $name {
            /// Returns the ID's number.
            ///
            /// # Returns
            ///
            /// The number, as stored in the resource file.
            pub fn get(self) -> u32 {
                self.0
            }
        }

        impl From<$name> for u32 {
            /// Converts an ID to its number.
            ///
            /// # Arguments
            ///
            /// * `id` - The ID.
            ///
            /// # Returns
            ///
            /// The number.
            fn from(id: $name) -> u32 {
                id.0
            }
        }
    )+};
}

/// The number of a form: its position in the form file, counting from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FormId(pub u32);

/// The ID of a string resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringId(pub u32);

/// The ID of an image resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(pub u32);

/// The ID of an icon resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IconId(pub u32);

/// The ID of a file resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

/// The ID of a menu resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MenuId(pub u32);

/// The ID of an alert resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AlertId(pub u32);

id_impls!(FormId, StringId, ImageId, IconId, FileId, MenuId, AlertId);
