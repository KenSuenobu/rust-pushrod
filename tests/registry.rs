// Widget Registry Tests
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

//! Tests for the widget registry: custom registration, replacement, common properties, and
//! unknown names reported as errors.

use pushrod::color::Color;
use pushrod::error::Error;
use pushrod::geometry::size;
use pushrod::registry::{Properties, PropertyValue, WidgetRegistry};
use pushrod::root_widget::RootWidget;

/// Builds a registry holding one custom type.
fn registry() -> WidgetRegistry {
    let mut registry = WidgetRegistry::new();

    registry.register("acme.blank", |props| {
        let w = props
            .get_int("w")
            .ok_or_else(|| Error::property("w", "required"))?;

        Ok(Box::new(RootWidget::new(size(w as u32, 10))))
    });

    registry
}

#[test]
fn custom_widgets_are_registrable_and_listed() {
    let registry = registry();

    assert!(registry.contains("acme.blank"));
    assert_eq!(registry.names(), vec!["acme.blank"]);
    assert_eq!(
        registry
            .create("acme.blank", &Properties::new().with_int("w", 25))
            .unwrap()
            .get_size()
            .w,
        25
    );
}

#[test]
fn unknown_names_are_errors() {
    let error = registry()
        .create("acme.nothing", &Properties::new())
        .err()
        .unwrap();

    assert_eq!(error, Error::unknown_widget_type("acme.nothing"));
    assert_eq!(
        error.to_string(),
        "widget type \"acme.nothing\" is not registered"
    );
}

#[test]
fn constructor_errors_pass_through() {
    let error = registry()
        .create("acme.blank", &Properties::new())
        .err()
        .unwrap();

    assert_eq!(error.to_string(), "property \"w\": required");
}

#[test]
fn registering_a_name_again_replaces_it() {
    let mut registry = registry();

    assert!(registry.register("acme.blank", |_| Ok(Box::new(RootWidget::new(size(1, 1))))));
    assert!(!registry.register("acme.other", |_| Ok(Box::new(RootWidget::new(size(1, 1))))));
    assert_eq!(
        registry
            .create("acme.blank", &Properties::new())
            .unwrap()
            .get_size()
            .w,
        1
    );
}

#[test]
fn common_properties_apply_to_every_widget() {
    let props = Properties::new()
        .with_int("w", 25)
        .with_int("x", 7)
        .with_int("y", 9)
        .with_int("width", 40)
        .with_color("color", Color::RGB(1, 2, 3))
        .with_bool("hidden", true);
    let widget = registry().create("acme.blank", &props).unwrap();

    assert_eq!((widget.get_origin().x, widget.get_origin().y), (7, 9));
    assert_eq!((widget.get_size().w, widget.get_size().h), (40, 10));
    assert_eq!(widget.get_color(), Color::RGB(1, 2, 3));
    assert!(widget.is_hidden());
}

#[test]
fn property_getters_check_the_kind() {
    let props = Properties::new()
        .with_int("n", 3)
        .with_float("f", 1.5)
        .with_text("t", "hi");

    assert_eq!(props.get_int("n"), Some(3));
    assert_eq!(props.get_float("n"), Some(3.0));
    assert_eq!(props.get_float("f"), Some(1.5));
    assert_eq!(props.get_int("f"), None);
    assert_eq!(props.get_text("t"), Some("hi"));
    assert_eq!(props.get_bool("t"), None);
    assert_eq!(props.get("t"), Some(&PropertyValue::Text("hi".into())));
    assert_eq!(props.get("missing"), None);
}
