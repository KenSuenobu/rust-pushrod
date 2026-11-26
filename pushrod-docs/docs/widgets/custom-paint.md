---
sidebar_position: 5
title: Custom Paint
description: A widget drawn entirely by a closure, for a one-off chart or preview without writing a widget struct.
---

# Custom Paint

`CustomPaint` (module `pushrod_widgets::custom_paint`): a widget drawn entirely by a closure you supply. Use it for a one-off drawing (a chart, a preview) that does not deserve its own `Widget` implementation.

![A bar chart drawn by a Custom Paint closure, one bar highlighted in black](/img/screenshots/custom-paint.png)

## Creating it

```rust checked title="src/chart.rs"
use pushrod::color::Color;
use pushrod::event::{Event, Propagation};
use pushrod::geometry::{Rect, point, size};
use pushrod::paint;
use pushrod_widgets::custom_paint::CustomPaint;

/// A bar chart of fictional monthly totals; each press highlights the next bar.
pub fn chart() -> CustomPaint<(Vec<u32>, usize)> {
    CustomPaint::with_data(
        point(10, 10),
        size(240, 100),
        (vec![40, 65, 30, 80], 0),
        |p, size, (bars, selected)| {
            paint::fill(p, Color::WHITE);

            for (i, value) in bars.iter().enumerate() {
                let height = size.h * (*value).min(100) / 100;
                let color = if i == *selected { Color::BLACK } else { Color::BLUE };

                p.fill_rect(Rect::new(i as i32 * 60 + 8, (size.h - height) as i32, 44, height), color);
            }

            paint::border(p, size, Color::BLACK, 1);
        },
    )
    .on_event(|(bars, selected), event, _ctx| {
        if matches!(event, Event::PointerPressed { .. }) {
            *selected = (*selected + 1) % bars.len();
            return Propagation::Stop;
        }

        Propagation::Continue
    })
}
```

Add it like any widget: `engine.add_widget(chart())`.

The paint closure draws through the backend-neutral `Painter`, so the same closure works with every backend and with the headless test painter. It runs only when the widget is invalidated, into the widget's own texture; frames are composed by blitting that texture.

| Constructor or setter | Type | Default | Meaning |
|---|---|---|---|
| `CustomPaint::new(origin, size, paint)` | `Point`, `Size`, `Fn(&mut dyn Painter, Size, &())` | required | A widget with no data. |
| `CustomPaint::with_data(origin, size, data, paint)` | `Point`, `Size`, `S`, `Fn(&mut dyn Painter, Size, &S)` | required | A widget that carries `data`, which the closures share. |
| `on_event(closure)` | `FnMut(&mut S, &Event, &mut EventCtx) -> Propagation` | none | Receives the widget's events. Subscribes the widget to pointer, scroll, keyboard, text, and focus events. |
| `with_event_mask(mask)` | `EventMask` | pointer, scroll, keyboard, text, focus | Chooses which event classes the widget receives. Call it after `on_event`. |

## Methods

| Method | Does |
|---|---|
| `data()` | Returns the shared data. |
| `data_mut()` | Returns the shared data for modification and invalidates the widget so it repaints. |
| `set_data(data)` | Replaces the data and invalidates the widget. |

## Common methods

Every widget also has the methods of the `Widget` trait: `get_id`, `get_parent`, `get_origin`/`set_origin`, `get_size`/`set_size`, `get_color`/`set_color`, `is_hidden`/`set_hidden`, and `is_invalidated`/`set_invalidated`. See [Creating widgets](../guides/creating-widgets.md).

## Events and callbacks

| Event | Emitted when | Callback |
|---|---|---|
| (any) | Whatever the event closure emits with `ctx.emit` or posts with `ctx.post` | `on_click`, `on_press`, `on_custom::<T>`, and so on, as appropriate |

Without an event closure the widget receives no events and emits none. With one, the widget is invalidated after every event it handles, so the paint closure always shows what the closure changed.

## Notes

- To change what is drawn from outside the widget, call `data_mut()` or `set_data(...)` (or `set_invalidated(true)` if the closure reads something else); the paint closure does not run again until the widget is invalidated.
- Draw in the standard Pushrod style: `paint::fill`, `paint::border`, `paint::text`, and the `Painter` fill, rectangle, and line calls. Colors come from the closure; theming is not part of this widget.
- Closures must be `'static`: move or clone what they need into them.
- This is the same technique as [Creating widgets](../guides/creating-widgets.md), minus the struct; reach for a full `Widget` implementation when the drawing needs its own methods or is reused.

## Planned

Per-state appearance and behavior flags apply to every widget; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md).
