---
sidebar_position: 8
title: Group Box
description: A titled frame that groups related controls, with the title in a gap in the top border.
---

# Group Box

`GroupBox` (module `pushrod_widgets::group_box`): a frame whose top border breaks at the left to show a title, used to group related controls in a dialog. It is display only: it receives no input and emits no events. Children are added to it like to any other widget.

![A Group Box tile: a frame titled Size holding a text line and a button](/img/screenshots/group-box.png)

## Creating it

```rust checked title="src/dialog.rs"
use pushrod_widgets::button::Button;
use pushrod_widgets::group_box::GroupBox;

/// A fictional dialog: a "Size" group holding an Apply button.
pub fn size_group() -> (GroupBox, Button) {
    let group = GroupBox::new("Size").at(20, 20).size(220, 100);
    let apply = Button::new("Apply").at(12, 24).size(120, 24);
    (group, apply)
}
```

Add the group first, then add each control with `engine.add_child_widget(group_id, control)`. Child origins are relative to the group box's upper left-hand corner.

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `GroupBox::new(title)` | `impl Into<String>` | required | The title shown in the top border; empty draws an unbroken border. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper left-hand corner, relative to the parent. |
| `size(w, h)` | `u32`, `u32` | `200, 100` | Width and height in pixels. |
| `font(font)` | `Font` | default font | The title's font. |
| `font_color(color)` | `Color` | black | The title's color. |
| `border_color(color)` | `Color` | black | The border color. |
| `border_width(width)` | `u8` | `1` | Border thickness in pixels; `0` draws no border. |

## Methods

| Method | Does |
|---|---|
| `set_title(title)` | Sets the title and invalidates the widget. |
| `get_title()` | Returns a copy of the title. |
| `set_border_color(color)` | Sets the border color and invalidates the widget. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (none) | Display only | n/a |

The widget's event mask is empty. Like every widget it can still receive application-defined custom events in `handle_event`.

## Notes

- The border runs along the vertical middle of the title's line and breaks where the title is drawn, with a few pixels of space on each side of the title.
- A title too long for the box is shortened with an ellipsis.
- The fill is transparent until set with `set_color`.
- For a plain bordered box with no title, use [Frame](frame.md).

## Planned

Theming of the border and title colors (see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md)).
