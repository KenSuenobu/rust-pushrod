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

//! # Finding handlers in the code on disk
//!
//! Handler files are the developer's.  Chassis reads them with tree-sitter and reports the
//! functions it finds, so the model always reflects the code as it is, never a cached copy.

use std::path::{Path, PathBuf};

use tree_sitter::Parser;

use super::error::ProjectError;

/// One function found at the top level of a handler file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handler {
    /// The form whose handler file holds the function, or `None` if no form has that file.
    pub form: Option<String>,
    /// The function's name.
    pub name: String,
    /// The signature as written, from `pub` or `fn` to just before the body.
    pub signature: String,
    /// The handler file, relative to the project.
    pub file: PathBuf,
    /// The line the function starts on, counting from 1.
    pub line: usize,
}

/// Lists the top-level functions of Rust source.
///
/// # Arguments
///
/// * `source` - The Rust source text.
/// * `file` - The file it came from, relative to the project, for errors and results.
///
/// # Returns
///
/// `(name, signature, line)` for each top-level function, in source order.
///
/// # Errors
///
/// [`ProjectError::Handlers`] if the parser cannot be set up or the source does not parse
/// cleanly.
pub fn parse_functions(
    source: &str,
    file: &Path,
) -> Result<Vec<(String, String, usize)>, ProjectError> {
    let fail = |message: String| ProjectError::Handlers {
        path: file.to_path_buf(),
        message,
    };
    let mut parser = Parser::new();

    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| fail(e.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| fail("the parser gave no result".to_string()))?;
    let root = tree.root_node();

    if root.has_error() {
        return Err(fail("the file has syntax errors".to_string()));
    }

    let bytes = source.as_bytes();
    let mut cursor = root.walk();
    let mut found = Vec::new();

    for node in root.children(&mut cursor) {
        if node.kind() != "function_item" {
            continue;
        }

        let Some(name) = node.child_by_field_name("name") else {
            continue;
        };
        let end = node
            .child_by_field_name("body")
            .map_or(node.end_byte(), |b| b.start_byte());
        let signature = String::from_utf8_lossy(&bytes[node.start_byte()..end])
            .trim()
            .to_string();

        found.push((
            String::from_utf8_lossy(&bytes[name.byte_range()]).into_owned(),
            signature,
            node.start_position().row + 1,
        ));
    }

    Ok(found)
}

/// Finds the name of one top-level function, so a rename can replace only those bytes.
///
/// # Arguments
///
/// * `source` - The Rust source text.
/// * `file` - The file it came from, relative to the project, for errors.
/// * `name` - The function to find.
///
/// # Returns
///
/// The byte range of the function's name in `source`, or `None` if there is no top-level function
/// of that name.
///
/// # Errors
///
/// [`ProjectError::Handlers`] if the parser cannot be set up or the source does not parse
/// cleanly.
pub fn function_name_range(
    source: &str,
    file: &Path,
    name: &str,
) -> Result<Option<std::ops::Range<usize>>, ProjectError> {
    let fail = |message: String| ProjectError::Handlers {
        path: file.to_path_buf(),
        message,
    };
    let mut parser = Parser::new();

    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| fail(e.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| fail("the parser gave no result".to_string()))?;
    let root = tree.root_node();

    if root.has_error() {
        return Err(fail("the file has syntax errors".to_string()));
    }

    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {
        if node.kind() != "function_item" {
            continue;
        }

        if let Some(n) = node.child_by_field_name("name")
            && &source[n.byte_range()] == name
        {
            return Ok(Some(n.byte_range()));
        }
    }

    Ok(None)
}
