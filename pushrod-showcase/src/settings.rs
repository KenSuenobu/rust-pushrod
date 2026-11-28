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

//! Remembering the last page between runs: its name, in a small text file.

use std::path::{Path, PathBuf};

/// Where the last page's name is stored.
///
/// # Returns
///
/// `$XDG_CONFIG_HOME/pushrod-showcase/last_page` or `$HOME/.config/pushrod-showcase/last_page`,
/// or `None` when neither variable is set.
pub fn default_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;

    Some(base.join("pushrod-showcase").join("last_page"))
}

/// Reads the remembered page name.
///
/// # Arguments
///
/// * `path` - The settings file.
///
/// # Returns
///
/// The name, or `None` when the file is missing, unreadable, or empty.
pub fn load_last_page(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let name = text.trim();

    (!name.is_empty()).then(|| name.to_string())
}

/// Remembers a page name.  Failures are ignored: forgetting the page is not worth an error.
///
/// # Arguments
///
/// * `path` - The settings file; its folder is created when missing.
/// * `name` - The page's name.
pub fn save_last_page(path: &Path, name: &str) {
    if let Some(folder) = path.parent() {
        let _ = std::fs::create_dir_all(folder);
    }

    let _ = std::fs::write(path, name);
}
