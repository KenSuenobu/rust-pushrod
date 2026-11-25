---
sidebar_position: 2
title: Box
description: A filled rectangle with a border of any width and color.
---

# Box

`BoxWidget` (module `pushrod_widgets::box_widget`): a filled rectangle with a border of any width and color.

![A Box widget with a 3-pixel black border](/img/screenshots/box.png)

## Creating it

```rust
use pushrod_widgets::box_widget::BoxWidget;
use pushrod::geometry::{point, size};
use pushrod::widget::Widget;
use pushrod::color::Color;

let mut frame = BoxWidget::new(point(20, 20), size(120, 80), Color::BLACK, 3);
frame.set_color(Color::WHITE);
engine.add_widget(frame);
```

| Parameter | Type | Meaning |
|---|---|---|
| `origin` | `Point` | Upper-left corner, relative to the parent. |
| `size` | `Size` | Width and height in pixels. |
| `border_color` | `Color` | Border color. |
| `border_width` | `u8` | Border width in pixels; `0` draws no border. |

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
