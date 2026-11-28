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

//! # Event wiring generation
//!
//! Connects the developer's handlers to the engine without registration code.  A handler is a
//! function in `src/forms/<form>.rs` named `on_<widget>_<event>` in snake case, such as
//! `on_ok_button_clicked` for the `Clicked` event of the widget whose ID symbol is `OK_BUTTON`.
//! [`generate_wiring`] finds those functions, matches each to its widget and event through the
//! widget registry's event metadata, and writes `src/generated/<form>_wiring.rs`:
//!
//! ```text
//! pub fn wire(engine: &mut Engine<AppState>, form: &FormHandle) {
//!     if let Some(id) = form.widget(ids::main_window::OK_BUTTON) {
//!         engine.on_click(id, on_ok_button_clicked);
//!     }
//! }
//! ```
//!
//! The output depends only on the form file, the handler names, and the registry, and entries are
//! sorted, so the same input always gives the same file.  Functions whose names match no widget
//! and event (helpers) are left alone.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pushrod::form::{FormFile, SymbolKind};
use pushrod::registry::WidgetRegistry;

use super::write_generated;
use crate::project::{GENERATED_DIR, Project, ProjectError, form_module};

/// One handler matched to a widget and event.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wiring {
    /// The handler function's name.
    pub handler: String,
    /// The widget's constant in the generated ID module, such as `OK_BUTTON`.
    pub widget: String,
    /// The event's name, such as `Clicked`.
    pub event: String,
    /// The engine registration method, such as `on_click`.
    pub callback: String,
}

/// Returns the project-relative path of a form's wiring file.
///
/// # Arguments
///
/// * `form` - The form's name in the resource file.
///
/// # Returns
///
/// `src/generated/<form module>_wiring.rs`.
pub fn wiring_path(form: &str) -> PathBuf {
    Path::new(GENERATED_DIR).join(format!("{}_wiring.rs", form_module(form)))
}

/// Builds an error that names the form's handler file.
///
/// # Arguments
///
/// * `project` - The project.
/// * `form` - The form's name.
/// * `message` - What is wrong.
///
/// # Returns
///
/// The error.
fn fail(project: &Project, form: &str, message: String) -> ProjectError {
    let path = project
        .forms()
        .iter()
        .find(|f| f.name == form)
        .map(|f| f.handler_file.clone())
        .unwrap_or_else(|| PathBuf::from(GENERATED_DIR));

    ProjectError::Handlers { path, message }
}

/// Matches a form's handlers to its widgets' events.
///
/// # Arguments
///
/// * `project` - The project; its resource file and handler files are read.
/// * `registry` - The widget registry that knows each widget type's events.
/// * `form` - The form's name in the resource file.
///
/// # Returns
///
/// One [`Wiring`] per matched handler, sorted by handler name.
///
/// # Errors
///
/// [`ProjectError::Handlers`] if the project has no resource file or no such form, a handler is
/// not `pub`, a name matches two widget and event pairs, or it names an event the engine only
/// delivers through `Engine::on_event` (it has no registration method).
pub fn match_handlers(
    project: &Project,
    registry: &WidgetRegistry,
    form: &str,
) -> Result<Vec<Wiring>, ProjectError> {
    let file: &FormFile = project.resource_file().ok_or_else(|| {
        fail(
            project,
            form,
            "the project has no resource file".to_string(),
        )
    })?;

    if !file.forms.iter().any(|f| f.name == form) {
        return Err(fail(
            project,
            form,
            "the project has no such form".to_string(),
        ));
    }

    // Candidates: (handler name, widget constant, event name, callback), over this form's ID
    // symbols and each widget type's events.
    let mut candidates: Vec<(String, Wiring)> = Vec::new();

    for symbol in file.symbols.iter().filter(|s| s.kind == SymbolKind::Id) {
        let Some(object) = file.objects.get(symbol.id as usize) else {
            continue;
        };
        let Some(events) = registry.all_events(&object.type_name) else {
            continue;
        };
        let widget = form_module(&symbol.name);

        for event in events {
            candidates.push((
                format!("on_{widget}_{}", form_module(event.name)),
                Wiring {
                    handler: String::new(),
                    widget: widget.to_ascii_uppercase(),
                    event: event.name.to_string(),
                    callback: event.callback.unwrap_or_default().to_string(),
                },
            ));
        }
    }

    candidates.sort();
    candidates.dedup();

    let mut out = Vec::new();

    for handler in project.handlers_for(form) {
        let mut found: Vec<&Wiring> = candidates
            .iter()
            .filter(|(name, _)| *name == handler.name)
            .map(|(_, w)| w)
            .collect();

        found.dedup();

        let Some(first) = found.first() else {
            continue;
        };

        if found.len() > 1 {
            return Err(fail(
                project,
                form,
                format!(
                    "`{}` could be wired to more than one widget or event",
                    handler.name
                ),
            ));
        }

        if first.callback.is_empty() {
            return Err(fail(
                project,
                form,
                format!(
                    "`{}` names the {} event, which has no registration method; handle it with Engine::on_event",
                    handler.name, first.event
                ),
            ));
        }

        if !handler.signature.trim_start().starts_with("pub") {
            return Err(fail(
                project,
                form,
                format!(
                    "`{}` must be `pub` so the generated wiring can use it",
                    handler.name
                ),
            ));
        }

        out.push(Wiring {
            handler: handler.name.clone(),
            ..(*first).clone()
        });
    }

    out.sort();

    // Two events can share one registration method (focus gained and lost both use
    // `on_focus_change`); wiring both would register that method twice and fire each handler on
    // both events.
    let mut used: HashMap<(&str, &str), &str> = HashMap::new();

    for w in &out {
        if let Some(first) = used.insert((&w.widget, &w.callback), &w.handler) {
            return Err(fail(
                project,
                form,
                format!(
                    "`{first}` and `{}` both use `Engine::{}` on `{}`; keep one",
                    w.handler, w.callback, w.widget
                ),
            ));
        }
    }

    Ok(out)
}

/// Generates the Rust source of a form's wiring file, before formatting.
///
/// # Arguments
///
/// * `project` - The project; its resource file and handler files are read.
/// * `registry` - The widget registry that knows each widget type's events.
/// * `form` - The form's name in the resource file.
///
/// # Returns
///
/// The source text of `src/generated/<form>_wiring.rs`.
///
/// # Errors
///
/// The errors of [`match_handlers`].
pub fn generate_wiring(
    project: &Project,
    registry: &WidgetRegistry,
    form: &str,
) -> Result<String, ProjectError> {
    let wiring = match_handlers(project, registry, form)?;
    let module = form_module(form);
    let mut out = format!(
        "// Generated by Chassis from the form `{}`.  Do not edit: this file is rewritten in full.\n\n",
        form.replace(['\n', '\r'], " ")
    );

    out.push_str(
        "use crate::AppState;\nuse pushrod::engine::Engine;\nuse pushrod::form::FormHandle;\n",
    );

    if wiring.is_empty() {
        out.push_str(
            "\n/// Registers the form's handlers; the form has none yet.\npub fn wire(_engine: &mut Engine<AppState>, _form: &FormHandle) {}\n",
        );

        return Ok(out);
    }

    let names: Vec<&str> = wiring.iter().map(|w| w.handler.as_str()).collect();

    out.push_str(&format!(
        "use crate::forms::{module}::{{{}}};\nuse super::ids;\n",
        names.join(", ")
    ));
    out.push_str(
        "\n/// Registers each of the form's handlers with the engine, by widget ID and event.\npub fn wire(engine: &mut Engine<AppState>, form: &FormHandle) {\n",
    );

    for w in &wiring {
        out.push_str(&format!(
            "    if let Some(id) = form.widget(ids::{module}::{}) {{\n        engine.{}(id, {});\n    }}\n",
            w.widget, w.callback, w.handler
        ));
    }

    out.push_str("}\n");

    Ok(out)
}

/// Generates a form's wiring file and writes it under `src/generated`, replacing the old one.
///
/// # Arguments
///
/// * `project` - The project.
/// * `registry` - The widget registry that knows each widget type's events.
/// * `form` - The form's name in the resource file.
///
/// # Returns
///
/// The path of the file, relative to the project.
///
/// # Errors
///
/// The errors of [`generate_wiring`] and [`write_generated`].
pub fn write_wiring(
    project: &Project,
    registry: &WidgetRegistry,
    form: &str,
) -> Result<PathBuf, ProjectError> {
    let rel = wiring_path(form);

    write_generated(project, &rel, &generate_wiring(project, registry, form)?)?;

    Ok(rel)
}
