---
sidebar_position: 3
title: Text
description: A single string of text on a filled background.
---

# Text

`Text` (module `pushrod_widgets::text`): a single string of text on a filled background.

![A Text widget showing Hello, Pushrod](/img/screenshots/text.png)

## Creating it

```rust
use pushrod::prelude::*;
use pushrod_widgets::prelude::*;

let label = engine.add_widget(
    Text::new("Hello, Pushrod")
        .at(20, 20)
        .size(140, 24)
        .font(Font::default().with_size(16))
        .justify(TextJustify::Center),
);
```

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `Text::new(text)` | `impl Into<String>` | required | The text to show. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper-left corner, relative to the parent. |
| `size(width, height)` | `u32`, `u32` | `200, 20` | Width and height in pixels. |
| `font(font)` | `Font` | `Font::default()` | A [font handle](../concepts/text-and-fonts.md): family, size, and style. |
| `font_color(color)` | `Color` | black | Text color. |
| `justify(justification)` | `TextJustify` | `Left` | `Left`, `Center`, or `Right`. |

## Methods

| Method | Does |
|---|---|
| `set_text(text)` / `get_text()` | The text shown. |
| `set_font(font)` / `get_font()` | The font; changing it repaints the text. |
| `set_font_color(color)` / `get_font_color()` | Text color. |
| `set_color(color)` | Background color (default transparent white). |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (none) | This widget doesn't emit events. | n/a |

## Notes

Text wraps at the widget's width and is centered vertically when it fits. Set the background color to match the parent when the label sits on a colored panel.

## Mnemonic

A `_` before a character (`Text::new("_Name")`) underlines it, and Alt plus that letter moves focus to the next focusable widget, which makes a label a shortcut to its field. See [Keyboard mnemonics](../concepts/keyboard-mnemonics.md).

## Planned

Text layout with full international support and a boxed-label variant are planned; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
