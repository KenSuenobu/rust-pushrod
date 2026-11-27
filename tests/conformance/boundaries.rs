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

//! Conformance for `docs/specs/modules.md`: the engine, the design layer, and the crates stay
//! separate.  These tests read the repository's own source, so a change that crosses a boundary
//! fails CI.

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root: the manifest directory of the `pushrod` crate.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Returns the code lines of every `.rs` file under `path` (a file or a directory), without
/// comment lines, as `(file, line)` pairs.
///
/// # Arguments
///
/// * `path` - A source file or a directory of source files.
///
/// # Returns
///
/// Every line that is not blank and not a `//` comment, with the file it came from.
fn code_lines(path: &Path) -> Vec<(PathBuf, String)> {
    let mut lines = Vec::new();

    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();

        entries.sort();
        for entry in entries {
            lines.extend(code_lines(&entry));
        }
    } else if path.extension().is_some_and(|ext| ext == "rs") {
        for line in fs::read_to_string(path).unwrap().lines() {
            let trimmed = line.trim();

            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                lines.push((path.to_path_buf(), trimmed.to_string()));
            }
        }
    }

    lines
}

/// Returns the names under a manifest's `[dependencies]` table.
///
/// # Arguments
///
/// * `manifest` - The path of a `Cargo.toml`.
///
/// # Returns
///
/// The dependency names, in file order.
fn dependencies(manifest: &Path) -> Vec<String> {
    let text = fs::read_to_string(manifest).unwrap();
    let mut in_table = false;
    let mut names = Vec::new();

    for line in text.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_table = line == "[dependencies]";
        } else if in_table
            && !line.starts_with('#')
            && let Some((name, _)) = line.split_once('=')
        {
            names.push(name.trim().to_string());
        }
    }

    names
}

/// Modules spec, "Engine": the core crate depends on no windowing library and no widget library.
#[test]
fn the_core_crate_depends_on_no_backend_or_widget_library() {
    let names = dependencies(&root().join("Cargo.toml"));

    assert!(!names.is_empty(), "the manifest was not read");
    for forbidden in ["sdl2", "pushrod-sdl2", "pushrod-widgets"] {
        assert!(!names.iter().any(|name| name == forbidden), "{forbidden}");
    }
}

/// Modules spec, "Engine": the core source never names a backend or widget crate in code.
#[test]
fn the_core_source_never_uses_a_backend_or_widget_crate() {
    let lines = code_lines(&root().join("src"));

    assert!(!lines.is_empty());
    for (file, line) in lines {
        for forbidden in ["sdl2", "pushrod_widgets", "pushrod_sdl2"] {
            assert!(
                !line.contains(forbidden),
                "{} mentions {forbidden}: {line}",
                file.display()
            );
        }
    }
}

/// Modules spec, "Design layer": the engine modules never use the appearance module, so styling
/// can change without touching the tree, events, invalidation, or backend contract.
#[test]
fn the_engine_modules_do_not_use_the_design_layer() {
    let engine = [
        "engine.rs",
        "cache.rs",
        "cache",
        "event.rs",
        "input.rs",
        "timer.rs",
        "backend",
    ];

    for name in engine {
        for (file, line) in code_lines(&root().join("src").join(name)) {
            assert!(
                !line.contains("appearance"),
                "{} uses the design layer: {line}",
                file.display()
            );
        }
    }
}

/// Modules spec, "Design layer": the standard widgets depend on the core crate only, so they use
/// the same public API an alternative widget library has.
#[test]
fn the_widget_crate_depends_on_the_core_only() {
    let names = dependencies(&root().join("pushrod-widgets").join("Cargo.toml"));

    assert_eq!(names, vec!["pushrod".to_string()]);

    for (file, line) in code_lines(&root().join("pushrod-widgets").join("src")) {
        assert!(
            !line.contains("pushrod_sdl2") && !line.contains("sdl2::"),
            "{} reaches a backend: {line}",
            file.display()
        );
    }
}
