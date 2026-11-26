---
sidebar_position: 1
title: Backend-neutral paint API
---

# Backend-neutral paint API

A widget's `paint` never touches SDL2. It draws through Pushrod's own `Painter` trait, so a widget library works with any rendering backend and never compiles against a specific SDL version.

```rust
fn paint(&self, p: &mut dyn Painter) {
    paint::fill(p, self.get_color());
    paint::border(p, self.get_size(), Color::BLACK, 1);
}
```

`Color`, `FontStyle`, `Point`, `Size` and `Rect` are Pushrod types (`pushrod::color`, `pushrod::geometry`), not SDL types.

## The `Painter` trait

Coordinates are relative to the widget's upper left-hand corner.

| Method | Does |
|---|---|
| `fill(color)` | Fills the whole surface. |
| `fill_rect(bounds, color)` | Fills a rectangle. |
| `rect(bounds, color)` | Draws a one-pixel rectangle outline. |
| `line(from, to, color)` | Draws a one-pixel line. |
| `text(bounds, style, color, msg) -> Size` | Draws wrapped, justified text and returns the size drawn. |
| `image(path, bounds)` | Draws an image file scaled to `bounds`; returns an error if it cannot be loaded. |
| `dither(bounds, first, second)` | Fills `bounds` with a 50% checkerboard of two colors. |
| `clip(bounds)` | Limits later drawing to `bounds`, or `None` to remove the limit. |
| `cache_policy()` | The backend's `CachePolicy`. |

The helpers `paint::fill`, `paint::border` and `paint::text` build on these, so widgets look the same on every backend.

## Backends

| Backend | Where | Does |
|---|---|---|
| `Sdl2Painter` | `pushrod_sdl2` | Each widget paints into its own GPU texture when invalidated; frames are blits of cached textures. See [Backends](./backends.md). |
| `RecordingPainter` | `pushrod::backend::headless` | Draws nothing and records every call as a `PaintCall`. For unit tests with no window. |

## Cache policy

A backend declares how painted widgets are kept between frames:

| `CachePolicy` | Meaning |
|---|---|
| `TextureCache` | Each widget has its own cached surface, repainted only when invalidated. The SDL2 backend uses this. |
| `DirectDraw` | Invalidated widgets are painted straight to the screen with clipping, for memory-poor systems. The policy is declared, but the SDL2 path does not render it. |

`WidgetCache::cache_policy()` returns the policy in use.

## Testing a widget without a window

```rust
use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::widget::Widget;
use pushrod_widgets::panel::Panel;

let mut panel = Panel::new().size(10, 10);
panel.set_color(Color::GRAY);

let mut p = RecordingPainter::default();
panel.paint(&mut p);
assert_eq!(p.calls, vec![PaintCall::Fill(Color::GRAY)]);
```

See also [Drawing](../concepts/drawing.md) and [Creating widgets](../guides/creating-widgets.md).
