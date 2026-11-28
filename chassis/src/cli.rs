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

//! # Command-line generator
//!
//! `chassis generate [project]` regenerates everything under `src/generated/` from the resource
//! file and the handler files, and `chassis check [project]` reports what is wrong without
//! writing anything.  Both use the same functions the designer uses ([`render`]), so a file
//! written here is byte for byte the file the designer writes.
//!
//! `check` finds three kinds of problem ([`ProblemKind`]) and exits non-zero if it finds any:
//!
//! * **Invalid**: a name that cannot become a Rust identifier, or a handler that cannot be wired
//!   (not `pub`, or it could mean two widgets or events).
//! * **Orphaned**: a function named `on_...` that matches no widget and event of its form, or a
//!   handler file for a form that does not exist.
//! * **Missing**: a generated file that does not exist or is out of date, so a handler is not
//!   wired (or a removed handler is still wired).  `chassis generate` fixes it.

use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use pushrod::form::codegen::generate_ids;
use pushrod::registry::WidgetRegistry;

use crate::codegen::wiring::{match_handlers, wiring_path};
use crate::codegen::{format_source, write_generated};
use crate::project::{Project, ProjectError, form_module};

/// The exit code when everything is fine.
pub const EXIT_OK: i32 = 0;
/// The exit code when `check` found problems.
pub const EXIT_PROBLEMS: i32 = 1;
/// The exit code when the command line is wrong or the project cannot be loaded or written.
pub const EXIT_FAILURE: i32 = 2;

/// The command line of `chassis`.
#[derive(Debug, Parser)]
#[command(name = "chassis", about = "The Pushrod form designer's code generator")]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `chassis`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Regenerates `src/generated/` from the resource file and the handler files.
    Generate {
        /// The project directory, where `Cargo.toml` is.
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Reports missing or orphaned handlers and invalid names; exits non-zero on problems.
    Check {
        /// The project directory, where `Cargo.toml` is.
        #[arg(default_value = ".")]
        project: PathBuf,
    },
}

/// The kind of a problem found by [`check`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemKind {
    /// A name or handler that cannot be turned into code.
    Invalid,
    /// A handler that matches nothing in its form.
    Orphaned,
    /// A generated file that does not exist or is out of date.
    Missing,
}

/// One problem found by [`check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// What kind of problem it is.
    pub kind: ProblemKind,
    /// The file it concerns, relative to the project, if one file is to blame.
    pub file: Option<PathBuf>,
    /// What is wrong, in one line.
    pub message: String,
}

impl std::fmt::Display for Problem {
    /// Writes `<kind>: <file>: <message>`, leaving out the file when there is none.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatter's result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            ProblemKind::Invalid => "invalid",
            ProblemKind::Orphaned => "orphaned",
            ProblemKind::Missing => "missing",
        };

        match &self.file {
            Some(file) => write!(f, "{kind}: {}: {}", file.display(), self.message),
            None => write!(f, "{kind}: {}", self.message),
        }
    }
}

/// The project-relative path of the generated ID constants.
pub const IDS_FILE: &str = "src/generated/ids.rs";
/// The project-relative path of the generated module list.
pub const MOD_FILE: &str = "src/generated/mod.rs";

/// Produces every generated file of a project, formatted, without writing any.
///
/// This is the one place that decides what `src/generated/` holds; the designer and the command
/// line both call it.
///
/// # Arguments
///
/// * `project` - The project.
/// * `registry` - The widget registry that knows each widget type's events.
///
/// # Returns
///
/// `(path relative to the project, contents)` for `ids.rs`, one `<form>_wiring.rs` per form, and
/// `mod.rs`, in that order.
///
/// # Errors
///
/// [`ProjectError::Resources`] if the project has no resource file or the IDs cannot be
/// generated (a name that is not a usable Rust identifier), the errors of
/// [`match_handlers`], and those of
/// [`format_source`].
pub fn render(
    project: &Project,
    registry: &WidgetRegistry,
) -> Result<Vec<(PathBuf, String)>, ProjectError> {
    let file = project
        .resource_file()
        .ok_or_else(|| ProjectError::Resources {
            path: PathBuf::from("*.prsc"),
            message: "the project has no resource file".to_string(),
        })?;
    let ids = generate_ids(file).map_err(|e| ProjectError::Resources {
        path: project
            .resource_path()
            .map(PathBuf::from)
            .unwrap_or_default(),
        message: e.to_string(),
    })?;
    let mut out = vec![(PathBuf::from(IDS_FILE), format_source(&ids)?)];
    let mut modules = vec!["ids".to_string()];

    for form in project.forms() {
        let source = crate::codegen::wiring::generate_wiring(project, registry, &form.name)?;

        out.push((wiring_path(&form.name), format_source(&source)?));
        modules.push(format!("{}_wiring", form_module(&form.name)));
    }

    modules.sort();

    let list: String = modules.iter().map(|m| format!("pub mod {m};\n")).collect();
    let text =
        format!("// Generated by Chassis.  Do not edit: this file is rewritten in full.\n\n{list}");

    out.push((PathBuf::from(MOD_FILE), format_source(&text)?));

    Ok(out)
}

/// Regenerates `src/generated/` and removes nothing.
///
/// # Arguments
///
/// * `project` - The project.
/// * `registry` - The widget registry.
///
/// # Returns
///
/// The paths written, relative to the project.
///
/// # Errors
///
/// The errors of [`render`] and [`write_generated`]; nothing is written if [`render`] fails.
pub fn generate(
    project: &Project,
    registry: &WidgetRegistry,
) -> Result<Vec<PathBuf>, ProjectError> {
    let files = render(project, registry)?;
    let mut written = Vec::new();

    for (rel, text) in files {
        write_generated(project, &rel, &text)?;
        written.push(rel);
    }

    Ok(written)
}

/// Checks a project and reports every problem, writing nothing.
///
/// # Arguments
///
/// * `project` - The project.
/// * `registry` - The widget registry.
///
/// # Returns
///
/// The problems, in the order found; empty if the project is sound.
pub fn check(project: &Project, registry: &WidgetRegistry) -> Vec<Problem> {
    let mut problems = Vec::new();

    if project.resource_file().is_none() {
        problems.push(Problem {
            kind: ProblemKind::Invalid,
            file: None,
            message: "the project has no resource file".to_string(),
        });

        return problems;
    }

    // Orphaned handler files: functions in a file no form owns.
    let mut seen = HashSet::new();

    for h in project.handlers().iter().filter(|h| h.form.is_none()) {
        if seen.insert(h.file.clone()) {
            problems.push(Problem {
                kind: ProblemKind::Orphaned,
                file: Some(h.file.clone()),
                message: "the file belongs to no form of the resource file".to_string(),
            });
        }
    }

    // Per form: unusable handlers, and `on_` functions that match nothing.
    for form in project.forms() {
        match match_handlers(project, registry, &form.name) {
            Ok(wired) => {
                let wired: HashSet<&str> = wired.iter().map(|w| w.handler.as_str()).collect();

                for h in project.handlers_for(&form.name) {
                    if h.name.starts_with("on_") && !wired.contains(h.name.as_str()) {
                        problems.push(Problem {
                            kind: ProblemKind::Orphaned,
                            file: Some(h.file.clone()),
                            message: format!(
                                "line {}: `{}` matches no widget and event of `{}`",
                                h.line, h.name, form.name
                            ),
                        });
                    }
                }
            }
            Err(e) => problems.push(Problem {
                kind: ProblemKind::Invalid,
                file: Some(form.handler_file.clone()),
                message: e.to_string(),
            }),
        }
    }

    // Names, then generated files that are absent or stale.  Skipped when anything above made
    // the output impossible, since the same error would be reported twice.
    match render(project, registry) {
        Ok(files) => {
            for (rel, text) in files {
                match std::fs::read_to_string(project.root().join(&rel)) {
                    Ok(on_disk) if on_disk == text => {}
                    Ok(_) => problems.push(Problem {
                        kind: ProblemKind::Missing,
                        file: Some(rel),
                        message: "out of date; run `chassis generate`".to_string(),
                    }),
                    Err(_) => problems.push(Problem {
                        kind: ProblemKind::Missing,
                        file: Some(rel),
                        message: "does not exist; run `chassis generate`".to_string(),
                    }),
                }
            }
        }
        Err(ProjectError::Resources { message, .. }) => problems.push(Problem {
            kind: ProblemKind::Invalid,
            file: None,
            message,
        }),
        Err(_) if problems.iter().any(|p| p.kind == ProblemKind::Invalid) => {}
        Err(e) => problems.push(Problem {
            kind: ProblemKind::Invalid,
            file: None,
            message: e.to_string(),
        }),
    }

    problems
}

/// Runs the command line.
///
/// # Arguments
///
/// * `args` - The arguments, starting with the program name.
/// * `registry` - The widget registry.
/// * `out` - Where normal output goes.
/// * `err` - Where errors and problems go.
///
/// # Returns
///
/// The exit code: [`EXIT_OK`], [`EXIT_PROBLEMS`] (`check` found problems), or [`EXIT_FAILURE`]
/// (bad arguments, or the project could not be loaded or written).  `--help` prints to `out`
/// and returns [`EXIT_OK`].
pub fn run<I, T>(
    args: I,
    registry: &WidgetRegistry,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) if !e.use_stderr() => {
            let _ = write!(out, "{e}");
            return EXIT_OK;
        }
        Err(e) => {
            let _ = write!(err, "{e}");
            return EXIT_FAILURE;
        }
    };
    let (dir, checking) = match cli.command {
        Command::Generate { project } => (project, false),
        Command::Check { project } => (project, true),
    };
    let project = match Project::load(&dir) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(err, "{e}");
            return EXIT_FAILURE;
        }
    };

    if checking {
        let problems = check(&project, registry);

        for p in &problems {
            let _ = writeln!(err, "{p}");
        }

        if problems.is_empty() {
            let _ = writeln!(out, "ok");
            return EXIT_OK;
        }

        let _ = writeln!(err, "{} problem(s)", problems.len());

        return EXIT_PROBLEMS;
    }

    match generate(&project, registry) {
        Ok(files) => {
            for f in files {
                let _ = writeln!(out, "wrote {}", f.display());
            }

            EXIT_OK
        }
        Err(e) => {
            let _ = writeln!(err, "{e}");
            EXIT_FAILURE
        }
    }
}
