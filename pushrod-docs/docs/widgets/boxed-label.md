---
sidebar_position: 6
title: Boxed Label
description: Display-only text centered in a filled box with a border.
---

# Boxed Label

`BoxedLabel` (module `pushrod_widgets::boxed_label`): text centered in a filled box with a border, for dialogs and status areas. It is display only: it receives no input and emits no events.

![Two Boxed Labels: Ready with a cyan fill and blue border, and a disabled Offline label](/img/screenshots/boxed-label.png)

## Creating it

```rust checked title="src/status.rs"
use pushrod::color::Color;
use pushrod::widget::Widget;
use pushrod_widgets::boxed_label::BoxedLabel;

/// A fictional status label: cyan fill, blue border.
pub fn status() -> BoxedLabel {
    let mut label = BoxedLabel::new("Ready")
        .at(20, 20)
        .size(120, 24)
        .border_color(Color::BLUE);
    label.set_color(Color::CYAN);
    label
}
```

Add it like any widget: `engine.add_widget(status())`.

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `BoxedLabel::new(text)` | `impl Into<String>` | required | The text to show. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper left-hand corner, relative to the parent. |
| `size(width, height)` | `u32`, `u32` | `120, 24` | Width and height in pixels. |
| `font(font)` | `Font` | engine default | The font. |
| `font_color(color)` | `Color` | black | The text color. |
| `border_color(color)` | `Color` | black | The border color. |
| `border_width(width)` | `u8` | `1` | Border width in pixels; `0` draws no border. |
| `set_color(color)` | `Color` | transparent | The fill color (from `Widget`). |

## Methods

| Method | Does |
|---|---|
| `set_text(text)` | Replaces the text and invalidates the widget. |
| `get_text()` | Returns a copy of the text. |
| `set_border_color(color)` | Sets the border color and invalidates the widget. |
| `set_font_color(color)` | Sets the text color and invalidates the widget. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (none) | Display only | n/a |

The widget's event mask is empty. Like every widget it can still receive application-defined custom events in `handle_event`.

## Notes

- Text is centered inside the border. Text wider than the box is shortened with an ellipsis.
- Drawn flat: fill, centered text, then a border. Disabled labels are dithered like every widget.
- For text without a border, or with other alignment, wrapping, or truncation, use [Text](text.md).

## Planned

Wrapped and selectable text (see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md)).
