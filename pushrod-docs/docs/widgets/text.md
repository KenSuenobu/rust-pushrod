---
sidebar_position: 3
title: Text
description: A single string of text on a filled background.
---

# Text

`TextWidget` (module `pushrod_widgets::text_widget`): a single string of text on a filled background.

![A Text widget showing Hello, Pushrod](/img/screenshots/text.png)

## Creating it

```rust
use pushrod::geometry::{point, size};
use pushrod_widgets::text_widget::{TextJustify, TextWidget};
use pushrod::color::{Color, FontStyle};

let label = engine.add_widget(TextWidget::new(
    point(20, 20),
    size(140, 24),
    "assets/OpenSans-Regular.ttf".into(),
    FontStyle::NORMAL,
    16,
    Color::BLACK,
    TextJustify::Center,
    "Hello, Pushrod".into(),
));
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
| `msg` | `String` | The text to show. |

## Methods

| Method | Does |
|---|---|
| `set_text(text)` / `get_text()` | The text shown. |
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

## Planned

Text layout with full international support, mnemonics, and a boxed-label variant are planned; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
