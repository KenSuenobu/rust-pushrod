---
sidebar_position: 19
title: Registry
description: Registers every standard widget that can be built from properties alone under the pushrod. prefix.
---

# Registry

`register` (module `pushrod_widgets::registry`, also `pushrod_widgets::register`): adds the standard widgets to a `WidgetRegistry` so form files can create them by type name. It is not a widget itself. For how the registry works, see [Widget registry and factory](../forms/widget-registry-and-factory.md).

## Using it

```rust checked title="src/main.rs"
use pushrod::registry::{Properties, WidgetRegistry};

fn main() {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    let ok = registry
        .create("pushrod.button", &Properties::new().with_text("text", "OK").with_int("width", 80))
        .unwrap();

    assert_eq!(ok.get_size().w, 80);
}
```

## Registered type names

Every type also accepts `x`, `y`, `width`, `height`, `hidden`, and `color`.

| Type name | Properties |
|---|---|
| `pushrod.button` | `text` |
| `pushrod.check_box` | `text`, `checked` |
| `pushrod.radio_button` | `text`, `checked` |
| `pushrod.text` | `text` |
| `pushrod.boxed_label` | `text` |
| `pushrod.group_box` | `title` |
| `pushrod.glyph_button` | `glyph` (one character) |
| `pushrod.image` | `path` |
| `pushrod.progress_bar` | `max` (default 100), `value` |
| `pushrod.slider` | `min` (default 0), `max` (default 100), `step`, `value`, `vertical` |
| `pushrod.separator` | `vertical` |
| `pushrod.text_field` | none |
| `pushrod.frame` | none |
| `pushrod.panel` | none |

## Notes

- A missing required property (for example `text` on a button) is an `Error::Property`, not a panic.
- Widgets that need code, not data (`CustomPaint`), or that are built from other widgets (`Container`, `ScrollBar`, `Alert`) are not registered.
- Calling `register` again, or registering one of these names afterward, replaces the constructor, so another library can restyle a standard widget for existing forms.
