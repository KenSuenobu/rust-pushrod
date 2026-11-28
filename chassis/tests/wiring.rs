//! Tests of event wiring generation: handlers are matched to widgets and events, the output is
//! deterministic, and the generated code registers handlers that fire.  No window is involved.

use std::fs;
use std::path::Path;

use chassis::codegen::format_source;
use chassis::codegen::wiring::{generate_wiring, match_handlers, wiring_path, write_wiring};
use chassis::{Project, ProjectError};
use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::form::codegen::generate_ids;
use pushrod::form::{FormFile, FormValue, ObjectRecord, Property, SymbolKind};
use pushrod::geometry::size;
use pushrod::input::{InputEvent, MouseButton};
use pushrod::registry::WidgetRegistry;
use tempfile::TempDir;

/// The generated code under test, as committed.  `CHASSIS_BLESS=1 cargo test -p chassis --test
/// wiring` rewrites its two files from the generator.
#[path = "wired/generated/mod.rs"]
mod generated;

/// The developer's handlers, as committed.
#[path = "wired/forms/mod.rs"]
mod forms;

/// The application state the handlers change.
#[derive(Default)]
pub struct AppState {
    /// How many times OK was clicked.
    pub clicks: u32,
    /// Whether Accept is checked.
    pub accepted: bool,
}

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"1.0.0\"\nedition = \"2024\"\n";

/// The committed handler file.
const HANDLERS: &str = include_str!("wired/forms/main_window.rs");

/// A registry holding the standard widgets.
fn registry() -> WidgetRegistry {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    registry
}

/// Builds the `MainWindow` form: a panel holding an OK button and an Accept check box.
fn form_file() -> FormFile {
    let mut file = FormFile::new();
    let mut panel = ObjectRecord::new("pushrod.panel");

    panel.width = 200;
    panel.height = 80;
    let root = file.add_object(None, panel);

    let mut ok = ObjectRecord::new("pushrod.button");

    ok.x = 10;
    ok.y = 10;
    ok.width = 60;
    ok.height = 24;
    ok.properties
        .push(Property::named("text", FormValue::Text("OK".into())));
    let ok = file.add_object(Some(root), ok);

    let mut accept = ObjectRecord::new("pushrod.check_box");

    accept.x = 10;
    accept.y = 40;
    accept.width = 100;
    accept.height = 24;
    accept
        .properties
        .push(Property::named("text", FormValue::Text("Accept".into())));
    let accept = file.add_object(Some(root), accept);

    file.add_form("MainWindow", root);
    file.add_symbol(SymbolKind::Id, ok, "OK_BUTTON");
    file.add_symbol(SymbolKind::Id, accept, "ACCEPT");

    file
}

/// Creates a project whose `MainWindow` handler file holds `handlers`.
fn project_with(handlers: &str) -> (TempDir, Project) {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(dir.path().join("app.prsc"), form_file().to_bytes().unwrap()).unwrap();
    fs::create_dir_all(dir.path().join("src/forms")).unwrap();
    fs::write(dir.path().join("src/forms/main_window.rs"), handlers).unwrap();

    let project = Project::load(dir.path()).unwrap();

    (dir, project)
}

/// Reads a project-relative file.
fn read(dir: &Path, rel: &Path) -> String {
    fs::read_to_string(dir.join(rel)).unwrap()
}

#[test]
fn handlers_are_matched_to_widgets_and_events() {
    let (_dir, project) = project_with(HANDLERS);
    let found = match_handlers(&project, &registry(), "MainWindow").unwrap();
    let names: Vec<_> = found
        .iter()
        .map(|w| {
            (
                w.handler.as_str(),
                w.widget.as_str(),
                w.event.as_str(),
                w.callback.as_str(),
            )
        })
        .collect();

    assert_eq!(
        names,
        [
            ("on_accept_toggled", "ACCEPT", "Toggled", "on_toggle"),
            ("on_ok_button_clicked", "OK_BUTTON", "Clicked", "on_click"),
        ]
    );
}

#[test]
fn output_is_deterministic() {
    let (_dir, project) = project_with(HANDLERS);
    let first = generate_wiring(&project, &registry(), "MainWindow").unwrap();

    for _ in 0..3 {
        assert_eq!(
            generate_wiring(&project, &registry(), "MainWindow").unwrap(),
            first
        );
    }

    // The same handlers in another order give the same file.
    let reordered = "use crate::AppState;\n".to_string()
        + "pub fn on_accept_toggled() {}\npub fn on_ok_button_clicked() {}\n";
    let (_dir2, other) = project_with(&reordered);

    assert_eq!(
        generate_wiring(&other, &registry(), "MainWindow").unwrap(),
        first
    );
}

#[test]
fn written_file_is_formatted_and_has_the_do_not_edit_header() {
    let (dir, project) = project_with(HANDLERS);
    let rel = write_wiring(&project, &registry(), "MainWindow").unwrap();

    assert_eq!(rel, wiring_path("MainWindow"));
    assert_eq!(rel, Path::new("src/generated/main_window_wiring.rs"));

    let text = read(dir.path(), &rel);

    assert!(text.starts_with("// Generated by Chassis"));
    assert!(text.contains("Do not edit"));
    assert!(text.contains("engine.on_click(id, on_ok_button_clicked);"));
    assert!(text.contains("form.widget(ids::main_window::OK_BUTTON)"));
    assert_eq!(format_source(&text).unwrap(), text);
}

#[test]
fn generated_files_match_the_committed_ones() {
    let (dir, project) = project_with(HANDLERS);
    let wiring = {
        write_wiring(&project, &registry(), "MainWindow").unwrap();
        read(dir.path(), &wiring_path("MainWindow"))
    };
    let ids = format_source(&generate_ids(&form_file()).unwrap()).unwrap();
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/wired/generated");

    if std::env::var_os("CHASSIS_BLESS").is_some() {
        fs::write(base.join("main_window_wiring.rs"), &wiring).unwrap();
        fs::write(base.join("ids.rs"), &ids).unwrap();
    }

    assert_eq!(
        fs::read_to_string(base.join("main_window_wiring.rs")).unwrap(),
        wiring
    );
    assert_eq!(fs::read_to_string(base.join("ids.rs")).unwrap(), ids);
}

/// A left mouse button event at a point.
fn mouse(down: bool, x: i32, y: i32) -> InputEvent {
    let position = pushrod::geometry::point(x, y);

    if down {
        InputEvent::MouseDown {
            position,
            button: MouseButton::Left,
            clicks: 1,
        }
    } else {
        InputEvent::MouseUp {
            position,
            button: MouseButton::Left,
            clicks: 1,
        }
    }
}

#[test]
fn every_wired_handler_fires() {
    let mut engine: Engine<AppState> = Engine::new(size(320, 200), 30);

    pushrod_widgets::register(engine.widget_registry());

    let form = engine.load_form(&form_file(), "MainWindow").unwrap();

    generated::main_window_wiring::wire(&mut engine, &form);

    // Draw a frame first, as a running program does, so every widget knows where it is.
    let mut backend = HeadlessBackend::new(size(320, 200));

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);

    let mut state = AppState::default();

    // The OK button is at (10, 10), 60 by 24; the check box at (10, 40).
    engine.dispatch(mouse(true, 20, 20), &mut state);
    engine.dispatch(mouse(false, 20, 20), &mut state);
    assert_eq!(state.clicks, 1);
    assert_eq!(forms::main_window::format_clicks(&state), "1 clicks");

    engine.dispatch(mouse(true, 20, 50), &mut state);
    engine.dispatch(mouse(false, 20, 50), &mut state);
    assert!(state.accepted);
    assert_eq!(state.clicks, 1);

    // The typed ID constants name the objects the handlers are wired to.
    assert_eq!(generated::ids::main_window::OK_BUTTON.0, 1);
}

#[test]
fn a_form_without_handlers_gets_an_empty_wire() {
    let (_dir, project) = project_with("pub fn helper() {}\n");
    let text = generate_wiring(&project, &registry(), "MainWindow").unwrap();

    assert!(text.contains("pub fn wire(_engine: &mut Engine<AppState>, _form: &FormHandle) {}"));
    assert!(!text.contains("use super::ids;"));
}

#[test]
fn problems_are_refused_with_the_handler_named() {
    let cases = [
        ("fn on_ok_button_clicked() {}\n", "must be `pub`"),
        (
            "pub fn on_ok_button_focus_gained() {}\npub fn on_ok_button_focus_lost() {}\n",
            "both use `Engine::on_focus_change`",
        ),
        (
            "pub fn on_ok_button_key_released() {}\n",
            "no registration method",
        ),
    ];

    for (source, expected) in cases {
        let (_dir, project) = project_with(source);
        match generate_wiring(&project, &registry(), "MainWindow") {
            Err(ProjectError::Handlers { message, .. }) => {
                assert!(message.contains(expected), "{message}")
            }
            other => panic!("expected `{expected}`, got {other:?}"),
        }
    }
}

#[test]
fn unknown_forms_and_projects_without_resources_are_refused() {
    let (_dir, project) = project_with(HANDLERS);

    assert!(matches!(
        generate_wiring(&project, &registry(), "Missing"),
        Err(ProjectError::Handlers { .. })
    ));

    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("Cargo.toml"), MANIFEST).unwrap();

    let bare = Project::load(dir.path()).unwrap();

    assert!(generate_wiring(&bare, &registry(), "MainWindow").is_err());
}

#[test]
fn writing_outside_generated_is_impossible() {
    let (dir, project) = project_with(HANDLERS);

    write_wiring(&project, &registry(), "MainWindow").unwrap();

    // The developer's handler file is untouched.
    assert_eq!(
        read(dir.path(), Path::new("src/forms/main_window.rs")),
        HANDLERS
    );
}
