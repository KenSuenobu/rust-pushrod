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

//! Opens the Pushrod Showcase window.  Run it with `cargo run -p pushrod-showcase`.
//!
//! * `--tour` visits every page headless and compares it with its golden image.
//! * `--tour --update-golden` rewrites the golden images.
//! * `--screenshots <dir>` writes one image per page, for the documentation site.

use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod_sdl2::WindowSettings;
use pushrod_showcase::shell::{self, ShowcaseState, WINDOW_H, WINDOW_W};
use pushrod_showcase::tour::{self, Command};
use std::process::ExitCode;

fn main() -> ExitCode {
    match tour::parse_args(std::env::args().skip(1)) {
        Ok(Command::Window) => match open_window() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => fail(&error.to_string()),
        },
        Ok(Command::Tour { update }) => match tour::run_tour(update) {
            Ok(count) => {
                println!(
                    "tour: {count} pages {}",
                    if update { "written" } else { "match" }
                );
                ExitCode::SUCCESS
            }
            Err(message) => fail(&message),
        },
        Ok(Command::Screenshots(dir)) => match tour::write_screenshots(&dir) {
            Ok(count) => {
                println!("screenshots: {count} images written to {}", dir.display());
                ExitCode::SUCCESS
            }
            Err(message) => fail(&message),
        },
        Err(message) => fail(&message),
    }
}

/// Prints an error and picks the failing exit code.
///
/// # Arguments
///
/// * `message` - What went wrong.
///
/// # Returns
///
/// The exit code.
fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

/// Opens the interactive window.
///
/// # Returns
///
/// `Ok` when the window closes.
///
/// # Errors
///
/// Returns the backend's error.
fn open_window() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(WINDOW_W, WINDOW_H), 30);
    let mut settings = WindowSettings::new("Pushrod Showcase", size(WINDOW_W, WINDOW_H));

    settings.resizable = true;

    let state = shell::build(&mut engine, pushrod_showcase::settings::default_path());

    pushrod_sdl2::run(engine, settings, state)?;

    Ok(())
}
