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

//! # Project errors

use std::fmt;
use std::path::PathBuf;

/// What can go wrong loading a project or writing one of its files.
#[derive(Debug)]
pub enum ProjectError {
    /// A file or directory could not be read or written.
    Io {
        /// The path involved.
        path: PathBuf,
        /// What the operating system said.
        message: String,
    },
    /// `Cargo.toml` is missing a required part or is not valid TOML.
    Manifest {
        /// What is wrong.
        message: String,
    },
    /// The resource file could not be read, or is not a valid resource file.
    Resources {
        /// The resource file.
        path: PathBuf,
        /// What is wrong.
        message: String,
    },
    /// A handler file is not valid Rust syntax that can be read.
    Handlers {
        /// The handler file.
        path: PathBuf,
        /// What is wrong.
        message: String,
    },
    /// Chassis refused to write a file it does not own, or to overwrite one it must not.
    Ownership {
        /// The path, relative to the project.
        path: PathBuf,
        /// Why the write is refused.
        message: String,
    },
}

impl fmt::Display for ProjectError {
    /// Writes the error as one line naming the file and the problem.
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
            ProjectError::Io { path, message } => write!(f, "{}: {message}", path.display()),
            ProjectError::Manifest { message } => write!(f, "Cargo.toml: {message}"),
            ProjectError::Resources { path, message } => {
                write!(f, "resource file {}: {message}", path.display())
            }
            ProjectError::Handlers { path, message } => {
                write!(f, "handler file {}: {message}", path.display())
            }
            ProjectError::Ownership { path, message } => {
                write!(f, "refused to write {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ProjectError {}
