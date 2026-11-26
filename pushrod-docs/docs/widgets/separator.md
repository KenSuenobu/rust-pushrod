---
sidebar_position: 7
title: Separator
description: A one pixel horizontal or vertical divider line, optionally etched.
---

# Separator

`Separator` (module `pushrod_widgets::separator`): a thin divider line for grouping dialog content. It is display only: it receives no input and emits no events.

![A Separator tile: a plain horizontal line, an etched horizontal line, a blue vertical line, and an etched vertical line](/img/screenshots/separator.png)

## Creating it

```rust checked title="src/dialog.rs"
use pushrod_widgets::separator::Separator;

/// A fictional dialog: a rule under the title and an etched divider between two columns.
pub fn dividers() -> (Separator, Separator) {
    let rule = Separator::horizontal(200).at(20, 40);
    let divider = Separator::vertical(80).etched().at(240, 20);
    (rule, divider)
}
```

Add it like any widget: `engine.add_widget(rule)`.

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `Separator::horizontal(width)` | `u32` | required | A line `width` pixels long and one pixel high. |
| `Separator::vertical(height)` | `u32` | required | A line `height` pixels long and one pixel wide. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper left-hand corner, relative to the parent. |
| `line_color(color)` | `Color` | dark gray | The line color. |
| `etched()` | none | off | Two pixels thick: the line beside a white line. |

## Methods

| Method | Does |
|---|---|
| `set_etched(etched)` | Turns the etched look on or off, resizes the widget to one or two pixels thick, and invalidates it. |
| `is_etched()` | Returns whether it is etched. |
| `is_horizontal()` | Returns `true` for a horizontal line. |
| `set_line_color(color)` | Sets the line color and invalidates the widget. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (none) | Display only | n/a |

The widget's event mask is empty. Like every widget it can still receive application-defined custom events in `handle_event`.

## Notes

- The line is drawn with the painter's one pixel `line`, so it is crisp at any length.
- An etched separator is the line color plus a white line; it looks best on a light background.
- For a bordered box, use [Frame](frame.md).

## Planned

Theming of the line colors (see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md)).
