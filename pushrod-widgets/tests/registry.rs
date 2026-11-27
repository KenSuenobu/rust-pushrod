// Standard Widget Registration Tests
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

//! Tests that every standard widget is registered and builds from properties.

use pushrod::error::Error;
use pushrod::geometry::size;
use pushrod::registry::{Properties, WidgetRegistry};
use pushrod::root_widget::RootWidget;
use pushrod_widgets::prelude::*;

/// Returns a registry holding the standard widgets.
fn standard() -> WidgetRegistry {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    registry
}

#[test]
fn all_built_ins_are_registered_and_build() {
    let registry = standard();
    let props = Properties::new()
        .with_text("text", "Hello")
        .with_text("title", "Group")
        .with_text("glyph", "+")
        .with_text("path", "missing.png");

    assert_eq!(registry.names().len(), 14);

    for name in registry.names() {
        assert!(name.starts_with("pushrod."));
        assert!(registry.create(name, &props).is_ok(), "{name}");
    }
}

#[test]
fn properties_reach_the_widget() {
    let registry = standard();
    let props = Properties::new()
        .with_text("text", "Agree")
        .with_bool("checked", true)
        .with_int("x", 12)
        .with_int("width", 90);
    let widget = registry.create("pushrod.check_box", &props).unwrap();
    let any: &dyn std::any::Any = &*widget;
    let check = any.downcast_ref::<CheckBox>().unwrap();

    assert!(check.is_checked());
    assert_eq!(widget.get_origin().x, 12);
    assert_eq!(widget.get_size().w, 90);
}

#[test]
fn missing_or_bad_properties_are_errors() {
    let registry = standard();

    assert!(matches!(
        registry.create("pushrod.button", &Properties::new()),
        Err(Error::Property { .. })
    ));
    assert!(matches!(
        registry.create(
            "pushrod.glyph_button",
            &Properties::new().with_text("glyph", "ab")
        ),
        Err(Error::Property { .. })
    ));
    assert!(matches!(
        registry.create("pushrod.nope", &Properties::new()),
        Err(Error::UnknownWidgetType { .. })
    ));
}

#[test]
fn a_library_can_replace_a_standard_type() {
    let mut registry = standard();

    registry.register("pushrod.button", |_| {
        Ok(Box::new(RootWidget::new(size(5, 5))))
    });

    assert_eq!(
        registry
            .create("pushrod.button", &Properties::new())
            .unwrap()
            .get_size()
            .w,
        5
    );
}
