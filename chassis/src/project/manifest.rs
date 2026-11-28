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

//! # The crate's `Cargo.toml`
//!
//! Chassis reads the manifest with `toml_edit` and never writes it.

use std::path::Path;

use toml_edit::DocumentMut;

use super::error::ProjectError;

/// The parts of `Cargo.toml` the project model uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// The crate's name.
    pub name: String,
    /// The crate's version.
    pub version: String,
    /// The resource file named by `[package.metadata.chassis] resources`, relative to the project.
    pub resources: Option<String>,
}

impl Package {
    /// Reads the manifest at `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The `Cargo.toml` file.
    ///
    /// # Returns
    ///
    /// The package.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Io`] if the file cannot be read, and [`ProjectError::Manifest`] if it is
    /// not TOML or has no `[package]` with a `name`.
    pub fn read(path: &Path) -> Result<Self, ProjectError> {
        let text = std::fs::read_to_string(path).map_err(|e| ProjectError::Io {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;

        Self::parse(&text)
    }

    /// Parses manifest text.
    ///
    /// # Arguments
    ///
    /// * `text` - The contents of a `Cargo.toml`.
    ///
    /// # Returns
    ///
    /// The package.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Manifest`] if the text is not TOML or has no `[package]` with a `name`.
    pub fn parse(text: &str) -> Result<Self, ProjectError> {
        let doc: DocumentMut =
            text.parse()
                .map_err(|e: toml_edit::TomlError| ProjectError::Manifest {
                    message: e.to_string(),
                })?;
        let package = doc
            .get("package")
            .and_then(|p| p.as_table_like())
            .ok_or_else(|| ProjectError::Manifest {
                message: "no [package] table".to_string(),
            })?;
        let name = package
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| ProjectError::Manifest {
                message: "[package] has no name".to_string(),
            })?;
        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");
        let resources = package
            .get("metadata")
            .and_then(|m| m.as_table_like())
            .and_then(|m| m.get("chassis"))
            .and_then(|c| c.as_table_like())
            .and_then(|c| c.get("resources"))
            .and_then(|r| r.as_str())
            .map(str::to_string);

        Ok(Package {
            name: name.to_string(),
            version: version.to_string(),
            resources,
        })
    }
}
