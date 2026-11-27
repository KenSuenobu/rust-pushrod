---
sidebar_position: 2
title: Widget registry and factory
description: Create widgets from a type name and properties with WidgetRegistry, register the standard set or your own widgets, and get an error, not a panic, for an unknown name.
---

# Widget registry and factory

A form file stores "a button with these properties". To load it, Pushrod has to build a widget from a **type name** such as `pushrod.button`, including widgets your application or another widget library defines. A `WidgetRegistry` does that: it maps stable type names to constructors that build a widget from `Properties`.

```mermaid
flowchart LR
    A["pushrod_widgets::register"] --> R[WidgetRegistry]
    B["Your application's register"] --> R
    C["Another widget library"] --> R
    F["Form file: pushrod.button + properties"] --> R
    R -->|create| W["Box&lt;dyn Widget&gt;"]
    R -->|unknown name| E["Error::UnknownWidgetType"]
```

## Creating widgets by name

```rust checked title="src/main.rs"
use pushrod::registry::{Properties, WidgetRegistry};

fn main() {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    let props = Properties::new()
        .with_text("text", "Save")
        .with_int("x", 10)
        .with_int("y", 50)
        .with_int("width", 80);
    let button = registry.create("pushrod.button", &props).unwrap();

    assert_eq!(button.get_size().w, 80);

    // An unknown name is an error, not a panic.
    assert!(registry.create("acme.tree_view", &Properties::new()).is_err());
}
```

`create` returns a `Box<dyn Widget>` that is not yet in a tree; add it with `WidgetCache::add_child`.

## Properties

`Properties` is a set of named values. Each value is a `PropertyValue`: `Bool`, `Int`, `Float`, `Text`, or `Color`. Build one with `with_bool`, `with_int`, `with_float`, `with_text`, and `with_color`, and read one with `get_bool`, `get_int`, `get_float` (a whole number converts), `get_text`, and `get_color`.

These **common properties** are applied by `create` to every widget, so a constructor never reads them:

| Property | Kind | Effect |
| --- | --- | --- |
| `x`, `y` | `Int` | Origin relative to the parent |
| `width`, `height` | `Int` | Size in pixels |
| `color` | `Color` | Background color |
| `hidden` | `Bool` | Hide the widget and its children |

## The standard widgets

`pushrod_widgets::register(&mut registry)` adds these types:

| Type name | Properties |
| --- | --- |
| `pushrod.button` | `text` |
| `pushrod.check_box` | `text`, `checked` |
| `pushrod.radio_button` | `text`, `checked` |
| `pushrod.text` | `text` |
| `pushrod.boxed_label` | `text` |
| `pushrod.group_box` | `title` |
| `pushrod.glyph_button` | `glyph` (exactly one character) |
| `pushrod.image` | `path` |
| `pushrod.progress_bar` | `max` (default 100), `value` |
| `pushrod.slider` | `min` (default 0), `max` (default 100), `step`, `value`, `vertical` |
| `pushrod.separator` | `vertical` |
| `pushrod.text_field` | none |
| `pushrod.frame` | none |
| `pushrod.panel` | none |

`CustomPaint` needs code rather than data, and `Container`, `ScrollBar`, and `Alert` are built from other widgets, so they are not registered. A missing required property (such as `text` for a button) is an `Error::Property`.

## Registering your own widgets

Give each library its own prefix and a `register` function:

```rust checked title="src/acme.rs"
use pushrod::error::Error;
use pushrod::geometry::size;
use pushrod::registry::WidgetRegistry;
use pushrod::root_widget::RootWidget;

pub fn register(registry: &mut WidgetRegistry) {
    registry.register("acme.blank", |props| {
        let side = props
            .get_int("side")
            .ok_or_else(|| Error::property("side", "required"))?;

        Ok(Box::new(RootWidget::new(size(side as u32, side as u32))))
    });
}
```

`register` returns `true` when it replaced an existing constructor. A library can use this to **replace a standard type** such as `pushrod.button` so existing forms get a new look without changing.

## Errors

| Error | When |
| --- | --- |
| `Error::UnknownWidgetType` | The type name is not registered |
| `Error::Property` | A constructor's required property is missing or has the wrong kind |

Property introspection (listing, getting, and setting properties on a live widget) is a later step; the registry only builds widgets.

Run `cargo run -p pushrod-widgets --example registry` to see a form built from type names.
