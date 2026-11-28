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

//! Tests for building a form from the engine's resources by [`FormId`], and for `Ui::resources`.
//! Sample data is fictional.

use pushrod::engine::Engine;
use pushrod::form::{FormFile, ObjectRecord};
use pushrod::geometry::size;
use pushrod::resource::{FormId, Resources, StringId};

/// Builds a file with a form named `main` and one string.
fn file() -> FormFile {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));

    file.add_form("main", root);
    file.resources
        .strings
        .add("HELLO", "Hello".to_string())
        .unwrap();

    file
}

#[test]
fn a_form_is_built_from_the_resources_by_id() {
    let mut engine: Engine = Engine::new(size(200, 100), 30);

    pushrod_widgets::register(engine.widget_registry());
    engine.set_resources(Resources::from_bytes(&file().to_bytes().unwrap()).unwrap());

    let form = engine.load_resource_form(FormId(1)).unwrap();

    assert_eq!(form.name(), "main");
    assert_eq!(form.len(), 1);
    assert!(engine.widgets().get(form.root()).is_some());
    assert!(engine.load_resource_form(FormId(2)).is_err());
}

#[test]
fn callbacks_read_resources_through_ui() {
    let mut engine: Engine<Option<String>> = Engine::new(size(200, 100), 30);

    engine.set_resources(Resources::from_bytes(&file().to_bytes().unwrap()).unwrap());
    engine.on_idle(|state, ui| {
        *state = ui.resources().string(StringId(1)).map(str::to_string);
    });

    let mut state = None;

    engine.run_idle(&mut state);

    assert_eq!(state.as_deref(), Some("Hello"));
}
