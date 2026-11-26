---
sidebar_position: 1
title: Panel
description: A rectangle filled with one color. Use it as a background or a panel that holds other widgets.
---

# Panel

`Panel` (module `pushrod_widgets::panel`): a rectangle filled with one color. Use it as a background or a panel that holds other widgets.

![A Base widget filled with blue](/img/screenshots/base.png)

## Creating it

```rust
use pushrod::prelude::*;
use pushrod_widgets::prelude::*;

let mut panel = Panel::new().at(20, 20).size(120, 80);
panel.set_color(Color::RGB(70, 130, 200));
let panel_id = engine.add_widget(panel);
```

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `Panel::new()` | none | | A transparent 100 by 100 panel at (0, 0). |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper-left corner, relative to the parent. |
| `size(width, height)` | `u32`, `u32` | `100, 100` | Width and height in pixels. |

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

The window's root widget (ID `0`) is the core's `RootWidget`, not a `Panel`; change the window color with `engine.set_background(color)`. Add other widgets as its children with `engine.add_child_widget(panel_id, widget)` to group and move them together.

## Planned

Per-state appearance and behavior flags apply to every widget; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
