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

use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod_sdl2::WindowSettings;
use pushrod_showcase::shell::{self, ShowcaseState, WINDOW_H, WINDOW_W};

fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(WINDOW_W, WINDOW_H), 30);
    let mut settings = WindowSettings::new("Pushrod Showcase", size(WINDOW_W, WINDOW_H));

    settings.resizable = true;

    let state = shell::build(&mut engine, pushrod_showcase::settings::default_path());

    pushrod_sdl2::run(engine, settings, state)?;

    Ok(())
}
