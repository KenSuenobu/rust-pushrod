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

//! # ID constant generation
//!
//! Turns the ID symbols of a form file into Rust constants, so a program attaches callbacks by a
//! name the compiler checks.  Renaming or removing a widget in the designer removes the constant
//! and breaks the build, instead of silently attaching to a different widget.
//!
//! Each form becomes a module (the form's name in `snake_case`) and each [`SymbolKind::Id`]
//! symbol of an object in that form becomes a constant (the symbol's name in `SHOUTING_CASE`):
//!
//! ```
//! use pushrod::form::codegen::generate_ids;
//! use pushrod::form::{FormFile, ObjectRecord, SymbolKind};
//!
//! let mut file = FormFile::new();
//! let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
//! let ok = file.add_object(Some(root), ObjectRecord::new("pushrod.button"));
//! file.add_form("main", root);
//! file.add_symbol(SymbolKind::Id, ok, "ok");
//!
//! let source = generate_ids(&file).unwrap();
//! assert!(source.contains("pub const OK: FormIndex = FormIndex(1);"));
//! ```
//!
//! Resources get the same treatment with typed IDs (see [`crate::resource`]): a `forms` module holds
//! one [`FormId`](crate::resource::FormId) per form (its position in the file, counting from 1), and
//! `strings`, `images`, `icons`, `files`, `menus`, and `alerts` modules hold one constant per
//! resource of that kind, such as `strings::SAVE_FAILED: StringId`.  A kind with no resources has no
//! module, and a form cannot take one of these seven names.
//!
//! The output is deterministic (forms in file order, constants by object number) and already
//! formatted the way `rustfmt` formats it, so regenerating an unchanged file changes nothing.
//! [`write_ids`] is for `build.rs`; the `pushrod-form-ids` command does the same from a shell.
//! Generation is an offline build step with no running engine, so it emits no engine event.

use super::format::{FormError, FormFile, NO_OBJECT, SymbolKind};
use super::resources::ResourceTable;
use crate::error::{Error, Result};
use std::collections::HashSet;
use std::path::Path;

/// Words that cannot be used as a module or constant name.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while",
];

/// The module names of the generated resource modules; a form cannot be named like one.
const RESERVED_MODULES: &[&str] = &[
    "forms", "strings", "images", "icons", "files", "menus", "alerts",
];

/// Splits a name into lowercase words at every character that is not a letter or digit and at
/// every lower-to-upper case change, so `OkButton`, `ok-button`, and `OK_BUTTON` agree.
///
/// # Arguments
///
/// * `name` - The name from the form file.
///
/// # Returns
///
/// The lowercase words, possibly none.
fn words(name: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;

    for c in name.chars() {
        if !c.is_ascii_alphanumeric() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            previous_lower = false;
            continue;
        }
        if c.is_ascii_uppercase() && previous_lower && !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
        previous_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        current.push(c.to_ascii_lowercase());
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Builds an identifier from a name, checking that it is usable.
///
/// # Arguments
///
/// * `what` - What is being named, for errors (`"form"` or `"ID symbol"`).
/// * `name` - The name from the form file.
/// * `upper` - `true` for `SHOUTING_CASE`, `false` for `snake_case`.
///
/// # Returns
///
/// The identifier.
///
/// # Errors
///
/// Returns [`FormError::Invalid`] if the name has no letters or digits, starts with a digit, or is
/// a Rust keyword.
fn identifier(what: &str, name: &str, upper: bool) -> std::result::Result<String, FormError> {
    let joined = words(name).join("_");
    let ident = if upper {
        joined.to_ascii_uppercase()
    } else {
        joined
    };

    if ident.is_empty() {
        return Err(FormError::invalid(format!(
            "{what} name `{name}` has no letters or digits"
        )));
    }
    if ident.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(FormError::invalid(format!(
            "{what} name `{name}` starts with a digit"
        )));
    }
    if !upper && KEYWORDS.contains(&ident.as_str()) {
        return Err(FormError::invalid(format!(
            "{what} name `{name}` is the Rust keyword `{ident}`"
        )));
    }
    Ok(ident)
}

/// Returns every object number in a form's tree, depth first.
///
/// # Arguments
///
/// * `file` - The form file.
/// * `root` - The form's root object number.
///
/// # Returns
///
/// The object numbers, root first.
fn subtree(file: &FormFile, root: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut stack = vec![root];

    while let Some(number) = stack.pop() {
        if number == NO_OBJECT || out.len() > file.objects.len() {
            continue;
        }
        out.push(number);
        stack.extend(file.children(number).into_iter().rev());
    }
    out
}

/// Writes a module of constants of one ID type.
///
/// # Arguments
///
/// * `out` - The source text being built.
/// * `module` - The module's name.
/// * `type_name` - The ID type of every constant, such as `StringId`.
/// * `what` - What each constant names, for documentation and errors, such as `string`.
/// * `entries` - The number and name of each constant.
///
/// # Errors
///
/// Returns [`FormError::Invalid`] if a name is not a usable identifier or two names become the same
/// constant.
fn write_module<'a>(
    out: &mut String,
    module: &str,
    type_name: &str,
    what: &str,
    entries: impl Iterator<Item = (u32, &'a str)>,
) -> std::result::Result<(), FormError> {
    let mut constants: Vec<(u32, String, &str)> = Vec::new();
    let mut seen = HashSet::new();

    for (id, name) in entries {
        let constant = identifier(what, name, true)?;

        if !seen.insert(constant.clone()) {
            return Err(FormError::invalid(format!(
                "two {what}s are named `{constant}` in Rust"
            )));
        }
        constants.push((id, constant, name));
    }
    constants.sort();

    out.push('\n');
    out.push_str(&format!(
        "#[allow(dead_code)]\npub mod {module} {{\n    use pushrod::resource::{type_name};\n"
    ));
    for (id, constant, name) in &constants {
        out.push_str(&format!(
            "\n    /// The {what} `{}`.\n    pub const {constant}: {type_name} = {type_name}({id});\n",
            name.replace(['\n', '\r'], " ")
        ));
    }
    out.push_str("}\n");
    Ok(())
}

/// Writes the module of one resource table, or nothing if the table is empty.
///
/// # Arguments
///
/// * `out` - The source text being built.
/// * `module` - The module's name.
/// * `type_name` - The ID type of every constant.
/// * `what` - What each constant names.
/// * `table` - The resource table.
///
/// # Errors
///
/// See [`write_module`].
fn write_table<T>(
    out: &mut String,
    module: &str,
    type_name: &str,
    what: &str,
    table: &ResourceTable<T>,
) -> std::result::Result<(), FormError> {
    if table.is_empty() {
        return Ok(());
    }
    write_module(
        out,
        module,
        type_name,
        what,
        table.iter().map(|e| (e.id, e.name.as_str())),
    )
}

/// Generates the Rust source of the ID constants of a form file.
///
/// # Arguments
///
/// * `file` - The form file.
///
/// # Returns
///
/// The source text: a `forms` module of [`FormId`](crate::resource::FormId) constants; one
/// `pub mod` per form, holding one `pub const NAME: FormIndex` per ID symbol of an object in that
/// form; and one module per non-empty resource kind (`strings`, `images`, `icons`, `files`,
/// `menus`, `alerts`) of typed constants.  Symbols that name no object of any form are left out.
///
/// # Errors
///
/// Returns [`FormError::Invalid`] if the file is inconsistent, two forms have the same module name,
/// a form is named like a resource module (`forms`, `strings`, ...), two symbols of a form or two
/// resources of a kind have the same constant name, or a name is not a usable Rust identifier.
pub fn generate_ids(file: &FormFile) -> std::result::Result<String, FormError> {
    file.validate()?;

    let mut out =
        String::from("// Generated by pushrod::form::codegen from a form file.  Do not edit.\n");
    let mut modules = HashSet::new();

    if !file.forms.is_empty() {
        let forms = file.forms.iter().zip(1u32..);

        write_module(
            &mut out,
            "forms",
            "FormId",
            "form",
            forms.map(|(form, id)| (id, form.name.as_str())),
        )?;
    }

    for form in &file.forms {
        let module = identifier("form", &form.name, false)?;

        if RESERVED_MODULES.contains(&module.as_str()) {
            return Err(FormError::invalid(format!(
                "form `{}` is named like the generated `{module}` module",
                form.name
            )));
        }

        if !modules.insert(module.clone()) {
            return Err(FormError::invalid(format!(
                "two forms are named `{module}` in Rust"
            )));
        }

        let members: HashSet<u32> = subtree(file, form.root).into_iter().collect();
        let mut symbols: Vec<(u32, String)> = Vec::new();

        for symbol in file.symbols.iter().filter(|s| s.kind == SymbolKind::Id) {
            if members.contains(&symbol.id) {
                symbols.push((symbol.id, identifier("ID symbol", &symbol.name, true)?));
            }
        }
        symbols.sort();
        symbols.dedup();

        let mut seen = HashSet::new();

        for (id, name) in &symbols {
            if !seen.insert(name.clone()) {
                return Err(FormError::invalid(format!(
                    "form `{}` has two ID symbols named `{name}` (object {id})",
                    form.name
                )));
            }
        }

        out.push('\n');
        out.push_str("#[allow(dead_code)]\n");
        if symbols.is_empty() {
            out.push_str(&format!("pub mod {module} {{}}\n"));
            continue;
        }
        out.push_str(&format!("pub mod {module} {{\n"));
        out.push_str("    use pushrod::form::FormIndex;\n");
        for (id, name) in &symbols {
            out.push_str(&format!(
                "\n    /// Object {id} of form `{}`.\n    pub const {name}: FormIndex = FormIndex({id});\n",
                form.name.replace(['\n', '\r'], " ")
            ));
        }
        out.push_str("}\n");
    }

    let resources = &file.resources;

    write_table(
        &mut out,
        "strings",
        "StringId",
        "string",
        &resources.strings,
    )?;
    write_table(&mut out, "images", "ImageId", "image", &resources.images)?;
    write_table(&mut out, "icons", "IconId", "icon", &resources.icons)?;
    write_table(&mut out, "files", "FileId", "file", &resources.files)?;
    write_table(&mut out, "menus", "MenuId", "menu", &resources.menus)?;
    write_table(&mut out, "alerts", "AlertId", "alert", &resources.alerts)?;
    Ok(out)
}

/// Reads a form file and writes the Rust source of its ID constants, for `build.rs`.  The output
/// file is only rewritten when its text would change, so an unchanged form does not rebuild the
/// program.
///
/// # Arguments
///
/// * `file` - The path of the form file.
/// * `out_rs` - The path of the Rust file to write; an existing file is replaced.
///
/// # Returns
///
/// `Ok(())` once the output file is up to date.
///
/// # Errors
///
/// Returns [`Error::FormFile`] naming the path if a file cannot be read or written, the form file
/// is damaged, or the names cannot be turned into identifiers (see [`generate_ids`]).
pub fn write_ids(file: impl AsRef<Path>, out_rs: impl AsRef<Path>) -> Result<()> {
    let (file, out_rs) = (file.as_ref(), out_rs.as_ref());
    let shown = file.display().to_string();
    let bytes = std::fs::read(file).map_err(|e| Error::form_file(shown.clone(), e.to_string()))?;
    let form =
        FormFile::from_bytes(&bytes).map_err(|e| Error::form_file(shown.clone(), e.to_string()))?;
    let source = generate_ids(&form).map_err(|e| Error::form_file(shown, e.to_string()))?;
    let out_shown = out_rs.display().to_string();

    if std::fs::read_to_string(out_rs).is_ok_and(|existing| existing == source) {
        return Ok(());
    }
    std::fs::write(out_rs, source).map_err(|e| Error::form_file(out_shown, e.to_string()))
}
