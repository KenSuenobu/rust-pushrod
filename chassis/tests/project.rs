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

//! Tests of the project model: loading, handlers, and ownership.  No window is involved.

use std::fs;
use std::path::Path;

use chassis::project::{GENERATED_DIR, Ownership, Project, ProjectError, form_module};
use pushrod::form::{FormFile, ObjectRecord};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"1.2.3\"\nedition = \"2024\"\n";

/// Builds a resource file with the named forms and one string.
fn resource_file(forms: &[&str]) -> FormFile {
    let mut file = FormFile::new();

    for name in forms {
        let root = file.add_object(None, ObjectRecord::new("Frame"));

        file.add_form(name, root);
    }

    file.resources
        .strings
        .add("GREETING", "Hello".to_string())
        .unwrap();
    file
}

/// Creates a project directory with a manifest and, optionally, a resource file.
fn project_dir(forms: Option<&[&str]>) -> TempDir {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("Cargo.toml"), MANIFEST).unwrap();

    if let Some(forms) = forms {
        fs::write(
            dir.path().join("app.prsc"),
            resource_file(forms).to_bytes().unwrap(),
        )
        .unwrap();
    }

    dir
}

/// Writes a project-relative file.
fn put(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn loads_manifest_forms_and_resource_counts() {
    let dir = project_dir(Some(&["MainWindow", "About"]));
    let project = Project::load(dir.path()).unwrap();

    assert_eq!(project.package().name, "demo");
    assert_eq!(project.package().version, "1.2.3");
    assert_eq!(
        project.resource_path().unwrap(),
        dir.path().join("app.prsc")
    );

    let names: Vec<_> = project.forms().iter().map(|f| f.name.as_str()).collect();

    assert_eq!(names, ["MainWindow", "About"]);
    assert_eq!(
        project.forms()[0].handler_file,
        Path::new("src/forms/main_window.rs")
    );
    assert!(!project.forms()[0].has_handler_file);
    assert_eq!(project.resource_counts().strings, 1);
    assert_eq!(project.resource_counts().images, 0);
}

#[test]
fn a_project_without_a_resource_file_has_no_forms() {
    let dir = project_dir(None);
    let project = Project::load(dir.path()).unwrap();

    assert!(project.resource_file().is_none());
    assert!(project.forms().is_empty());
    assert_eq!(project.resource_counts().strings, 0);
}

#[test]
fn the_manifest_can_name_the_resource_file() {
    let dir = project_dir(Some(&["A"]));

    fs::rename(dir.path().join("app.prsc"), dir.path().join("other.prsc")).unwrap();
    fs::write(
        dir.path().join("second.prsc"),
        resource_file(&["B"]).to_bytes().unwrap(),
    )
    .unwrap();
    assert!(Project::load(dir.path()).is_err());

    let named = format!("{MANIFEST}[package.metadata.chassis]\nresources = \"second.prsc\"\n");

    fs::write(dir.path().join("Cargo.toml"), named).unwrap();

    let project = Project::load(dir.path()).unwrap();

    assert_eq!(project.forms()[0].name, "B");
}

#[test]
fn a_named_resource_file_must_exist_and_stay_inside() {
    let dir = project_dir(None);

    for bad in ["missing.prsc", "../outside.prsc"] {
        let text = format!("{MANIFEST}[package.metadata.chassis]\nresources = \"{bad}\"\n");

        fs::write(dir.path().join("Cargo.toml"), text).unwrap();
        assert!(Project::load(dir.path()).is_err(), "{bad}");
    }
}

#[test]
fn bad_manifests_and_resource_files_are_errors() {
    let dir = project_dir(None);

    fs::write(dir.path().join("Cargo.toml"), "not toml [").unwrap();
    assert!(matches!(
        Project::load(dir.path()),
        Err(ProjectError::Manifest { .. })
    ));

    fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    assert!(matches!(
        Project::load(dir.path()),
        Err(ProjectError::Manifest { .. })
    ));

    fs::remove_file(dir.path().join("Cargo.toml")).unwrap();
    assert!(matches!(
        Project::load(dir.path()),
        Err(ProjectError::Io { .. })
    ));

    let dir = project_dir(None);

    fs::write(dir.path().join("app.prsc"), b"garbage").unwrap();
    assert!(matches!(
        Project::load(dir.path()),
        Err(ProjectError::Resources { .. })
    ));
}

#[test]
fn finds_handlers_in_the_code_on_disk() {
    let dir = project_dir(Some(&["MainWindow"]));

    put(
        dir.path(),
        "src/forms/main_window.rs",
        "use std::fmt;\n\npub fn ok_clicked(ctx: &mut Context) {\n    fn inner() {}\n}\n\n         fn helper(a: u32,\n    b: u32) -> u32 { a + b }\n\nimpl Foo { fn method(&self) {} }\n",
    );
    put(dir.path(), "src/forms/stray.rs", "pub fn lonely() {}\n");
    put(dir.path(), "src/forms/mod.rs", "pub mod main_window;\n");

    let project = Project::load(dir.path()).unwrap();
    let mine: Vec<_> = project.handlers_for("MainWindow").collect();

    assert_eq!(mine.len(), 2);
    assert_eq!(mine[0].name, "ok_clicked");
    assert_eq!(mine[0].signature, "pub fn ok_clicked(ctx: &mut Context)");
    assert_eq!(mine[0].line, 3);
    assert_eq!(mine[0].file, Path::new("src/forms/main_window.rs"));
    assert_eq!(mine[1].name, "helper");
    assert_eq!(mine[1].line, 7);
    assert!(mine[1].signature.starts_with("fn helper(a: u32,"));
    assert!(mine[1].signature.ends_with("-> u32"));
    assert!(project.forms()[0].has_handler_file);

    let stray = project
        .handlers()
        .iter()
        .find(|h| h.name == "lonely")
        .unwrap();

    assert_eq!(stray.form, None);
    assert_eq!(project.handlers().len(), 3);
}

#[test]
fn the_model_follows_edits_on_disk() {
    let dir = project_dir(Some(&["MainWindow"]));

    put(dir.path(), "src/forms/main_window.rs", "pub fn one() {}\n");

    let mut project = Project::load(dir.path()).unwrap();

    assert_eq!(project.handlers().len(), 1);
    put(
        dir.path(),
        "src/forms/main_window.rs",
        "pub fn one() {}\npub fn two() {}\n",
    );
    project.reload_handlers().unwrap();
    assert_eq!(project.handlers().len(), 2);
}

#[test]
fn a_handler_file_with_syntax_errors_is_reported() {
    let dir = project_dir(Some(&["MainWindow"]));

    put(dir.path(), "src/forms/main_window.rs", "pub fn broken( {\n");
    assert!(matches!(
        Project::load(dir.path()),
        Err(ProjectError::Handlers { .. })
    ));
}

#[test]
fn classifies_ownership() {
    let dir = project_dir(Some(&["MainWindow"]));
    let project = Project::load(dir.path()).unwrap();

    assert_eq!(project.ownership("app.prsc"), Ownership::Resource);
    assert_eq!(project.ownership("./app.prsc"), Ownership::Resource);
    assert_eq!(
        project.ownership("src/generated/ids.rs"),
        Ownership::Generated
    );
    assert_eq!(
        project.ownership("src/forms/main_window.rs"),
        Ownership::Handler
    );
    assert_eq!(project.ownership("Cargo.toml"), Ownership::User);
    assert_eq!(project.ownership("src/main.rs"), Ownership::User);
    assert_eq!(
        project.ownership("src/generated/../main.rs"),
        Ownership::User
    );
    assert_eq!(project.ownership("/etc/passwd"), Ownership::User);
}

#[test]
fn generated_files_are_written_and_replaced_freely() {
    let dir = project_dir(None);
    let project = Project::load(dir.path()).unwrap();
    let rel = Path::new(GENERATED_DIR).join("ids.rs");

    project.write_generated(&rel, "// one\n").unwrap();
    project.write_generated(&rel, "// two\n").unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join(&rel)).unwrap(),
        "// two\n"
    );
}

#[test]
fn chassis_refuses_to_write_files_it_does_not_own() {
    let dir = project_dir(Some(&["MainWindow"]));
    let project = Project::load(dir.path()).unwrap();

    put(dir.path(), "src/main.rs", "fn main() {}\n");

    for rel in [
        "src/main.rs",
        "Cargo.toml",
        "src/forms/main_window.rs",
        "src/generated/../main.rs",
        "../escape.rs",
        "/tmp/absolute.rs",
    ] {
        let err = project.write_generated(rel, "overwritten").unwrap_err();

        assert!(matches!(err, ProjectError::Ownership { .. }), "{rel}");
    }

    assert_eq!(
        fs::read_to_string(dir.path().join("src/main.rs")).unwrap(),
        "fn main() {}\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("Cargo.toml")).unwrap(),
        MANIFEST
    );
}

#[test]
fn a_stub_is_created_once_and_never_overwritten() {
    let dir = project_dir(Some(&["MainWindow"]));
    let mut project = Project::load(dir.path()).unwrap();
    let rel = project
        .create_stub("MainWindow", "pub fn ok_clicked() {}\n")
        .unwrap();

    assert_eq!(rel, Path::new("src/forms/main_window.rs"));
    assert_eq!(project.handlers().len(), 1);
    assert!(project.forms()[0].has_handler_file);

    put(dir.path(), "src/forms/main_window.rs", "pub fn mine() {}\n");

    let err = project
        .create_stub("MainWindow", "pub fn clobber() {}\n")
        .unwrap_err();

    assert!(matches!(err, ProjectError::Ownership { .. }));
    assert_eq!(
        fs::read_to_string(dir.path().join(&rel)).unwrap(),
        "pub fn mine() {}\n"
    );
    assert!(matches!(
        project.create_stub("NoSuchForm", ""),
        Err(ProjectError::Ownership { .. })
    ));
}

#[test]
fn the_resource_file_is_saved_and_the_model_reloaded() {
    let dir = project_dir(Some(&["MainWindow"]));
    let mut project = Project::load(dir.path()).unwrap();

    project
        .save_resources(resource_file(&["MainWindow", "Settings"]))
        .unwrap();
    assert_eq!(project.forms().len(), 2);

    let again = Project::load(dir.path()).unwrap();

    assert_eq!(again.forms()[1].name, "Settings");

    let mut bad = resource_file(&["A"]);

    bad.forms[0].root = 999;
    assert!(matches!(
        project.save_resources(bad),
        Err(ProjectError::Resources { .. })
    ));
    assert_eq!(project.forms().len(), 2);
}

#[test]
fn a_resource_file_can_be_created_once() {
    let dir = project_dir(None);
    let mut project = Project::load(dir.path()).unwrap();

    assert!(matches!(
        project.save_resources(resource_file(&["A"])),
        Err(ProjectError::Ownership { .. })
    ));

    for bad in ["app.txt", "sub/app.prsc", "../app.prsc"] {
        assert!(
            matches!(
                project.create_resource_file(bad, resource_file(&["A"])),
                Err(ProjectError::Ownership { .. })
            ),
            "{bad}"
        );
    }

    project
        .create_resource_file("app.prsc", resource_file(&["A"]))
        .unwrap();
    assert_eq!(project.ownership("app.prsc"), Ownership::Resource);
    assert!(
        project
            .create_resource_file("again.prsc", resource_file(&["A"]))
            .is_err()
    );
}

#[test]
fn form_names_become_module_names() {
    assert_eq!(form_module("MainWindow"), "main_window");
    assert_eq!(form_module("About"), "about");
    assert_eq!(form_module("Open File dialog"), "open_file_dialog");
    assert_eq!(form_module("HTTPServer2"), "httpserver2");
    assert_eq!(form_module("my-form"), "my_form");
}
