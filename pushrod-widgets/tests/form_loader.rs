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

//! # Form files
//!

//! Tests for loading a form from a form file: the golden comparison with a code-built form, ID
//! resolution, error messages, and the engine event.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::event::{EngineEvent, EngineEventKind, Event};
use pushrod::form::{FormFile, FormValue, LayoutSpec, ObjectRecord, Property, SymbolKind};
use pushrod::geometry::{point, size};
use pushrod::widget::Widget;
use pushrod_widgets::prelude::*;

/// ID constants, as a generator would write them.
mod main_form {
    pub const ROOT: u32 = 0;
    pub const LABEL: u32 = 1;
    pub const OK: u32 = 2;
}

/// Builds the sample form file: a panel with a label and an OK button.
fn sample_file() -> FormFile {
    let mut file = FormFile::new();
    let mut panel = ObjectRecord::new("pushrod.panel");

    panel.x = 5;
    panel.y = 6;
    panel.width = 200;
    panel.height = 80;
    let root = file.add_object(None, panel);

    let mut label = ObjectRecord::new("pushrod.text");

    label.x = 10;
    label.y = 10;
    label.width = 100;
    label.height = 20;
    label
        .properties
        .push(Property::named("text", FormValue::Text("Name".into())));
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
    file.add_symbol(SymbolKind::Action, 7, "confirm");

    file
}

/// Makes an engine whose registry holds the standard widgets.
fn engine() -> Engine {
    let mut engine: Engine = Engine::new(size(320, 200), 30);

    pushrod_widgets::register(engine.widget_registry());

    engine
}

/// Draws one frame and returns what each widget painted, by order of painting.
fn paint(engine: &mut Engine) -> Vec<Vec<pushrod::backend::headless::PaintCall>> {
    let mut backend = HeadlessBackend::new(size(320, 200));

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);

    backend
        .painted
        .into_iter()
        .map(|(_, calls)| calls)
        .collect()
}

#[test]
fn loaded_form_renders_identically_to_the_code_built_form() {
    let mut loaded = engine();

    loaded.load_form(&sample_file(), "main").unwrap();

    let mut built = engine();
    let mut panel = Panel::new();

    panel.set_origin(point(5, 6));
    panel.set_size(size(200, 80));
    let panel = built.add_widget(panel);

    let mut label = Text::new("Name");

    label.set_origin(point(10, 10));
    label.set_size(size(100, 20));
    built.add_child_widget(panel, label);

    let mut ok = Button::new("OK");

    ok.set_origin(point(10, 40));
    ok.set_size(size(60, 24));
    built.add_child_widget(panel, ok);

    let golden = paint(&mut built);

    assert_eq!(golden.len(), 4);
    assert_eq!(paint(&mut loaded), golden);
}

#[test]
fn ids_resolve_by_number_and_by_name() {
    let mut engine = engine();
    let form = engine.load_form(&sample_file(), "main").unwrap();

    assert_eq!(form.name(), "main");
    assert_eq!(form.len(), 3);
    assert_eq!(form.id(main_form::ROOT), Some(form.root()));

    let ok = form.id(main_form::OK).unwrap();

    assert_eq!(form.named("OK_BUTTON"), Some(ok));
    assert_eq!(form.action("confirm"), Some(7));
    assert!(form.id(99).is_none());
    assert!(form.named("MISSING").is_none());
    assert!(engine.widgets().get_as::<Button>(ok).is_some());
    assert!(
        engine
            .widgets()
            .get_as::<Text>(form.id(main_form::LABEL).unwrap())
            .is_some()
    );
}

#[test]
fn a_form_loads_into_a_given_parent() {
    let mut engine = engine();
    let first = engine.load_form(&sample_file(), "main").unwrap();
    let second = engine
        .load_form_into(first.root(), &sample_file(), "main")
        .unwrap();

    assert_ne!(first.root(), second.root());
    assert!(matches!(
        engine.load_form_into(9999, &sample_file(), "main"),
        Err(Error::UnknownWidget { id: 9999 })
    ));
}

#[test]
fn errors_name_the_object_and_property() {
    let mut engine = engine();
    let mut file = sample_file();

    file.objects[2].properties.clear();

    let error = engine.load_form(&file, "main").unwrap_err().to_string();

    assert!(error.contains("object 2"), "{error}");
    assert!(error.contains("pushrod.button"), "{error}");
    assert!(error.contains("text"), "{error}");
}

#[test]
fn unknown_types_forms_and_layouts_are_errors_and_change_nothing() {
    let mut engine = engine();
    let before = engine.widgets().len();
    let mut file = sample_file();

    file.objects[1].type_name = "acme.missing".into();

    let error = engine.load_form(&file, "main").unwrap_err().to_string();

    assert!(
        error.contains("object 1") && error.contains("acme.missing"),
        "{error}"
    );

    let error = engine
        .load_form(&sample_file(), "other")
        .unwrap_err()
        .to_string();

    assert!(error.contains("other"), "{error}");

    let mut file = sample_file();

    file.objects[0]
        .properties
        .push(Property::Layout(LayoutSpec {
            kind: 99,
            params: vec![],
        }));

    let error = engine.load_form(&file, "main").unwrap_err().to_string();

    assert!(
        error.contains("object 0") && error.contains("layout kind 99"),
        "{error}"
    );
    assert_eq!(engine.widgets().len(), before, "a failed load adds nothing");
}

#[test]
fn a_damaged_file_is_an_error() {
    let mut engine = engine();
    let mut file = sample_file();

    file.objects[1].first_child = 77;

    assert!(matches!(
        engine.load_form(&file, "main"),
        Err(Error::FormFile { .. })
    ));
}

#[test]
fn loading_emits_form_loaded() {
    let mut engine = engine();

    engine.load_form(&sample_file(), "main").unwrap();

    let events = engine.widgets().take_pending_events();
    let loaded: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Engine(e @ EngineEvent::FormLoaded { .. }) => Some(e),
            _ => None,
        })
        .collect();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].kind(), EngineEventKind::FormLoaded);
    assert!(
        matches!(loaded[0], EngineEvent::FormLoaded { form, widgets: 3, .. } if form == "main")
    );
}
