---
sidebar_position: 4
title: Button
description: A push button with a text label that emits `Clicked`.
---

# Button

`ButtonWidget` (module `pushrod_widgets::button_widget`): a push button with a text label that emits `Clicked`.

![A Button, normal and pressed](/img/screenshots/button-states.png)

## Creating it

```rust
use pushrod_widgets::button_widget::ButtonWidget;
use pushrod::geometry::{point, size};
use pushrod_widgets::text_widget::TextJustify;
use sdl2::pixels::Color;
use sdl2::ttf::FontStyle;

let button = engine.add_widget(ButtonWidget::new(
    point(20, 70),
    size(120, 30),
    "assets/OpenSans-Regular.ttf".into(),
    FontStyle::NORMAL,
    16,
    Color::BLACK,
    TextJustify::Center,
    "Click me".into(),
    2,
));

engine.on_click(button, |state, ui| {
    // runs on every click
});
```

| Parameter | Type | Meaning |
|---|---|---|
| `origin` | `Point` | Upper-left corner, relative to the parent. |
| `size` | `Size` | Width and height in pixels. |
| `font_name` | `String` | Path to a TrueType font file. |
| `font_style` | `FontStyle` | `FontStyle::NORMAL`, `BOLD`, `ITALIC`, ... |
| `font_size` | `u16` | Size in points. |
| `font_color` | `Color` | Text color. |
| `justification` | `TextJustify` | `Left`, `Center`, or `Right`. |
| `msg` | `String` | The label. |
| `border_width` | `u8` | Inset of the label from the edges, in pixels. |

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
| `PushrodEvent::Clicked(id, clicks)` | The left button is released over the button while it is still pressed. | `engine.on_click(id, ..)` |

## Notes

Behavior: pressing the mouse button inverts the button (black background, white text). Dragging off it restores the normal look and dragging back inverts it again. The click only counts if the mouse button is released while the button is still inverted.

## Planned

Default (Return) and cancel (Escape) buttons, toggle buttons, buttons that fire on press, per-state colors, and keyboard mnemonics are planned; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
