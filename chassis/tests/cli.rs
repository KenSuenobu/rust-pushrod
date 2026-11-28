//! Tests of the command-line generator: `generate` writes what the designer writes, and `check`
//! reports missing, orphaned, and invalid things.  No window is involved.

use std::fs;
use std::path::Path;

use chassis::Project;
use chassis::cli::{Problem, ProblemKind, check, generate, render, run};
use pushrod::form::{FormFile, ObjectRecord, SymbolKind};
use pushrod::registry::WidgetRegistry;
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"1.0.0\"\nedition = \"2024\"\n";

/// The committed handler file of the wiring test.
const HANDLERS: &str = include_str!("wired/forms/main_window.rs");

/// The registry of every Pushrod widget.
fn registry() -> WidgetRegistry {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    registry
}

/// A form with an OK button and an Accept check box; `ok_name` names the button's ID.
fn form_file(ok_name: &str) -> FormFile {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    let ok = file.add_object(Some(root), ObjectRecord::new("pushrod.button"));
    let accept = file.add_object(Some(root), ObjectRecord::new("pushrod.check_box"));

    file.add_form("MainWindow", root);
    file.add_symbol(SymbolKind::Id, ok, ok_name);
    file.add_symbol(SymbolKind::Id, accept, "ACCEPT");

    file
}

/// Creates a project directory with the handlers and form file given.
fn make(handlers: &str, file: &FormFile) -> TempDir {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(dir.path().join("app.prsc"), file.to_bytes().unwrap()).unwrap();
    fs::create_dir_all(dir.path().join("src/forms")).unwrap();
    fs::write(dir.path().join("src/forms/main_window.rs"), handlers).unwrap();

    dir
}

/// Checks the project in `dir`.
fn problems(dir: &Path) -> Vec<Problem> {
    check(&Project::load(dir).unwrap(), &registry())
}

/// Runs the command line, returning the exit code, output, and errors.
fn cli(args: &[&str]) -> (i32, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(args, &registry(), &mut out, &mut err);

    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn generate_writes_what_render_produces_and_what_is_committed() {
    let dir = make(HANDLERS, &form_file("OK_BUTTON"));
    let project = Project::load(dir.path()).unwrap();
    let written = generate(&project, &registry()).unwrap();
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/wired/generated");

    assert_eq!(written.len(), 3);

    for (rel, text) in render(&project, &registry()).unwrap() {
        assert_eq!(fs::read_to_string(dir.path().join(rel)).unwrap(), text);
    }

    for name in ["ids.rs", "main_window_wiring.rs"] {
        assert_eq!(
            fs::read_to_string(dir.path().join("src/generated").join(name)).unwrap(),
            fs::read_to_string(base.join(name)).unwrap(),
            "{name}"
        );
    }

    let list = fs::read_to_string(dir.path().join("src/generated/mod.rs")).unwrap();

    assert!(list.contains("pub mod ids;\npub mod main_window_wiring;\n"));
}

#[test]
fn generate_is_repeatable_and_leaves_the_developers_files_alone() {
    let dir = make(HANDLERS, &form_file("OK_BUTTON"));
    let project = Project::load(dir.path()).unwrap();

    generate(&project, &registry()).unwrap();

    let first = fs::read_to_string(dir.path().join("src/generated/ids.rs")).unwrap();

    generate(&project, &registry()).unwrap();

    assert_eq!(
        fs::read_to_string(dir.path().join("src/generated/ids.rs")).unwrap(),
        first
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("src/forms/main_window.rs")).unwrap(),
        HANDLERS
    );
}

#[test]
fn a_generated_project_checks_clean() {
    let dir = make(HANDLERS, &form_file("OK_BUTTON"));

    generate(&Project::load(dir.path()).unwrap(), &registry()).unwrap();

    assert_eq!(problems(dir.path()), []);
}

#[test]
fn missing_and_stale_generated_files_are_reported() {
    let dir = make(HANDLERS, &form_file("OK_BUTTON"));
    let found = problems(dir.path());

    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|p| p.kind == ProblemKind::Missing));

    generate(&Project::load(dir.path()).unwrap(), &registry()).unwrap();
    fs::write(
        dir.path().join("src/generated/main_window_wiring.rs"),
        "// old\n",
    )
    .unwrap();

    let found = problems(dir.path());

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, ProblemKind::Missing);
    assert!(found[0].to_string().contains("main_window_wiring.rs"));
}

#[test]
fn orphaned_handlers_are_reported() {
    let handlers = format!("{HANDLERS}\npub fn on_gone_button_clicked() {{}}\n");
    let dir = make(&handlers, &form_file("OK_BUTTON"));

    generate(&Project::load(dir.path()).unwrap(), &registry()).unwrap();
    fs::write(dir.path().join("src/forms/other.rs"), "pub fn x() {}\n").unwrap();

    let found = problems(dir.path());

    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|p| p.kind == ProblemKind::Orphaned));
    assert!(found[0].to_string().contains("other.rs"));
    assert!(found[1].to_string().contains("on_gone_button_clicked"));
}

#[test]
fn invalid_names_and_handlers_are_reported() {
    let dir = make(HANDLERS, &form_file("9LIVES"));
    let found = problems(dir.path());

    assert!(found.iter().any(|p| p.kind == ProblemKind::Invalid));
    assert!(generate(&Project::load(dir.path()).unwrap(), &registry()).is_err());
    assert!(!dir.path().join("src/generated").exists());

    let dir = make("fn on_ok_button_clicked() {}\n", &form_file("OK_BUTTON"));
    let found = problems(dir.path());

    assert!(
        found
            .iter()
            .any(|p| p.kind == ProblemKind::Invalid && p.to_string().contains("must be `pub`"))
    );
}

#[test]
fn a_project_without_a_resource_file_is_invalid() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("Cargo.toml"), MANIFEST).unwrap();

    let found = problems(dir.path());

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, ProblemKind::Invalid);
}

#[test]
fn the_command_line_exits_non_zero_on_problems() {
    let dir = make(HANDLERS, &form_file("OK_BUTTON"));
    let path = dir.path().to_str().unwrap();
    let (code, _, err) = cli(&["chassis", "check", path]);

    assert_eq!(code, 1);
    assert!(err.contains("missing:") && err.contains("3 problem(s)"));

    let (code, out, _) = cli(&["chassis", "generate", path]);

    assert_eq!(code, 0);
    assert!(out.contains("wrote src/generated/ids.rs"));

    let (code, out, err) = cli(&["chassis", "check", path]);

    assert_eq!((code, out.as_str(), err.as_str()), (0, "ok\n", ""));
}

#[test]
fn bad_arguments_and_unloadable_projects_fail_with_code_two() {
    assert_eq!(cli(&["chassis"]).0, 2);
    assert_eq!(cli(&["chassis", "frobnicate"]).0, 2);
    assert_eq!(cli(&["chassis", "check", "/no/such/project"]).0, 2);
    assert_eq!(cli(&["chassis", "generate", "/no/such/project"]).0, 2);

    let (code, out, _) = cli(&["chassis", "--help"]);

    assert_eq!(code, 0);
    assert!(out.contains("generate") && out.contains("check"));
}
