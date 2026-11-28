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

//! Loads a form from a form file and attaches a callback by ID, instead of building the widgets in
//! code.  The example writes a small form file in memory, loads it headlessly, and clicks the OK
//! button.  Run it with `cargo run -p pushrod-widgets --example form`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::form::{FormFile, FormValue, ObjectRecord, Property, SymbolKind};
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, MouseButton};

/// The object numbers of the `main` form, as `pushrod-form-ids` would generate them.
mod main_form {
    /// The OK button.
    pub const OK: u32 = 2;
}

/// Builds the sample form file: a panel holding a label and an OK button.
///
/// # Returns
///
/// The form file, with one form named `main`.
fn sample_file() -> FormFile {
    let mut file = FormFile::new();
    let mut panel = ObjectRecord::new("pushrod.panel");

    panel.width = 200;
    panel.height = 80;
    let root = file.add_object(None, panel);

    let mut label = ObjectRecord::new("pushrod.text");

    label.x = 10;
    label.y = 10;
    label.width = 120;
    label.height = 20;
    label.properties.push(Property::named(
        "text",
        FormValue::Text("Sample name".into()),
    ));
    file.add_object(Some(root), label);

    let mut ok = ObjectRecord::new("pushrod.button");

    ok.x = 10;
    ok.y = 40;
    ok.width = 60;
    ok.height = 24;
    ok.properties
        .push(Property::named("text", FormValue::Text("OK".into())));
    file.add_object(Some(root), ok);

    file.add_form("main", root);
    file.add_symbol(SymbolKind::Id, main_form::OK, "OK_BUTTON");

    file
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
    let ok = form.id(main_form::OK).expect("the form has an OK button");

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
