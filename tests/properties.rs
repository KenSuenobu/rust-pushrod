// Property Introspection Tests
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

//! Tests for listing and setting widget properties by name, and the `PropertyChanged` event.

use pushrod::error::Error;
use pushrod::event::{EngineEvent, EngineEventKind};
use pushrod::prelude::*;
use pushrod::properties::{Property, PropertyKind, core_properties};
use pushrod::registry::PropertyValue;
use pushrod::root_widget::RootWidget;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

/// Returns an engine with one widget and the ID of that widget.
fn engine_with_widget() -> (Engine<()>, WidgetId) {
    let mut engine: Engine<()> = Engine::new(size(200, 100), 30);
    let id = engine.add_widget(RootWidget::new(size(50, 20)));

    (engine, id)
}

/// Returns the value of a listed property.
fn value_of(engine: &mut Engine<()>, id: WidgetId, name: &str) -> PropertyValue {
    engine
        .widgets()
        .get(id)
        .unwrap()
        .properties()
        .into_iter()
        .find(|p| p.name == name)
        .unwrap()
        .value
}

#[test]
fn every_widget_lists_the_core_properties() {
    let (mut engine, id) = engine_with_widget();
    let names: Vec<String> = engine
        .widgets()
        .get(id)
        .unwrap()
        .properties()
        .into_iter()
        .map(|p| p.name)
        .collect();

    for name in [
        "x",
        "y",
        "width",
        "height",
        "color",
        "hidden",
        "enabled",
        "tooltip",
        "behavior",
        "state",
        "stretch",
        "margin_left",
        "margin_top",
        "margin_right",
        "margin_bottom",
        "align_horizontal",
        "align_vertical",
    ] {
        assert!(names.contains(&name.to_string()), "{name}");
    }
}

#[test]
fn each_core_property_round_trips() {
    let (mut engine, id) = engine_with_widget();
    let changes = [
        ("x", PropertyValue::Int(7)),
        ("y", PropertyValue::Int(9)),
        ("width", PropertyValue::Int(60)),
        ("height", PropertyValue::Int(30)),
        ("color", PropertyValue::Color(Color::RGB(1, 2, 3))),
        ("hidden", PropertyValue::Bool(true)),
        ("enabled", PropertyValue::Bool(false)),
        ("tooltip", PropertyValue::Text("Hello".to_string())),
        ("stretch", PropertyValue::Int(2)),
        ("margin_left", PropertyValue::Int(1)),
        ("margin_top", PropertyValue::Int(2)),
        ("margin_right", PropertyValue::Int(3)),
        ("margin_bottom", PropertyValue::Int(4)),
        (
            "align_horizontal",
            PropertyValue::Text("center".to_string()),
        ),
        ("align_vertical", PropertyValue::Text("fill".to_string())),
    ];

    for (name, value) in changes {
        assert_eq!(
            engine.set_widget_property(id, name, &value),
            Ok(true),
            "{name}"
        );
        assert_eq!(value_of(&mut engine, id, name), value, "{name}");
        assert_eq!(
            engine.set_widget_property(id, name, &value),
            Ok(false),
            "{name}"
        );
    }

    let widget = engine.widgets().get(id).unwrap();

    assert_eq!(widget.get_origin(), point(7, 9));
    assert_eq!(widget.get_size(), size(60, 30));
    assert!(widget.is_hidden());
    assert!(!widget.is_enabled());
    assert_eq!(widget.tooltip(), "Hello");
}

#[test]
fn behavior_and_state_are_listed_as_bits() {
    let (mut engine, id) = engine_with_widget();
    let bits = i64::from(Behavior::HIDDEN.bits());

    assert_eq!(
        engine.set_widget_property(id, "behavior", &PropertyValue::Int(bits)),
        Ok(true)
    );
    assert!(engine.widgets().get(id).unwrap().is_hidden());
    assert!(
        engine
            .set_widget_property(id, "behavior", &PropertyValue::Int(1 << 15))
            .is_err()
    );
}

#[test]
fn bad_names_kinds_and_values_are_errors() {
    let (mut engine, id) = engine_with_widget();

    assert!(matches!(
        engine.set_widget_property(id, "nope", &PropertyValue::Int(1)),
        Err(Error::Property { .. })
    ));
    assert!(matches!(
        engine.set_widget_property(id, "x", &PropertyValue::Text("1".to_string())),
        Err(Error::Property { .. })
    ));
    assert!(matches!(
        engine.set_widget_property(id, "width", &PropertyValue::Int(-1)),
        Err(Error::Property { .. })
    ));
    assert!(matches!(
        engine.set_widget_property(
            id,
            "align_horizontal",
            &PropertyValue::Text("up".to_string())
        ),
        Err(Error::Property { .. })
    ));
    assert_eq!(
        engine.set_widget_property(999, "x", &PropertyValue::Int(1)),
        Err(Error::UnknownWidget { id: 999 })
    );
}

#[test]
fn a_change_emits_property_changed_but_the_same_value_does_not() {
    let (mut engine, id) = engine_with_widget();
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    let mut state = ();

    engine.on_engine_event_filtered(
        &[EngineEventKind::PropertyChanged],
        move |_s, _ui, event| sink.borrow_mut().push(event.clone()),
    );

    engine
        .set_widget_property(id, "x", &PropertyValue::Int(11))
        .unwrap();
    engine
        .set_widget_property(id, "x", &PropertyValue::Int(11))
        .unwrap();
    engine
        .set_widget_property(id, "nope", &PropertyValue::Int(1))
        .unwrap_err();
    engine.advance(Instant::now(), &mut state);

    assert_eq!(
        *log.borrow(),
        vec![EngineEvent::PropertyChanged {
            id,
            name: "x".to_string()
        }]
    );
}

#[test]
fn property_constructors_set_the_kind() {
    assert_eq!(Property::bool("a", true).kind, PropertyKind::Bool);
    assert_eq!(Property::int("a", 1).kind, PropertyKind::Int);
    assert_eq!(Property::float("a", 1.0).kind, PropertyKind::Float);
    assert_eq!(Property::text("a", "t").kind, PropertyKind::Text);
    assert_eq!(Property::color("a", Color::BLACK).kind, PropertyKind::Color);
}

#[test]
fn core_properties_works_on_a_trait_object() {
    let widget = RootWidget::new(size(5, 6));
    let dynamic: &dyn Widget = &widget;

    assert_eq!(core_properties(dynamic).len(), 17);
}
