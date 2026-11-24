---
sidebar_position: 2
title: Handling events
---

# Handling events

## How events flow

```text
SDL2 event ──> WidgetCache ──> Widget::handle_event(&mut self, &event, ctx)
                                      │
                                ctx.emit(Clicked(..))
                                      v
                              event queue (in order)
                                      │   (no widget is borrowed any more)
                                      v
                  your callbacks: on_click(id, ..), on_event(..)
                     FnMut(&mut AppState, &mut Ui [, &event])
```

1. Pushrod reads events from SDL2 and routes each one to a widget:
   - **Mouse button down** goes to the top-most visible widget under the pointer, which then *captures* the mouse until the button is released.
   - **Mouse motion** sends `BoundsChange(left, entered)` to the widget the pointer left and the one it entered, then the motion itself to the captured widget (or the one under the pointer).
   - **Mouse button up** goes to the captured widget, then releases the capture.
2. The widget updates itself and may queue events with `ctx.emit(...)`. A button, for example, emits `Clicked` when released while still pressed.
3. After routing, Pushrod delivers each queued event to your callbacks, in order. Because no widget is borrowed at that point, callbacks can freely change your state and any widget.

## Events

| Event | Sent when |
|---|---|
| `PushrodEvent::BoundsChange(left, entered)` | The pointer moves from one widget to another (to both widgets). |
| `PushrodEvent::Clicked(id, clicks)` | A widget was clicked; `clicks` is 1 for a single click, 2 for a double click. Emitted by buttons and by your own widgets. |
| `PushrodEvent::SystemEvent(id, event)` | A raw SDL2 event routed to widget `id`. |

:::note[Planned changes]
Event names are being modernized (for example `PointerEntered`/`PointerExited` instead of `BoundsChange`), and propagation, keyboard focus, and more callbacks are on the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md). This page will change with them.
:::

## Callbacks

| Method | Runs when | Signature |
|---|---|---|
| `engine.on_click(id, f)` | Widget `id` emits `Clicked` | `FnMut(&mut S, &mut Ui)` |
| `engine.on_event(f)` | Any widget emits any event | `FnMut(&mut S, &mut Ui, &PushrodEvent)` |

Several callbacks can be registered for the same widget; they run in the order they were registered, and `on_event` callbacks run after the widget-specific ones.

## The `Ui` handle

| Method | Does |
|---|---|
| `ui.get::<T>(id)` | Returns `Option<&T>` for the widget with that ID, if it is a `T`. |
| `ui.get_mut::<T>(id)` | Returns `Option<&mut T>`; use it to change a widget (`set_text`, `set_color`, ...). |
| `ui.widgets()` | The whole `WidgetCache`: add widgets, or reach a widget without knowing its type (`get_mut(id)`). |
| `ui.shutdown()` | Stops the engine after the current frame. |

```rust
engine.on_click(save, move |state, ui| {
    state.saves += 1;
    ui.widgets().get_mut(indicator).unwrap().set_color(Color::RGB(70, 160, 70));
});

engine.on_click(quit, |_, ui| ui.shutdown());

engine.on_event(|_, _, event| {
    if let PushrodEvent::Clicked(id, clicks) = event {
        println!("widget {} clicked ({} click(s))", id, clicks);
    }
});
```

## Emitting events from a widget

Inside `handle_event`, call `ctx.emit(...)` to queue an event for the callbacks. See [Creating widgets](./creating-widgets.md).

## Testing without a window

`Engine::dispatch(event, &mut state)` processes one SDL2 event exactly as the run loop would, including callbacks, so you can test event handling without opening a window. See [Testing a widget](./creating-widgets.md#testing-a-widget).
