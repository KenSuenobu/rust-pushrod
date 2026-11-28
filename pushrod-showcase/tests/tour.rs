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

//! Tests for the scripted tour (issue #301).
//!
//! * The command line, the file names, and the comparison are plain logic and always run.
//! * Every page must have a golden image checked in.
//! * Rendering needs SDL, and text antialiasing differs between platforms, so the render
//!   checks run where `PUSHROD_GOLDEN=1` is set (the `golden` CI job): the tour compares every
//!   page with its golden, and the screenshot writer produces one image per page.  Both share
//!   one test function because SDL's video subsystem is initialized once.

use pushrod_showcase::golden::{Pixels, compare, diff_image};
use pushrod_showcase::tour::{self, Command};
use std::path::PathBuf;

/// Builds an image of one color.
fn solid(w: u32, h: u32, color: [u8; 4]) -> Pixels {
    Pixels {
        w,
        h,
        rgba: color
            .iter()
            .copied()
            .cycle()
            .take((w * h * 4) as usize)
            .collect(),
    }
}

/// Splits a command line into arguments.
fn args(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

#[test]
fn no_arguments_open_the_window() {
    assert_eq!(tour::parse_args(args("")), Ok(Command::Window));
}

#[test]
fn tour_flags_are_read() {
    assert_eq!(
        tour::parse_args(args("--tour")),
        Ok(Command::Tour { update: false })
    );
    assert_eq!(
        tour::parse_args(args("--tour --update-golden")),
        Ok(Command::Tour { update: true })
    );
    assert_eq!(
        tour::parse_args(args("--screenshots docs/img")),
        Ok(Command::Screenshots(PathBuf::from("docs/img")))
    );
}

#[test]
fn bad_command_lines_are_refused() {
    assert!(tour::parse_args(args("--screenshots")).is_err());
    assert!(tour::parse_args(args("--update-golden")).is_err());
    assert!(tour::parse_args(args("--tour --screenshots out")).is_err());
    assert!(
        tour::parse_args(args("--bogus"))
            .unwrap_err()
            .contains("--bogus")
    );
}

#[test]
fn page_names_become_file_stems() {
    assert_eq!(tour::slug("Row and column"), "row-and-column");
    assert_eq!(tour::slug("Focus and Tab"), "focus-and-tab");
    assert_eq!(tour::slug("  Dialogs & Alerts! "), "dialogs-alerts");
}

#[test]
fn page_stems_are_unique() {
    let mut stems: Vec<String> = tour::page_names().iter().map(|n| tour::slug(n)).collect();
    let count = stems.len();

    stems.sort();
    stems.dedup();

    assert!(count > 0);
    assert_eq!(stems.len(), count, "two pages share a file name");
}

#[test]
fn every_page_has_a_golden_image() {
    let missing: Vec<String> = tour::page_names()
        .iter()
        .map(|name| format!("showcase-{}.png", tour::slug(name)))
        .filter(|file| !tour::golden_dir().join(file).exists())
        .collect();

    assert!(
        missing.is_empty(),
        "no golden for {missing:?}; run: cargo run -p pushrod-showcase -- --tour --update-golden"
    );
}

#[test]
fn small_differences_match_and_big_ones_do_not() {
    let base = solid(10, 10, [100, 100, 100, 255]);

    assert!(compare(&base, &solid(10, 10, [102, 100, 100, 255])).matches);

    let other = solid(10, 10, [200, 100, 100, 255]);
    let result = compare(&base, &other);

    assert!(!result.matches);
    assert_eq!(result.bad, 100);
    assert!(!compare(&base, &solid(5, 5, [100, 100, 100, 255])).matches);
}

#[test]
fn the_diff_image_marks_changed_pixels_red() {
    let mut changed = solid(2, 1, [10, 10, 10, 255]);

    changed.rgba[0] = 250;

    let diff = diff_image(&changed, &solid(2, 1, [10, 10, 10, 255]));

    assert_eq!(&diff.rgba[0..4], &[255, 0, 0, 255]);
    assert_ne!(&diff.rgba[4..8], &[255, 0, 0, 255]);
}

#[test]
fn the_tour_and_the_screenshots_render_every_page() {
    if std::env::var_os("PUSHROD_GOLDEN").is_none() {
        eprintln!("skipped: set PUSHROD_GOLDEN=1 to render the tour and compare with tests/golden");

        return;
    }

    let result = tour::run_tour(false);
    let dir = std::env::temp_dir().join(format!("pushrod-showcase-shots-{}", std::process::id()));
    let written = tour::write_screenshots(&dir);
    let names = tour::page_names();

    for name in &names {
        let file = dir.join(format!("showcase-page-{}.png", tour::slug(name)));

        assert!(file.exists(), "no screenshot for {name}");
    }

    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(written, Ok(names.len()));
    assert_eq!(result, Ok(names.len()));
}
