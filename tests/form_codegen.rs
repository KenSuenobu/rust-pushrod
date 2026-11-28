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

//! Tests for ID constant generation: `generate_ids`, `write_ids`, and the `pushrod-form-ids` command.

use pushrod::error::Error;
use pushrod::form::codegen::{generate_ids, write_ids};
use pushrod::form::{FormFile, FormIndex, ObjectRecord, SymbolKind};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Builds a file with a `main` form (root 0, OK 1, Cancel 2) and a `Sign In` form (root 3, user 4).
fn sample() -> FormFile {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    let ok = file.add_object(Some(root), ObjectRecord::new("pushrod.button"));
    let cancel = file.add_object(Some(root), ObjectRecord::new("pushrod.button"));
    let second = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    let user = file.add_object(Some(second), ObjectRecord::new("pushrod.text_field"));

    file.add_form("main", root);
    file.add_form("Sign In", second);
    file.add_symbol(SymbolKind::Id, cancel, "CANCEL_BUTTON");
    file.add_symbol(SymbolKind::Id, ok, "OK");
    file.add_symbol(SymbolKind::Id, user, "userName");
    file.add_symbol(SymbolKind::Action, 7, "save");
    file
}

/// Makes a fresh temporary directory for one test.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pushrod-codegen-{}-{name}", std::process::id()));

    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn generates_a_module_per_form_and_a_constant_per_symbol() {
    let source = generate_ids(&sample()).unwrap();

    assert!(source.contains("pub mod main {"));
    assert!(source.contains("pub mod sign_in {"));
    assert!(source.contains("pub const OK: FormIndex = FormIndex(1);"));
    assert!(source.contains("pub const CANCEL_BUTTON: FormIndex = FormIndex(2);"));
    assert!(source.contains("pub const USER_NAME: FormIndex = FormIndex(4);"));
    assert!(!source.contains("save"), "actions are not ID constants");
    assert!(
        source.find("OK:").unwrap() < source.find("CANCEL_BUTTON:").unwrap(),
        "constants are ordered by object number"
    );
}

#[test]
fn output_is_deterministic() {
    let mut reordered = sample();

    reordered.symbols.reverse();
    assert_eq!(
        generate_ids(&sample()).unwrap(),
        generate_ids(&reordered).unwrap()
    );
}

#[test]
fn output_is_rustfmt_clean() {
    let source = generate_ids(&sample()).unwrap();
    let child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    let Ok(mut child) = child else {
        eprintln!("rustfmt is not installed; skipping");
        return;
    };

    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), source);
}

#[test]
fn a_form_without_symbols_is_an_empty_module() {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));

    file.add_form("empty", root);
    assert!(generate_ids(&file).unwrap().contains("pub mod empty {}"));
}

#[test]
fn symbols_outside_every_form_are_left_out() {
    let mut file = sample();

    file.add_symbol(SymbolKind::Id, 99, "GHOST");
    assert!(!generate_ids(&file).unwrap().contains("GHOST"));
}

#[test]
fn unusable_names_are_errors() {
    for (form, symbol) in [
        ("type", "OK"),
        ("main", "9LIVES"),
        ("main", "--"),
        ("", "OK"),
    ] {
        let mut file = FormFile::new();
        let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));

        file.add_form(form, root);
        file.add_symbol(SymbolKind::Id, root, symbol);
        assert!(generate_ids(&file).is_err(), "{form:?} / {symbol:?}");
    }
}

#[test]
fn duplicate_names_are_errors() {
    let mut file = sample();

    file.add_symbol(SymbolKind::Id, 2, "ok");
    assert!(generate_ids(&file).is_err(), "two constants named OK");

    let mut file = sample();

    file.add_form("MAIN", 3);
    assert!(generate_ids(&file).is_err(), "two modules named main");
}

#[test]
fn the_generated_type_is_the_form_index() {
    assert_eq!(u32::from(FormIndex(5)), 5);
}

#[test]
fn write_ids_writes_once_and_leaves_an_unchanged_file_alone() {
    let dir = temp_dir("write");
    let (form, out) = (dir.join("a.prsc"), dir.join("ids.rs"));

    sample().write_to_path_for_test(&form);
    write_ids(&form, &out).unwrap();
    let first = std::fs::metadata(&out).unwrap().modified().unwrap();

    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        generate_ids(&sample()).unwrap()
    );
    std::thread::sleep(std::time::Duration::from_millis(20));
    write_ids(&form, &out).unwrap();
    assert_eq!(std::fs::metadata(&out).unwrap().modified().unwrap(), first);
}

#[test]
fn write_ids_reports_the_path_on_failure() {
    let dir = temp_dir("fail");
    let missing = dir.join("missing.prsc");
    let damaged = dir.join("damaged.prsc");

    std::fs::write(&damaged, b"not a form").unwrap();
    for path in [&missing, &damaged] {
        match write_ids(path, dir.join("ids.rs")) {
            Err(Error::FormFile { path: shown, .. }) => {
                assert_eq!(shown, path.display().to_string())
            }
            other => panic!("expected a form file error, got {other:?}"),
        }
    }
}

#[test]
fn the_command_writes_the_file_and_fails_on_bad_input() {
    let dir = temp_dir("cli");
    let (form, out) = (dir.join("a.prsc"), dir.join("ids.rs"));
    let command = env!("CARGO_BIN_EXE_pushrod-form-ids");

    sample().write_to_path_for_test(&form);
    assert!(
        Command::new(command)
            .arg(&form)
            .arg(&out)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::fs::read_to_string(&out)
            .unwrap()
            .contains("pub mod main")
    );
    assert!(!Command::new(command).status().unwrap().success());
    assert!(
        !Command::new(command)
            .arg(dir.join("missing.prsc"))
            .arg(&out)
            .status()
            .unwrap()
            .success()
    );
}

/// Writes a form file to disk, for the tests.
trait WriteForTest {
    /// Writes the file's bytes to a path.
    fn write_to_path_for_test(&self, path: &std::path::Path);
}

impl WriteForTest for FormFile {
    /// Writes the file's bytes to a path.
    ///
    /// # Arguments
    ///
    /// * `path` - Where to write.
    fn write_to_path_for_test(&self, path: &std::path::Path) {
        std::fs::write(path, self.to_bytes().unwrap()).unwrap();
    }
}
