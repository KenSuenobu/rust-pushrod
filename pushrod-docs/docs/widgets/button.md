---
sidebar_position: 4
title: Button
description: A push button with a text label that emits `Clicked`.
---

# Button

`Button` (module `pushrod_widgets::button`): a push button with a text label that emits `Clicked`.

![A Button, normal and pressed](/img/screenshots/button-states.png)

## Creating it

```rust
use pushrod::prelude::*;
use pushrod_widgets::prelude::*;

let button = engine.add_widget(
    Button::new("Click me")
        .at(20, 70)
        .size(120, 30)
        .font(Font::default().with_size(16)),
);

engine.on_click(button, |state, ui| {
    // runs on every click
});
```

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `Button::new(label)` | `impl Into<String>` | required | The label. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper-left corner, relative to the parent. |
| `size(width, height)` | `u32`, `u32` | `100, 24` | Width and height in pixels. |
| `font(font)` | `Font` | `Font::default()` | A [font handle](../concepts/text-and-fonts.md): family, size, and style. |
| `font_color(color)` | `Color` | black | Label color. |
| `justify(justification)` | `TextJustify` | `Center` | `Left`, `Center`, or `Right`. |
| `border_width(width)` | `u8` | `2` | Inset of the label from the edges, in pixels. |

## Methods

| Method | Does |
|---|---|
| `set_text(text)` / `get_text()` | The label. |
| `set_border_width(width)` / `get_border_width()` | Label inset. |
| `is_selected()` | `true` while drawn pressed. |
| `set_color(color)` | Background color when not pressed. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| `Event::Clicked { id, button, count }` | The left button is released over the button while it is still pressed. | `engine.on_click(id, ..)` |

## Notes

Behavior: pressing the mouse button inverts the button (black background, white text). Dragging off it restores the normal look and dragging back inverts it again. The click only counts if the mouse button is released while the button is still inverted.

## Planned

Default (Return) and cancel (Escape) buttons, toggle buttons, buttons that fire on press, per-state colors, and keyboard mnemonics are planned; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
