---
sidebar_position: 9
title: Image
description: Shows a PNG or BMP file centered, stretched, scaled to fit, or tiled.
---

# Image

`ImageWidget` (module `pushrod_widgets::image_widget`): shows a bitmap from a file. PNG and BMP load (so does anything else `SDL2_image` reads). It is display only unless you call `clickable()`, which makes it emit `Clicked`.

![An Image tile: the same sample picture drawn Center, Stretch, Fit, and Tile, and a crossed box for a missing file](/img/screenshots/image.png)

## Creating it

```rust checked title="src/dialog.rs"
use pushrod_widgets::image_widget::{FitMode, ImageWidget};

/// A fictional dialog: a logo scaled to fit, and a clickable tiled banner.
pub fn pictures() -> (ImageWidget, ImageWidget) {
    let logo = ImageWidget::new("assets/logo.png")
        .at(20, 20)
        .size(160, 120)
        .fit(FitMode::Fit);
    let banner = ImageWidget::new("assets/banner.png")
        .at(200, 20)
        .size(240, 60)
        .fit(FitMode::Tile)
        .clickable();
    (logo, banner)
}
```

Add it like any widget: `engine.add_widget(logo)`.

| Setter | Type | Default | Meaning |
|---|---|---|---|
| `ImageWidget::new(path)` | `impl Into<String>` | required | The image file. It is read when the widget is first painted, not when it is created. |
| `at(x, y)` | `i32`, `i32` | `0, 0` | Upper left-hand corner, relative to the parent. |
| `size(width, height)` | `u32`, `u32` | `100, 100` | The widget's size. |
| `fit(mode)` | `FitMode` | `Center` | How the image is placed (below). |
| `clickable()` | none | off | Emit `Clicked` when clicked. |
| `missing_color(color)` | `Color` | black | Color of the crossed box drawn for a missing file. |

The background is transparent until set with `set_color`.

## Fit modes

| `FitMode` | Draws |
|---|---|
| `Center` | The image at its own size, centered. Parts that do not fit are cut off. |
| `Stretch` | Scaled to the whole widget, ignoring the aspect ratio. |
| `Fit` | Scaled to the largest size that fits, keeping the aspect ratio, centered. |
| `Tile` | Repeated at its own size from the upper left-hand corner. Edge tiles are cut off. At most 4096 tiles are drawn. |

## Methods

| Method | Does |
|---|---|
| `set_path(path)` | Changes the file and invalidates the widget. |
| `path()` | Returns the file path. |
| `set_fit(mode)` / `fit_mode()` | Changes or returns the fit mode; changing invalidates the widget. |
| `is_clickable()` | Returns whether it emits `Clicked`. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| `Clicked { id, button, count }` | Only if made `clickable()`: the mouse button is pressed and released on it | `on_click(id, ..)` |

Like every widget it can also receive application-defined custom events in `handle_event`.

## Notes

- **One load per file.** The SDL2 backend decodes each file once into a GPU texture kept in a cache keyed by path; every widget showing that path, and every tile, copies that texture.
- **Missing or unreadable files do not panic.** The widget draws a box with both diagonals, and the backend reports the failure once as an error event.
- A file that failed to load is tried again the next time the widget paints, so it appears once the file exists.
- A cached image is not reloaded if the file changes on disk while the application runs.
- Backends implement `Painter::image_size` and `Painter::image_part` to support this widget; the test `RecordingPainter` records them and takes its image sizes from its `images` map.

## Planned

Icons and image buttons build on this widget (see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md)).
