---
sidebar_position: 2
title: Frame
description: A filled rectangle with a border of any width and color.
---

# Frame

`Frame` (module `pushrod_widgets::frame`): a filled rectangle with a border of any width and color.

![A Box widget with a 3-pixel black border](/img/screenshots/box.png)

## Creating it

```rust
use pushrod::prelude::*;
use pushrod_widgets::prelude::*;

let mut frame = Frame::new()
    .at(20, 20)
    .size(120, 80)
    .border_color(Color::BLACK)
    .border_width(3);
frame.set_color(Color::WHITE);
engine.add_widget(frame);
```

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `Frame::new()` | none | | A 100 by 100 frame at (0, 0) with a black one-pixel border. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper-left corner, relative to the parent. |
| `size(width, height)` | `u32`, `u32` | `100, 100` | Width and height in pixels. |
| `border_color(color)` | `Color` | black | Border color. |
| `border_width(width)` | `u8` | `1` | Border width in pixels; `0` draws no border. |

## Methods

| Method | Does |
|---|---|
| `set_border_color(color)` / `get_border_color()` | Border color. |
| `set_border_width(width)` / `get_border_width()` | Border width in pixels. |
| `set_color(color)` | Fill color inside the border. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (none) | This widget doesn't emit events. | n/a |

## Notes

Drawn as `paint::fill` followed by `paint::border`.

## Planned

A titled group box is planned; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
