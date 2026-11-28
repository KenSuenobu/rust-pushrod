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

// ---- Event metadata (6.13) ----

use pushrod::cache::WidgetCache;
use pushrod::event::EventKind;
use pushrod::geometry::point;
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::registry::SHARED_EVENTS;
use std::collections::BTreeSet;

/// Builds the widget `name`, puts it at (10, 10) in a 150 by 24 box under a root panel, and
/// feeds it hover, click, keyboard, and typing input.
///
/// # Returns
///
/// Every kind of event the engine reported while doing so.
fn emitted_kinds(registry: &WidgetRegistry, name: &str) -> BTreeSet<String> {
    let props = Properties::new()
        .with_text("text", "Hello")
        .with_text("title", "Group")
        .with_text("glyph", "+")
        .with_text("path", "missing.png")
        .with_int("width", 150)
        .with_int("height", 24);
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(400, 200)));
    cache.add_child(0, registry.create(name, &props).unwrap());
    cache.get_mut(1).unwrap().set_origin(point(10, 10));

    let (x, y) = (60, 22);
    let key = |key| InputEvent::KeyDown {
        key,
        modifiers: Modifiers::empty(),
        repeat: false,
    };
    let inputs = [
        InputEvent::MouseMove {
            position: point(x, y),
        },
        InputEvent::MouseDown {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
        InputEvent::MouseUp {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
        key(Key::Char(' ')),
        key(Key::Right),
        key(Key::Left),
        InputEvent::Text("a".to_string()),
        key(Key::Return),
        InputEvent::MouseMove {
            position: point(300, 150),
        },
    ];

    inputs
        .into_iter()
        .flat_map(|input| cache.handle_event(input))
        .map(|event| format!("{:?}", event.kind()))
        .collect()
}

#[test]
fn declared_events_match_the_events_a_widget_emits() {
    let registry = standard();
    let shared: BTreeSet<String> = SHARED_EVENTS
        .iter()
        .map(|e| format!("{:?}", e.kind))
        .collect();

    for name in registry.names() {
        let declared: BTreeSet<String> = registry
            .events(name)
            .unwrap()
            .iter()
            .map(|e| format!("{:?}", e.kind))
            .collect();
        let emitted = emitted_kinds(&registry, name);
        let own: BTreeSet<_> = emitted.difference(&shared).cloned().collect();

        assert_eq!(
            own, declared,
            "{name}: the events it emitted (besides the shared ones) differ from its declaration"
        );
    }
}

#[test]
fn every_standard_widget_declares_at_most_one_default_and_a_callback_for_it() {
    let registry = standard();

    for name in registry.names() {
        let events = registry.events(name).unwrap();
        let defaults: Vec<_> = events.iter().filter(|e| e.is_default).collect();

        assert!(defaults.len() <= 1, "{name}");
        assert_eq!(defaults.len(), usize::from(!events.is_empty()), "{name}");

        for event in events {
            assert_eq!(event.name, format!("{:?}", event.kind), "{name}");
            assert!(!event.description.is_empty(), "{name}.{}", event.name);
            assert!(event.callback.is_some(), "{name}.{}", event.name);
        }
    }
}

#[test]
fn clicking_widgets_default_to_clicked_and_value_widgets_to_their_change() {
    let registry = standard();
    let default = |name: &str| registry.default_event(name).map(|e| (e.name, e.callback));

    assert_eq!(
        default("pushrod.button"),
        Some(("Clicked", Some("on_click")))
    );
    assert_eq!(
        default("pushrod.check_box"),
        Some(("Toggled", Some("on_toggle")))
    );
    assert_eq!(
        default("pushrod.slider"),
        Some(("ValueChanged", Some("on_value_change")))
    );
    assert_eq!(
        default("pushrod.text_field"),
        Some(("TextChanged", Some("on_text_change")))
    );
    assert_eq!(default("pushrod.panel"), None);
}

#[test]
fn all_events_lists_the_widgets_own_before_the_shared_ones() {
    let registry = standard();
    let all = registry.all_events("pushrod.text_field").unwrap();

    assert_eq!(all.len(), 2 + SHARED_EVENTS.len());
    assert_eq!(all[0].kind, EventKind::TextChanged);
    assert!(registry.all_events("pushrod.missing").is_none());
    assert_eq!(
        registry.all_events("pushrod.panel").unwrap().len(),
        SHARED_EVENTS.len()
    );
}

#[test]
fn replacing_a_constructor_keeps_the_declared_events() {
    let mut registry = standard();

    assert!(registry.register("pushrod.button", |_| Ok(Box::new(Button::new("Styled")))));
    assert_eq!(
        registry.default_event("pushrod.button").unwrap().name,
        "Clicked"
    );
}

#[test]
fn a_third_party_library_declares_its_own_events() {
    const EVENTS: &[pushrod::registry::EventInfo] = &[pushrod::registry::EventInfo {
        name: "Opened",
        description: "The node opened.",
        kind: EventKind::Custom,
        payload: &[],
        is_default: true,
        callback: None,
    }];
    let mut registry = WidgetRegistry::new();

    registry.register_with_events("acme.tree", EVENTS, |_| Ok(Box::new(Panel::new())));

    assert_eq!(registry.default_event("acme.tree").unwrap().name, "Opened");
    assert_eq!(registry.events("acme.tree").unwrap().len(), 1);
    assert!(registry.events("acme.none").is_none());

    // A widget registered the old way declares nothing and has no default.
    registry.register("acme.plain", |_| Ok(Box::new(Panel::new())));
    assert!(registry.events("acme.plain").unwrap().is_empty());
    assert!(registry.default_event("acme.plain").is_none());

    // Replacing with events replaces the declaration.
    assert!(registry.register_with_events("acme.tree", &[], |_| Ok(Box::new(Panel::new()))));
    assert!(registry.default_event("acme.tree").is_none());
}
