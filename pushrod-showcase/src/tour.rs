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

//! The scripted tour: opens every page headless, renders it with SDL's software renderer (the
//! deterministic one the golden tests use), and compares it with a golden image.  The same
//! renderer writes the page images the documentation site shows.
//!
//! * `pushrod-showcase --tour` compares every page with `tests/golden/<page>.png` and exits
//!   with an error when one differs or has no golden.  Differences are written to the diff
//!   directory (`target/golden-diff` by default) as `.actual`, `.expected`, and `.diff` images.
//! * `pushrod-showcase --tour --update-golden` rewrites the goldens.
//! * `pushrod-showcase --screenshots <dir>` writes `showcase-page-<page>.png` for every page.

use crate::golden::{self, Pixels};
use crate::shell::{self, ShowcaseState, WINDOW_H, WINDOW_W};
use pushrod::backend::Backend;
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::input::InputEvent;
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use std::path::{Path, PathBuf};

/// What the command line asked the showcase to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Open the interactive window.
    Window,
    /// Compare every page with its golden image.
    Tour {
        /// Rewrite the goldens instead of comparing.
        update: bool,
    },
    /// Write one image per page to a directory.
    Screenshots(PathBuf),
}

/// Reads the command line.
///
/// # Arguments
///
/// * `args` - The arguments after the program name.
///
/// # Returns
///
/// The command to run.
///
/// # Errors
///
/// Returns a message for an unknown flag, a missing directory, or `--update-golden` without
/// `--tour`.
pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Command, String> {
    let mut tour = false;
    let mut update = false;
    let mut screenshots = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--tour" => tour = true,
            "--update-golden" => update = true,
            "--screenshots" => {
                let dir = args
                    .next()
                    .ok_or("--screenshots needs a directory".to_string())?;

                screenshots = Some(PathBuf::from(dir));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    match (tour, update, screenshots) {
        (true, _, Some(_)) => Err("--tour and --screenshots cannot be combined".to_string()),
        (true, update, None) => Ok(Command::Tour { update }),
        (false, true, _) => Err("--update-golden needs --tour".to_string()),
        (false, false, Some(dir)) => Ok(Command::Screenshots(dir)),
        (false, false, None) => Ok(Command::Window),
    }
}

/// Turns a page name into a file stem: lower case, with every run of other characters as one
/// `-`.
///
/// # Arguments
///
/// * `name` - The page's name.
///
/// # Returns
///
/// The stem, for example `"Row and column"` becomes `"row-and-column"`.
pub fn slug(name: &str) -> String {
    let mut out = String::new();

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }

    out.trim_end_matches('-').to_string()
}

/// Returns where the goldens live.
///
/// # Returns
///
/// The `tests/golden` directory of this crate.
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Returns where diff images are written.
///
/// # Returns
///
/// `$PUSHROD_GOLDEN_DIFF_DIR`, or `target/golden-diff` in the workspace.
pub fn diff_dir() -> PathBuf {
    std::env::var_os("PUSHROD_GOLDEN_DIFF_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/golden-diff"))
}

/// Returns every page's name, in the order the page list shows them.
///
/// # Returns
///
/// The names.
pub fn page_names() -> Vec<&'static str> {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(WINDOW_W, WINDOW_H), 30);
    let state = shell::build(&mut engine, None);

    (0..state.registry.len())
        .filter_map(|index| state.registry.get(index).map(|page| page.name()))
        .collect()
}

/// Chooses the drivers that make rendering deterministic and windowless, unless the
/// environment already chose.
fn prepare_sdl() {
    // SAFETY: called from the main thread before SDL starts, and nothing else reads these.
    unsafe {
        if std::env::var_os("SDL_VIDEODRIVER").is_none() {
            std::env::set_var("SDL_VIDEODRIVER", "dummy");
        }

        if std::env::var_os("SDL_RENDER_DRIVER").is_none() {
            std::env::set_var("SDL_RENDER_DRIVER", "software");
        }
    }
}

/// Renders one page of the showcase window to an image file.
///
/// # Arguments
///
/// * `name` - The page's name.
/// * `path` - Where to save the PNG.
///
/// # Returns
///
/// `Ok` once the file is written.
///
/// # Errors
///
/// Returns a message when no page has that name, or the window or the file fails.
pub fn render_page(name: &str, path: &Path) -> Result<(), String> {
    let window = size(WINDOW_W, WINDOW_H);
    let mut engine: Engine<ShowcaseState> = Engine::new(window, 30);
    let mut state = shell::build(&mut engine, None);

    if state.registry.find(name).is_none() {
        return Err(format!("no page named {name}"));
    }

    state.start_page = Some(name.to_string());

    let mut backend = Sdl2Backend::new(&WindowSettings::new("Pushrod Showcase", window))
        .map_err(|e| e.to_string())?;

    // Without the run loop, nothing reports the display's scale; tell the engine as it would.
    engine.dispatch(InputEvent::ScaleChanged(backend.scale_factor()), &mut state);

    // The first idle pass opens the page; the frames after it lay it out.
    engine.run_idle(&mut state);

    for _ in 0..3 {
        engine.widgets().draw_loop(&mut backend);
    }

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }

    let path = path
        .to_str()
        .ok_or_else(|| format!("{} is not valid text", path.display()))?;

    backend.save_screenshot(path).map_err(|e| e.to_string())
}

/// Writes one image per page for the documentation site.
///
/// # Arguments
///
/// * `dir` - The directory to write `showcase-page-<page>.png` files to.
///
/// # Returns
///
/// The number of images written.
///
/// # Errors
///
/// Returns the first message when a page fails to render.
pub fn write_screenshots(dir: &Path) -> Result<usize, String> {
    prepare_sdl();

    let names = page_names();

    for name in &names {
        render_page(name, &dir.join(format!("showcase-page-{}.png", slug(name))))?;
    }

    Ok(names.len())
}

/// Visits every page and compares it with its golden.
///
/// # Arguments
///
/// * `update` - Rewrite the goldens instead of comparing.
///
/// # Returns
///
/// The number of pages visited.
///
/// # Errors
///
/// Returns one line per page that differs or has no golden, joined with newlines.
pub fn run_tour(update: bool) -> Result<usize, String> {
    prepare_sdl();

    let names = page_names();
    let scratch =
        std::env::temp_dir().join(format!("pushrod-showcase-tour-{}", std::process::id()));
    let mut failures = Vec::new();

    for name in &names {
        let stem = format!("showcase-{}", slug(name));
        let path = scratch.join(format!("{stem}.png"));

        render_page(name, &path)?;

        let actual = golden::load_png(&path).ok_or(format!("{name}: cannot read the render"))?;
        let golden_path = golden_dir().join(format!("{stem}.png"));

        if update {
            golden::save_png(&actual, &golden_path)?;
            continue;
        }

        failures.extend(check(name, &stem, &actual, &golden_path)?);
    }

    let _ = std::fs::remove_dir_all(&scratch);

    if failures.is_empty() {
        Ok(names.len())
    } else {
        Err(failures.join("\n"))
    }
}

/// Compares one page with its golden and writes the diff images when it differs.
///
/// # Arguments
///
/// * `name` - The page's name.
/// * `stem` - The file stem for the diff images.
/// * `actual` - What was rendered.
/// * `golden_path` - Where the golden is.
///
/// # Returns
///
/// `None` when the page matches, otherwise the message to report.
///
/// # Errors
///
/// Returns a message when the diff images cannot be written.
fn check(
    name: &str,
    stem: &str,
    actual: &Pixels,
    golden_path: &Path,
) -> Result<Option<String>, String> {
    let dir = diff_dir();

    let Some(expected) = golden::load_png(golden_path) else {
        golden::write_diffs(&dir, stem, actual, None)?;

        return Ok(Some(format!(
            "{name}: no golden at {} (run with --tour --update-golden)",
            golden_path.display()
        )));
    };
    let result = golden::compare(actual, &expected);

    if result.matches {
        return Ok(None);
    }

    golden::write_diffs(&dir, stem, actual, Some(&expected))?;

    Ok(Some(format!(
        "{name}: {} pixels differ from the golden (diffs in {})",
        result.bad,
        dir.display()
    )))
}
