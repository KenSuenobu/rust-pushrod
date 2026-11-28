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

//! # The project model
//!
//! A Chassis project is a Cargo crate.  The model knows which of its files Chassis owns and which
//! belong to the developer, so Chassis can never overwrite user code.
//!
//! | Path | Owner | Chassis may |
//! |---|---|---|
//! | `Cargo.toml` | the developer | read |
//! | `*.prsc` (the resource file) | Chassis | read and write, through Pushrod |
//! | `src/forms/<form>.rs` (handlers) | the developer | create once, as a stub; never overwrite |
//! | `src/generated/**` | Chassis | write freely; it is regenerated |
//! | everything else | the developer | read |
//!
//! Every write goes through [`Project::write_generated`], [`Project::create_stub`], or
//! [`Project::save_resources`], which refuse anything else with [`ProjectError::Ownership`].
//! The model reads the code on disk each time it loads, so it always matches what the developer
//! has written.

mod error;
pub(crate) mod handlers;
mod manifest;

use std::path::{Component, Path, PathBuf};

use pushrod::form::FormFile;

pub use error::ProjectError;
pub use handlers::Handler;
pub use manifest::Package;

/// The folder of handler files, relative to the project.
pub const HANDLERS_DIR: &str = "src/forms";
/// The folder of Chassis-owned generated code, relative to the project.
pub const GENERATED_DIR: &str = "src/generated";

/// Who owns a file of the project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// The developer's file; Chassis only reads it.
    User,
    /// A handler file under `src/forms`: the developer's, except that Chassis may create a stub
    /// when the file does not exist.
    Handler,
    /// The resource file: Chassis reads and writes it.
    Resource,
    /// A file under `src/generated`: Chassis writes it and may replace it at any time.
    Generated,
}

/// One form of the resource file and where its handlers live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormInfo {
    /// The form's name in the resource file.
    pub name: String,
    /// The handler file, relative to the project: `src/forms/<module>.rs`.
    pub handler_file: PathBuf,
    /// Whether the handler file exists yet.
    pub has_handler_file: bool,
}

/// How many resources of each kind the resource file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceCounts {
    /// Strings.
    pub strings: usize,
    /// Images.
    pub images: usize,
    /// Icons.
    pub icons: usize,
    /// Files.
    pub files: usize,
    /// Menus.
    pub menus: usize,
    /// Alerts.
    pub alerts: usize,
}

/// A loaded Chassis project.
#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    package: Package,
    resource_path: Option<PathBuf>,
    resources: Option<FormFile>,
    forms: Vec<FormInfo>,
    handlers: Vec<Handler>,
}

/// Turns a form name into the module name of its handler file, such as `MainWindow` into
/// `main_window`.
///
/// # Arguments
///
/// * `name` - The form's name.
///
/// # Returns
///
/// Lowercase words joined by underscores; anything that is not a letter or digit separates words.
pub fn form_module(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if c.is_ascii_uppercase() && prev_lower {
                out.push('_');
            }

            out.push(c.to_ascii_lowercase());
            prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        } else {
            if !out.is_empty() && !out.ends_with('_') {
                out.push('_');
            }

            prev_lower = false;
        }
    }

    out.trim_end_matches('_').to_string()
}

/// Checks a project-relative path stays inside the project.
///
/// # Arguments
///
/// * `rel` - The path.
///
/// # Returns
///
/// `true` if it is relative and has no `..`, root, or prefix components.
fn is_contained(rel: &Path) -> bool {
    rel.components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

impl Project {
    /// Loads a project directory: its manifest, resource file, and handler files.
    ///
    /// The resource file is the one named by `[package.metadata.chassis] resources`, or else the
    /// only `*.prsc` file in the project directory; a project with none has no forms yet.
    ///
    /// # Arguments
    ///
    /// * `dir` - The project directory, where `Cargo.toml` is.
    ///
    /// # Returns
    ///
    /// The project.
    ///
    /// # Errors
    ///
    /// [`ProjectError`] if the manifest, the resource file, or a handler file cannot be read, if
    /// several `*.prsc` files exist and none is named, or if a handler file does not parse.
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let root = dir.as_ref().to_path_buf();
        let package = Package::read(&root.join("Cargo.toml"))?;
        let resource_path = Self::find_resource_file(&root, &package)?;
        let resources = match &resource_path {
            Some(path) => Some(Self::read_resources(path)?),
            None => None,
        };
        let forms = Self::forms_of(&root, resources.as_ref());
        let handlers = Self::read_handlers(&root, &forms)?;

        Ok(Project {
            root,
            package,
            resource_path,
            resources,
            forms,
            handlers,
        })
    }

    /// Finds the resource file.
    ///
    /// # Arguments
    ///
    /// * `root` - The project directory.
    /// * `package` - The manifest, which may name the file.
    ///
    /// # Returns
    ///
    /// The file's path, or `None` if the project has none.
    ///
    /// # Errors
    ///
    /// [`ProjectError`] if the named file is missing or outside the project, or the choice is
    /// ambiguous.
    fn find_resource_file(root: &Path, package: &Package) -> Result<Option<PathBuf>, ProjectError> {
        if let Some(name) = &package.resources {
            let rel = Path::new(name);

            if !is_contained(rel) {
                return Err(ProjectError::Manifest {
                    message: format!("resources `{name}` is outside the project"),
                });
            }

            let path = root.join(rel);

            if !path.is_file() {
                return Err(ProjectError::Io {
                    path,
                    message: "the resource file named in Cargo.toml does not exist".to_string(),
                });
            }

            return Ok(Some(path));
        }

        let entries = std::fs::read_dir(root).map_err(|e| ProjectError::Io {
            path: root.to_path_buf(),
            message: e.to_string(),
        })?;
        let mut found: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "prsc"))
            .collect();

        found.sort();

        match found.len() {
            0 => Ok(None),
            1 => Ok(found.pop()),
            _ => Err(ProjectError::Manifest {
                message: "several .prsc files found; name one in [package.metadata.chassis] \
                          resources"
                    .to_string(),
            }),
        }
    }

    /// Reads a resource file through Pushrod.
    ///
    /// # Arguments
    ///
    /// * `path` - The resource file.
    ///
    /// # Returns
    ///
    /// The parsed file.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Resources`] if it cannot be read or is not a valid resource file.
    fn read_resources(path: &Path) -> Result<FormFile, ProjectError> {
        let fail = |message: String| ProjectError::Resources {
            path: path.to_path_buf(),
            message,
        };
        let bytes = std::fs::read(path).map_err(|e| fail(e.to_string()))?;
        let file = FormFile::from_bytes(&bytes).map_err(|e| fail(e.to_string()))?;

        file.validate().map_err(|e| fail(e.to_string()))?;

        Ok(file)
    }

    /// Lists the forms of a resource file with their handler files.
    ///
    /// # Arguments
    ///
    /// * `root` - The project directory.
    /// * `file` - The resource file, if there is one.
    ///
    /// # Returns
    ///
    /// One entry per form, in file order.
    fn forms_of(root: &Path, file: Option<&FormFile>) -> Vec<FormInfo> {
        file.map(|f| {
            f.forms
                .iter()
                .map(|form| {
                    let handler_file =
                        Path::new(HANDLERS_DIR).join(format!("{}.rs", form_module(&form.name)));

                    FormInfo {
                        name: form.name.clone(),
                        has_handler_file: root.join(&handler_file).is_file(),
                        handler_file,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
    }

    /// Reads every handler file under `src/forms` (except `mod.rs`).
    ///
    /// # Arguments
    ///
    /// * `root` - The project directory.
    /// * `forms` - The project's forms, to match files to forms.
    ///
    /// # Returns
    ///
    /// The functions found, ordered by file and then by position.
    ///
    /// # Errors
    ///
    /// [`ProjectError`] if the folder or a file cannot be read, or a file does not parse.
    fn read_handlers(root: &Path, forms: &[FormInfo]) -> Result<Vec<Handler>, ProjectError> {
        let dir = root.join(HANDLERS_DIR);

        if !dir.is_dir() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&dir).map_err(|e| ProjectError::Io {
            path: dir.clone(),
            message: e.to_string(),
        })?;
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.is_file()
                    && p.extension().is_some_and(|x| x == "rs")
                    && p.file_name().is_some_and(|n| n != "mod.rs")
            })
            .collect();
        let mut handlers = Vec::new();

        files.sort();

        for path in files {
            let rel = Path::new(HANDLERS_DIR).join(path.file_name().unwrap_or_default());
            let source = std::fs::read_to_string(&path).map_err(|e| ProjectError::Io {
                path: path.clone(),
                message: e.to_string(),
            })?;
            let form = forms
                .iter()
                .find(|f| f.handler_file == rel)
                .map(|f| f.name.clone());

            for (name, signature, line) in handlers::parse_functions(&source, &rel)? {
                handlers.push(Handler {
                    form: form.clone(),
                    name,
                    signature,
                    file: rel.clone(),
                    line,
                });
            }
        }

        Ok(handlers)
    }

    /// Returns the project directory.
    ///
    /// # Returns
    ///
    /// The directory the project was loaded from.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the crate's manifest data.
    ///
    /// # Returns
    ///
    /// The package name, version, and resource file setting.
    pub fn package(&self) -> &Package {
        &self.package
    }

    /// Returns the resource file's path.
    ///
    /// # Returns
    ///
    /// The path, or `None` if the project has no resource file.
    pub fn resource_path(&self) -> Option<&Path> {
        self.resource_path.as_deref()
    }

    /// Returns the loaded resource file.
    ///
    /// # Returns
    ///
    /// The parsed file, or `None` if the project has none.
    pub fn resource_file(&self) -> Option<&FormFile> {
        self.resources.as_ref()
    }

    /// Reports the project's forms.
    ///
    /// # Returns
    ///
    /// The forms in resource-file order, each with its handler file.
    pub fn forms(&self) -> &[FormInfo] {
        &self.forms
    }

    /// Counts the resources of each kind.
    ///
    /// # Returns
    ///
    /// The counts; all zero when there is no resource file.
    pub fn resource_counts(&self) -> ResourceCounts {
        self.resources
            .as_ref()
            .map(|f| ResourceCounts {
                strings: f.resources.strings.iter().count(),
                images: f.resources.images.iter().count(),
                icons: f.resources.icons.iter().count(),
                files: f.resources.files.iter().count(),
                menus: f.resources.menus.iter().count(),
                alerts: f.resources.alerts.iter().count(),
            })
            .unwrap_or_default()
    }

    /// Reports every function found in the handler files.
    ///
    /// # Returns
    ///
    /// The handlers, ordered by file and then by position.
    pub fn handlers(&self) -> &[Handler] {
        &self.handlers
    }

    /// Reports the handlers in one form's handler file.
    ///
    /// # Arguments
    ///
    /// * `form` - The form's name.
    ///
    /// # Returns
    ///
    /// The form's handlers, in source order; empty if the form has no handler file.
    pub fn handlers_for<'a>(&'a self, form: &'a str) -> impl Iterator<Item = &'a Handler> {
        self.handlers
            .iter()
            .filter(move |h| h.form.as_deref() == Some(form))
    }

    /// Says who owns a file.
    ///
    /// # Arguments
    ///
    /// * `rel` - The path, relative to the project directory.
    ///
    /// # Returns
    ///
    /// The file's [`Ownership`].  Paths that leave the project are the developer's.
    pub fn ownership(&self, rel: impl AsRef<Path>) -> Ownership {
        let rel = rel.as_ref();

        if !is_contained(rel) {
            return Ownership::User;
        }

        let rel: PathBuf = rel
            .components()
            .filter(|c| *c != Component::CurDir)
            .collect();

        if rel.starts_with(GENERATED_DIR) {
            Ownership::Generated
        } else if rel.starts_with(HANDLERS_DIR) {
            Ownership::Handler
        } else if self
            .resource_path
            .as_ref()
            .and_then(|p| p.strip_prefix(&self.root).ok())
            .is_some_and(|p| p == rel)
        {
            Ownership::Resource
        } else {
            Ownership::User
        }
    }

    /// Builds the refusal error for a path.
    ///
    /// # Arguments
    ///
    /// * `rel` - The path, relative to the project.
    /// * `message` - Why the write is refused.
    ///
    /// # Returns
    ///
    /// The error.
    pub(crate) fn refuse(rel: &Path, message: &str) -> ProjectError {
        ProjectError::Ownership {
            path: rel.to_path_buf(),
            message: message.to_string(),
        }
    }

    /// Writes a file that belongs to Chassis under `src/generated`, replacing any earlier copy.
    ///
    /// # Arguments
    ///
    /// * `rel` - The path relative to the project; it must be inside `src/generated`.
    /// * `contents` - The file's new contents.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the file is written; missing folders are created.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Ownership`] if `rel` is not inside `src/generated`, and
    /// [`ProjectError::Io`] if the write fails.
    pub fn write_generated(
        &self,
        rel: impl AsRef<Path>,
        contents: &str,
    ) -> Result<(), ProjectError> {
        let rel = rel.as_ref();

        if self.ownership(rel) != Ownership::Generated {
            return Err(Self::refuse(
                rel,
                "only files under src/generated belong to Chassis",
            ));
        }

        self.write(rel, contents.as_bytes())
    }

    /// Creates the handler file of a form with starter code, only if it does not exist yet.
    ///
    /// # Arguments
    ///
    /// * `form` - The form's name in the resource file.
    /// * `contents` - The stub's source text.
    ///
    /// # Returns
    ///
    /// The path of the new file, relative to the project.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Ownership`] if the project has no such form or the file already exists (the
    /// developer's code is never overwritten), and [`ProjectError::Io`] if the write fails.
    pub fn create_stub(&mut self, form: &str, contents: &str) -> Result<PathBuf, ProjectError> {
        let rel = self
            .forms
            .iter()
            .find(|f| f.name == form)
            .map(|f| f.handler_file.clone())
            .ok_or_else(|| Self::refuse(Path::new(HANDLERS_DIR), "the project has no such form"))?;
        let path = self.root.join(&rel);

        if path.exists() {
            return Err(Self::refuse(
                &rel,
                "the handler file exists and belongs to the developer",
            ));
        }

        self.write(&rel, contents.as_bytes())?;
        self.reload_handlers()?;

        Ok(rel)
    }

    /// Saves the resource file and reloads the model from it.
    ///
    /// # Arguments
    ///
    /// * `file` - The new contents, written with Pushrod's writer.
    ///
    /// # Returns
    ///
    /// `Ok(())` once saved.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Ownership`] if the project has no resource file (Chassis creates it
    /// through [`Project::create_resource_file`]), and [`ProjectError::Resources`] if `file` is
    /// invalid or cannot be written.
    pub fn save_resources(&mut self, file: FormFile) -> Result<(), ProjectError> {
        let path = self.resource_path.clone().ok_or_else(|| {
            Self::refuse(Path::new("*.prsc"), "the project has no resource file yet")
        })?;

        self.store_resources(path, file)
    }

    /// Creates the resource file of a project that has none.
    ///
    /// # Arguments
    ///
    /// * `name` - The file name, ending in `.prsc`, directly in the project directory.
    /// * `file` - The contents.
    ///
    /// # Returns
    ///
    /// `Ok(())` once created.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Ownership`] if the project already has a resource file or `name` is not a
    /// plain `.prsc` file name, and [`ProjectError::Resources`] if the write fails.
    pub fn create_resource_file(&mut self, name: &str, file: FormFile) -> Result<(), ProjectError> {
        let rel = Path::new(name);
        let plain = rel.components().count() == 1 && is_contained(rel);

        if self.resource_path.is_some() {
            return Err(Self::refuse(rel, "the project already has a resource file"));
        }

        if !plain || rel.extension().is_none_or(|x| x != "prsc") {
            return Err(Self::refuse(
                rel,
                "a resource file is a *.prsc file in the project directory",
            ));
        }

        self.store_resources(self.root.join(rel), file)
    }

    /// Writes a resource file and reloads everything derived from it.
    ///
    /// # Arguments
    ///
    /// * `path` - Where to write.
    /// * `file` - The contents.
    ///
    /// # Returns
    ///
    /// `Ok(())` once written and reloaded.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Resources`] if `file` is invalid or cannot be written.
    fn store_resources(&mut self, path: PathBuf, file: FormFile) -> Result<(), ProjectError> {
        let fail = |message: String| ProjectError::Resources {
            path: path.clone(),
            message,
        };

        file.validate().map_err(|e| fail(e.to_string()))?;

        let bytes = file.to_bytes().map_err(|e| fail(e.to_string()))?;

        std::fs::write(&path, bytes).map_err(|e| fail(e.to_string()))?;

        self.forms = Self::forms_of(&self.root, Some(&file));
        self.resource_path = Some(path);
        self.resources = Some(file);

        self.reload_handlers()
    }

    /// Reads the handler files again.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the model matches the disk.
    ///
    /// # Errors
    ///
    /// [`ProjectError`] if a handler file cannot be read or does not parse.
    pub fn reload_handlers(&mut self) -> Result<(), ProjectError> {
        self.forms = Self::forms_of(&self.root, self.resources.as_ref());
        self.handlers = Self::read_handlers(&self.root, &self.forms)?;

        Ok(())
    }

    /// Writes bytes to a project-relative path, creating folders.
    ///
    /// # Arguments
    ///
    /// * `rel` - The path relative to the project; the caller has checked ownership.
    /// * `bytes` - The contents.
    ///
    /// # Returns
    ///
    /// `Ok(())` once written.
    ///
    /// # Errors
    ///
    /// [`ProjectError::Io`] if a folder or the file cannot be written.
    pub(crate) fn write(&self, rel: &Path, bytes: &[u8]) -> Result<(), ProjectError> {
        let path = self.root.join(rel);
        let io = |message: String| ProjectError::Io {
            path: path.clone(),
            message,
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(e.to_string()))?;
        }

        std::fs::write(&path, bytes).map_err(|e| io(e.to_string()))
    }
}
