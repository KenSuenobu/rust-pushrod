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

//! Loads a form from a form file and attaches a callback by a generated ID constant, instead of
//! building the widgets in code.  `build.rs` writes a small form file and generates its constants
//! with `pushrod::form::codegen::write_ids`; the example loads the form headlessly and clicks the
//! OK button.  Run it with `cargo run -p pushrod-widgets --example form`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::form::FormFile;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, MouseButton};

// The ID constants `build.rs` generated from the form file (`pub mod main { pub const OK_BUTTON }`).
include!(concat!(env!("OUT_DIR"), "/form_ids.rs"));

/// Reads the form file `build.rs` wrote.
///
/// # Returns
///
/// The form file, with one form named `main`.
fn sample_file() -> FormFile {
    FormFile::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/sample.prsc")))
        .expect("the sample form is valid")
}

fn main() {
    let mut engine: Engine<Vec<String>> = Engine::new(size(320, 200), 30);

    pushrod_widgets::register(engine.widget_registry());
    engine.on_engine_event(|log, _ui, event| {
        if let EngineEvent::FormLoaded { form, widgets, .. } = event {
            log.push(format!("loaded form {form} with {widgets} widgets"));
        }
    });

    let form = engine
        .load_form(&sample_file(), "main")
        .expect("the sample form is valid");
    let ok = form
        .widget(main::OK_BUTTON)
        .expect("the form has an OK button");

    engine.on_click(ok, |log, _ui| log.push("OK was clicked".into()));

    let mut backend = HeadlessBackend::new(size(320, 200));
    let position = point(20, 50);

    for event in [
        InputEvent::MouseDown {
            position,
            button: MouseButton::Left,
            clicks: 1,
        },
        InputEvent::MouseUp {
            position,
            button: MouseButton::Left,
            clicks: 1,
        },
    ] {
        backend.push_event(event);
    }

    for line in engine.run(&mut backend, Vec::new()) {
        println!("{line}");
    }
}
