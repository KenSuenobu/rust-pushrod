// Errors
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

//! # Errors
//!
//! Library code reports failures as [`Error`] values instead of panicking, so an application can
//! recover or report the problem.  Fallible public functions return [`Result`].  A failure that
//! happens inside the run loop (a font that will not load while text is drawn, a texture the
//! driver cannot create) has no caller to return to, so the engine delivers it as
//! [`EngineEvent::Error`](crate::event::EngineEvent::Error) and keeps running.
//!
//! ```
//! use pushrod::error::Error;
//!
//! let error = Error::font("missing.ttf", "No such file");
//!
//! assert_eq!(error.to_string(), "font \"missing.ttf\": No such file");
//! ```

use crate::widget::WidgetId;
use std::fmt;

/// A failure reported by Pushrod or a backend.
///
/// Every variant carries text, not a library-specific error type, so the value is `Clone` and
/// `Eq` and can travel in an [`EngineEvent`](crate::event::EngineEvent).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    /// A backend could not start or lost a resource it needs (window, renderer, text library).
    Backend {
        /// What went wrong, as the backend reported it.
        message: String,
    },
    /// A font file could not be loaded or used.
    Font {
        /// The path of the font file.
        path: String,
        /// What went wrong.
        message: String,
    },
    /// An image could not be read, decoded or written.
    Image {
        /// The path of the image file.
        path: String,
        /// What went wrong.
        message: String,
    },
    /// A texture could not be created, updated or drawn.
    Texture {
        /// What went wrong.
        message: String,
    },
    /// A `Widget` ID does not exist in the widget tree.
    UnknownWidget {
        /// The ID that was not found.
        id: WidgetId,
    },
    /// A form file could not be read or is not valid.
    FormFile {
        /// The path of the form file.
        path: String,
        /// What went wrong.
        message: String,
    },
}

/// The result of a fallible Pushrod function.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Creates a [`Error::Backend`].
    ///
    /// # Arguments
    ///
    /// * `message` - What went wrong.
    ///
    /// # Returns
    ///
    /// The error.
    pub fn backend(message: impl Into<String>) -> Self {
        Error::Backend {
            message: message.into(),
        }
    }

    /// Creates a [`Error::Font`].
    ///
    /// # Arguments
    ///
    /// * `path` - The font file's path.
    /// * `message` - What went wrong.
    ///
    /// # Returns
    ///
    /// The error.
    pub fn font(path: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Font {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a [`Error::Image`].
    ///
    /// # Arguments
    ///
    /// * `path` - The image file's path.
    /// * `message` - What went wrong.
    ///
    /// # Returns
    ///
    /// The error.
    pub fn image(path: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Image {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a [`Error::Texture`].
    ///
    /// # Arguments
    ///
    /// * `message` - What went wrong.
    ///
    /// # Returns
    ///
    /// The error.
    pub fn texture(message: impl Into<String>) -> Self {
        Error::Texture {
            message: message.into(),
        }
    }

    /// Creates a [`Error::FormFile`].
    ///
    /// # Arguments
    ///
    /// * `path` - The form file's path.
    /// * `message` - What went wrong.
    ///
    /// # Returns
    ///
    /// The error.
    pub fn form_file(path: impl Into<String>, message: impl Into<String>) -> Self {
        Error::FormFile {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    /// Writes a one-line description, without a trailing period.
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
            Error::Backend { message } => write!(f, "backend: {message}"),
            Error::Font { path, message } => write!(f, "font \"{path}\": {message}"),
            Error::Image { path, message } => write!(f, "image \"{path}\": {message}"),
            Error::Texture { message } => write!(f, "texture: {message}"),
            Error::UnknownWidget { id } => write!(f, "widget {id} does not exist"),
            Error::FormFile { path, message } => write!(f, "form file \"{path}\": {message}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_the_failing_thing() {
        assert_eq!(
            Error::backend("no driver").to_string(),
            "backend: no driver"
        );
        assert_eq!(
            Error::font("a.ttf", "missing").to_string(),
            "font \"a.ttf\": missing"
        );
        assert_eq!(
            Error::image("a.png", "bad").to_string(),
            "image \"a.png\": bad"
        );
        assert_eq!(Error::texture("full").to_string(), "texture: full");
        assert_eq!(
            Error::UnknownWidget { id: 7 }.to_string(),
            "widget 7 does not exist"
        );
        assert_eq!(
            Error::form_file("f.form", "syntax").to_string(),
            "form file \"f.form\": syntax"
        );
    }

    #[test]
    fn is_a_std_error() {
        let boxed: Box<dyn std::error::Error> = Box::new(Error::texture("x"));

        assert_eq!(boxed.to_string(), "texture: x");
    }
}
