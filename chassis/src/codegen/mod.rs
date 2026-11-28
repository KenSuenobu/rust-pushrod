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

//! # Code generation rules
//!
//! What Chassis writes, when, and what it never touches, so developers can trust that Chassis
//! never destroys their code:
//!
//! 1. Chassis rewrites only `src/generated/`, and runs `rustfmt` over what it writes
//!    ([`write_generated`]).
//! 2. It creates `src/main.rs` once ([`create_main`]) and `src/forms/<form>.rs` once
//!    ([`Project::create_stub`]); a file that exists is never overwritten.
//! 3. Afterwards it only appends handler stubs ([`insert_handler_stub`]) and renames a handler's
//!    name in its signature ([`rename_handler`]).
//! 4. Handlers are connected to the engine by generated wiring ([`wiring`]), never by code the
//!    developer writes.
//! 5. It never edits a function body.  Both edits change the file by splicing text at one place;
//!    every other byte is kept as it was, and the result is parsed again before it is written.

pub mod wiring;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::project::{HANDLERS_DIR, Project, ProjectError, handlers};

/// Words that cannot name a function (strict and reserved keywords).
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];

/// The project-relative path of the application's entry point.
pub const MAIN_FILE: &str = "src/main.rs";

/// Formats Rust source with `rustfmt`.  Line endings are always `\n`, so generated files are
/// the same on every platform.
///
/// # Arguments
///
/// * `source` - The Rust source text.
///
/// # Returns
///
/// The formatted source.
///
/// # Errors
///
/// [`ProjectError::Io`] if `rustfmt` cannot be run, and [`ProjectError::Handlers`] if it rejects
/// the source.
pub fn format_source(source: &str) -> Result<String, ProjectError> {
    let io = |message: String| ProjectError::Io {
        path: PathBuf::from("rustfmt"),
        message,
    };
    let mut child = Command::new("rustfmt")
        .args([
            "--edition",
            "2024",
            "--emit",
            "stdout",
            "--config",
            "newline_style=Unix",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| io(e.to_string()))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io("no standard input".to_string()))?;
    let input = source.to_string();
    // Feed rustfmt from a thread so a large source cannot fill the pipes and stall both sides.
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let output = child.wait_with_output().map_err(|e| io(e.to_string()))?;

    writer
        .join()
        .map_err(|_| io("the writer thread panicked".to_string()))?
        .map_err(|e| io(e.to_string()))?;

    if !output.status.success() {
        return Err(ProjectError::Handlers {
            path: PathBuf::from("rustfmt"),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    String::from_utf8(output.stdout).map_err(|e| io(e.to_string()))
}

/// Writes a generated file under `src/generated`, formatted with `rustfmt`.
///
/// # Arguments
///
/// * `project` - The project.
/// * `rel` - The path relative to the project; it must be inside `src/generated`.
/// * `contents` - The Rust source to write.
///
/// # Returns
///
/// `Ok(())` once the file is written.
///
/// # Errors
///
/// [`ProjectError::Ownership`] if `rel` is outside `src/generated` (checked before `rustfmt`
/// runs), or the errors of [`format_source`] and [`Project::write_generated`].
pub fn write_generated(
    project: &Project,
    rel: impl AsRef<Path>,
    contents: &str,
) -> Result<(), ProjectError> {
    let rel = rel.as_ref();

    if project.ownership(rel) != crate::Ownership::Generated {
        return Err(Project::refuse(
            rel,
            "only files under src/generated belong to Chassis",
        ));
    }

    project.write_generated(rel, &format_source(contents)?)
}

/// Creates `src/main.rs`, formatted, only if it does not exist.
///
/// # Arguments
///
/// * `project` - The project.
/// * `contents` - The Rust source to write.
///
/// # Returns
///
/// The path of the new file, relative to the project.
///
/// # Errors
///
/// [`ProjectError::Ownership`] if the file already exists (it is the developer's from then on),
/// or the errors of [`format_source`] and [`ProjectError::Io`] if the write fails.
pub fn create_main(project: &Project, contents: &str) -> Result<PathBuf, ProjectError> {
    let rel = PathBuf::from(MAIN_FILE);

    if project.root().join(&rel).exists() {
        return Err(Project::refuse(
            &rel,
            "the file exists and belongs to the developer",
        ));
    }

    project.write(&rel, format_source(contents)?.as_bytes())?;

    Ok(rel)
}

/// Finds the handler file of a form that already exists.
///
/// # Arguments
///
/// * `project` - The project.
/// * `form` - The form's name in the resource file.
///
/// # Returns
///
/// The handler file, relative to the project.
///
/// # Errors
///
/// [`ProjectError::Ownership`] if the project has no such form, or its handler file does not
/// exist yet (create it with [`Project::create_stub`]).
fn handler_file(project: &Project, form: &str) -> Result<PathBuf, ProjectError> {
    let rel = project
        .forms()
        .iter()
        .find(|f| f.name == form)
        .map(|f| f.handler_file.clone())
        .ok_or_else(|| Project::refuse(Path::new(HANDLERS_DIR), "the project has no such form"))?;

    if !project.root().join(&rel).is_file() {
        return Err(Project::refuse(
            &rel,
            "the handler file does not exist; create it with create_stub first",
        ));
    }

    Ok(rel)
}

/// Reads a handler file.
///
/// # Arguments
///
/// * `project` - The project.
/// * `rel` - The handler file, relative to the project.
///
/// # Returns
///
/// The file's text.
///
/// # Errors
///
/// [`ProjectError::Io`] if it cannot be read.
fn read_handler_file(project: &Project, rel: &Path) -> Result<String, ProjectError> {
    std::fs::read_to_string(project.root().join(rel)).map_err(|e| ProjectError::Io {
        path: rel.to_path_buf(),
        message: e.to_string(),
    })
}

/// Appends a handler stub to a form's handler file, leaving every existing byte as it was.
///
/// # Arguments
///
/// * `project` - The project, which is reloaded afterwards.
/// * `form` - The form's name in the resource file.
/// * `stub` - Source holding exactly one function, such as
///   `pub fn on_save(window: &mut Window) {}`.
///
/// # Returns
///
/// The name of the inserted function.
///
/// # Errors
///
/// [`ProjectError::Ownership`] if the form or its handler file is missing, or the file already
/// has a function of that name (the developer's is kept);  [`ProjectError::Handlers`] if `stub`
/// is not exactly one function or the file would no longer parse.
pub fn insert_handler_stub(
    project: &mut Project,
    form: &str,
    stub: &str,
) -> Result<String, ProjectError> {
    let rel = handler_file(project, form)?;
    let existing = read_handler_file(project, &rel)?;
    let mut found = handlers::parse_functions(stub, &rel)?;

    if found.len() != 1 {
        return Err(ProjectError::Handlers {
            path: rel,
            message: "a stub must be exactly one function".to_string(),
        });
    }

    let (name, _, _) = found.remove(0);

    if handlers::function_name_range(&existing, &rel, &name)?.is_some() {
        return Err(Project::refuse(
            &rel,
            &format!("the file already has a function named `{name}`"),
        ));
    }

    let mut updated = existing;

    if !updated.ends_with('\n') && !updated.is_empty() {
        updated.push('\n');
    }

    if !updated.is_empty() {
        updated.push('\n');
    }

    updated.push_str(stub.trim_end());
    updated.push('\n');
    handlers::parse_functions(&updated, &rel)?;
    project.write(&rel, updated.as_bytes())?;
    project.reload_handlers()?;

    Ok(name)
}

/// Renames a handler in its signature, changing only the bytes of the name.
///
/// The body is never touched, so a call to the old name inside it (a recursive call, say) is the
/// developer's to update.
///
/// # Arguments
///
/// * `project` - The project, which is reloaded afterwards.
/// * `form` - The form's name in the resource file.
/// * `old` - The handler's current name.
/// * `new` - The new name, a valid Rust identifier.
///
/// # Returns
///
/// `Ok(())` once the file is written.
///
/// # Errors
///
/// [`ProjectError::Ownership`] if the form, its file, or the handler `old` is missing, or `new`
/// is already used; [`ProjectError::Handlers`] if `new` is not an identifier that keeps the file
/// valid Rust.
pub fn rename_handler(
    project: &mut Project,
    form: &str,
    old: &str,
    new: &str,
) -> Result<(), ProjectError> {
    let rel = handler_file(project, form)?;
    let source = read_handler_file(project, &rel)?;
    let valid = new
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && new.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

    if !valid || new == "_" || KEYWORDS.contains(&new) {
        return Err(ProjectError::Handlers {
            path: rel,
            message: format!("`{new}` is not a valid function name"),
        });
    }

    let range = handlers::function_name_range(&source, &rel, old)?
        .ok_or_else(|| Project::refuse(&rel, &format!("there is no function named `{old}`")))?;

    if old != new && handlers::function_name_range(&source, &rel, new)?.is_some() {
        return Err(Project::refuse(
            &rel,
            &format!("the file already has a function named `{new}`"),
        ));
    }

    let updated = format!("{}{new}{}", &source[..range.start], &source[range.end..]);

    // Parse the result before writing, so a name that breaks the file is refused.
    handlers::parse_functions(&updated, &rel)?;
    project.write(&rel, updated.as_bytes())?;
    project.reload_handlers()
}
