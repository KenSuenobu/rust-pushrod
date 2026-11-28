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

//! Tests of the code generation rules: Chassis never edits a function body, and writes only what
//! it owns.  No window is involved.

use std::fs;
use std::path::Path;

use chassis::codegen::{
    MAIN_FILE, create_main, format_source, insert_handler_stub, rename_handler, write_generated,
};
use chassis::{Project, ProjectError};
use pushrod::form::{FormFile, ObjectRecord};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"1.0.0\"\nedition = \"2024\"\n";

/// The developer's handler file: comments, a helper, and bodies that must survive.
const HANDLERS: &str = "// My notes - keep.\nuse std::fmt;\n\npub fn on_save(count: u32) -> u32 {\n    // tricky   spacing\n    count   +  1\n}\n\nfn helper() { println!(\"hi\"); }\n";

/// Creates a project with one form, `MainWindow`, and its handler file.
fn project() -> (TempDir, Project) {
    let dir = TempDir::new().unwrap();
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("Frame"));

    file.add_form("MainWindow", root);
    fs::write(dir.path().join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(dir.path().join("app.prsc"), file.to_bytes().unwrap()).unwrap();
    fs::create_dir_all(dir.path().join("src/forms")).unwrap();
    fs::write(dir.path().join("src/forms/main_window.rs"), HANDLERS).unwrap();

    let project = Project::load(dir.path()).unwrap();

    (dir, project)
}

/// Reads a project-relative file.
fn read(dir: &Path, rel: &str) -> String {
    fs::read_to_string(dir.join(rel)).unwrap()
}

#[test]
fn generated_output_is_formatted() {
    let (dir, project) = project();

    write_generated(
        &project,
        "src/generated/ids.rs",
        "pub mod a{pub const X:u32=1;}",
    )
    .unwrap();

    assert_eq!(
        read(dir.path(), "src/generated/ids.rs"),
        "pub mod a {\n    pub const X: u32 = 1;\n}\n"
    );
}

#[test]
fn generated_files_are_replaced_freely() {
    let (dir, project) = project();

    write_generated(&project, "src/generated/ids.rs", "const A: u8 = 1;").unwrap();
    write_generated(&project, "src/generated/ids.rs", "const B: u8 = 2;").unwrap();

    assert_eq!(
        read(dir.path(), "src/generated/ids.rs"),
        "const B: u8 = 2;\n"
    );
}

#[test]
fn writes_outside_generated_are_refused() {
    let (dir, project) = project();

    for rel in [
        "src/forms/main_window.rs",
        "src/main.rs",
        "Cargo.toml",
        "src/generated/../main.rs",
    ] {
        let err = write_generated(&project, rel, "fn x() {}").unwrap_err();

        assert!(matches!(err, ProjectError::Ownership { .. }), "{rel}");
    }

    assert_eq!(read(dir.path(), "src/forms/main_window.rs"), HANDLERS);
    assert!(!dir.path().join("src/main.rs").exists());
}

#[test]
fn bad_source_is_not_written() {
    let (dir, project) = project();

    assert!(write_generated(&project, "src/generated/bad.rs", "fn {").is_err());
    assert!(!dir.path().join("src/generated/bad.rs").exists());
    assert!(format_source("fn {").is_err());
}

#[test]
fn main_is_created_once() {
    let (dir, project) = project();

    assert_eq!(
        create_main(&project, "fn main(){}").unwrap(),
        Path::new(MAIN_FILE)
    );
    assert_eq!(read(dir.path(), MAIN_FILE), "fn main() {}\n");

    fs::write(dir.path().join(MAIN_FILE), "fn main() { /* mine */ }").unwrap();

    let err = create_main(&project, "fn main(){}").unwrap_err();

    assert!(matches!(err, ProjectError::Ownership { .. }));
    assert_eq!(read(dir.path(), MAIN_FILE), "fn main() { /* mine */ }");
}

#[test]
fn stub_is_appended_and_everything_else_survives() {
    let (dir, mut project) = project();
    let name = insert_handler_stub(&mut project, "MainWindow", "pub fn on_close() {}").unwrap();
    let text = read(dir.path(), "src/forms/main_window.rs");

    assert_eq!(name, "on_close");
    assert!(text.starts_with(HANDLERS));
    assert!(text.ends_with("pub fn on_close() {}\n"));

    let names: Vec<_> = project.handlers().iter().map(|h| h.name.as_str()).collect();

    assert_eq!(names, ["on_save", "helper", "on_close"]);
}

#[test]
fn existing_handler_is_never_replaced_by_a_stub() {
    let (dir, mut project) = project();
    let err = insert_handler_stub(&mut project, "MainWindow", "pub fn on_save() {}").unwrap_err();

    assert!(matches!(err, ProjectError::Ownership { .. }));
    assert_eq!(read(dir.path(), "src/forms/main_window.rs"), HANDLERS);
}

#[test]
fn stub_must_be_one_function() {
    let (dir, mut project) = project();

    for stub in ["", "fn a() {} fn b() {}", "fn broken("] {
        assert!(insert_handler_stub(&mut project, "MainWindow", stub).is_err());
    }

    assert_eq!(read(dir.path(), "src/forms/main_window.rs"), HANDLERS);
}

#[test]
fn unknown_form_or_missing_file_is_refused() {
    let (dir, mut project) = project();

    assert!(insert_handler_stub(&mut project, "Nope", "fn a() {}").is_err());

    fs::remove_file(dir.path().join("src/forms/main_window.rs")).unwrap();

    assert!(insert_handler_stub(&mut project, "MainWindow", "fn a() {}").is_err());
    assert!(!dir.path().join("src/forms/main_window.rs").exists());
}

#[test]
fn rename_changes_only_the_name() {
    let (dir, mut project) = project();

    rename_handler(&mut project, "MainWindow", "on_save", "on_save_file").unwrap();

    let text = read(dir.path(), "src/forms/main_window.rs");

    assert_eq!(text, HANDLERS.replace("fn on_save(", "fn on_save_file("));
    assert!(text.contains("// tricky   spacing\n    count   +  1\n"));
    assert!(project.handlers().iter().any(|h| h.name == "on_save_file"));
    assert!(project.handlers().iter().all(|h| h.name != "on_save"));
}

#[test]
fn rename_refuses_bad_requests_without_writing() {
    let (dir, mut project) = project();

    for (old, new) in [
        ("missing", "x"),
        ("on_save", "helper"),
        ("on_save", ""),
        ("on_save", "1abc"),
        ("on_save", "has space"),
        ("on_save", "fn"),
        ("on_save", "_"),
    ] {
        assert!(
            rename_handler(&mut project, "MainWindow", old, new).is_err(),
            "{old} -> {new}"
        );
    }

    assert_eq!(read(dir.path(), "src/forms/main_window.rs"), HANDLERS);
}

#[test]
fn bodies_survive_every_designer_operation() {
    let (dir, mut project) = project();
    let bodies = [
        "    // tricky   spacing\n    count   +  1\n",
        "{ println!(\"hi\"); }",
    ];

    insert_handler_stub(&mut project, "MainWindow", "pub fn on_open() {}").unwrap();
    rename_handler(&mut project, "MainWindow", "on_save", "on_store").unwrap();
    write_generated(&project, "src/generated/wiring.rs", "pub fn wire(){}").unwrap();
    create_main(&project, "fn main(){}").unwrap();
    rename_handler(&mut project, "MainWindow", "on_open", "on_show").unwrap();
    insert_handler_stub(&mut project, "MainWindow", "pub fn on_hide() {}").unwrap();
    rename_handler(&mut project, "MainWindow", "on_store", "on_save").unwrap();
    project.reload_handlers().unwrap();

    let text = read(dir.path(), "src/forms/main_window.rs");

    for body in bodies {
        assert!(text.contains(body), "lost: {body}");
    }

    assert!(text.starts_with("// My notes - keep.\nuse std::fmt;\n"));
    assert!(text.contains("pub fn on_save(count: u32) -> u32 {"));
}
