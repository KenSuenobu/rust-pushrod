---
sidebar_position: 1
title: Base
description: A rectangle filled with one color. Use it as a background or a panel that holds other widgets.
---

# Base

`BaseWidget` (module `pushrod_widgets::base_widget`): a rectangle filled with one color. Use it as a background or a panel that holds other widgets.

![A Base widget filled with blue](/img/screenshots/base.png)

## Creating it

```rust
use pushrod_widgets::base_widget::BaseWidget;
use pushrod::geometry::{point, size};
use pushrod::widget::Widget;
use pushrod::color::Color;

let mut panel = BaseWidget::new(point(20, 20), size(120, 80));
panel.set_color(Color::RGB(70, 130, 200));
let panel_id = engine.add_widget(panel);
```

| Parameter | Type | Meaning |
|---|---|---|
| `origin` | `Point` | Upper-left corner, relative to the parent. |
| `size` | `Size` | Width and height in pixels. |

## Methods

| Method | Does |
|---|---|
| `set_color(color)` | Sets the fill color (the default is transparent white). |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (none) | This widget doesn't emit events. | n/a |

## Notes

The window's root widget (ID `0`) is the core's `RootWidget`, not a `BaseWidget`; change the window color with `engine.set_background(color)`. Add other widgets as its children with `engine.add_child_widget(panel_id, widget)` to group and move them together.

## Planned

Per-state appearance and behavior flags apply to every widget; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
