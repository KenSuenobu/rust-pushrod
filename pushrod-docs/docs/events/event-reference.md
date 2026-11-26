---
sidebar_position: 1
title: Event Reference
description: Every Pushrod event, its fields, and when it is emitted.
---

# Event Reference

`pushrod::event::Event` is the one type for everything that happens in a Pushrod application. Each variant names what happened and carries named fields. Widgets receive events in `handle_event`, and applications receive the events widgets emit through [callbacks or `next_event`](../guides/handling-events.md).

```rust
use pushrod::event::{Event, EventKind};

fn describe(event: &Event) -> String {
    match event {
        Event::Clicked { id, count, .. } => format!("widget {} clicked {} time(s)", id, count),
        other => format!("{:?} on {:?}", other.kind(), other.widget()),
    }
}

assert_eq!(Event::CloseRequested.kind(), EventKind::CloseRequested);
assert_eq!(Event::CloseRequested.widget(), None);
```

Two methods are on every event:

| Method | Returns |
|---|---|
| `event.kind()` | The `EventKind`: the variant without its data. Used by masks and the callback registry. |
| `event.widget()` | `Option<WidgetId>`: the widget the event concerns. `None` for window, time, engine, custom, and raw events. |

Positions are `Point`s in window coordinates.

## Pointer

| Event | Emitted when |
|---|---|
| `PointerEntered { id }` | The pointer moves onto widget `id`. |
| `PointerExited { id }` | The pointer moves off widget `id`. |
| `PointerMoved { id, position }` | The pointer moves. Sent to the widget that captured the mouse, or the one under the pointer. |
| `PointerPressed { id, button, position, count }` | A mouse button goes down over widget `id`. The widget captures the mouse until release. |
| `PointerReleased { id, button, position, count }` | A mouse button goes up. Sent to the widget that captured the press. |
| `Clicked { id, button, count }` | A widget was pressed and released on itself. `count` is 1 for a single click, 2 for a double click. |
| `Scrolled { id, delta }` | The wheel or trackpad scrolls over widget `id`. |

## Keyboard

| Event | Emitted when |
|---|---|
| `KeyPressed { id, key, modifiers, repeat }` | A key goes down (`repeat` is `true` for auto-repeat). |
| `KeyReleased { id, key, modifiers }` | A key goes up. |
| `TextEntered { id, text }` | Text is typed. |

## Focus

| Event | Emitted when |
|---|---|
| `FocusGained { id }` | Widget `id` becomes the keyboard focus. |
| `FocusLost { id }` | Widget `id` stops being the keyboard focus. |

## Semantic (emitted by widgets)

| Event | Emitted when |
|---|---|
| `Activated { id }` | A widget is activated. |
| `Toggled { id, checked }` | A checkable widget changes state; `checked` is the new state. |
| `ValueChanged { id }` | A slider, spinner, or similar changes value. |
| `SelectionChanged { id }` | A selection changes. |
| `TextChanged { id }` | An editor's text changes. |
| `EditingFinished { id }` | Editing is confirmed or the editor loses focus. |
| `Triggered { id, action }` | A named action is triggered. |
| `ContextMenuRequested { id, position }` | The user asks for a context menu. |
| `PropertyChanged { id, name }` | A named property changes. |
| `ViewportChanged { id, offset }` | A scrolling widget's visible area moves. |

## Lifecycle

| Event | Emitted when |
|---|---|
| `Shown { id }` / `Hidden { id }` | A widget becomes visible / hidden. |
| `Resized { id, size }` | A widget changes size. |
| `Moved { id, origin }` | A widget changes origin. |
| `Removed { id }` | A widget is removed from the tree. |

## Window, time, engine, and extension

Window events are [broadcast](../concepts/broadcast-events-to-all-interested-widgets.md) rather than sent to one widget.

| Event | Emitted when |
|---|---|
| `WindowResized { size }` | The window changes size. Broadcast to widgets with `WINDOW` in their mask. |
| `WindowFocusChanged { focused }` | The window gains or loses focus. Broadcast to widgets with `WINDOW` in their mask. |
| `CloseRequested` | The user asks to close the window. |
| `Tick { elapsed }` | A frame passes while a widget animates: the widget asked for it with `ctx.request_animation()`. See [Timers and idle callbacks](../concepts/timers-and-idle-callbacks.md). |
| `Engine(EngineEvent)` | Pushrod performs an internal action. |
| `Custom(CustomEvent)` | An application or library defines its own event. See [Custom Events](./custom-events.md). |

:::note[What emits what today]
Pointer, key, and text events propagate: see [Event propagation](./event-propagation.md). The cache currently produces the pointer events (`PointerEntered`, `PointerExited`, `PointerMoved`, `PointerPressed`, `PointerReleased`), and `ButtonWidget` emits `Clicked`. The other variants are the shared vocabulary that later features (keyboard, focus, new widgets, timers) emit; `EngineEvent` currently has `EventMaskChanged { scope, old, new }` and `WidgetAdded { id, parent }`; see [Engine events](./engine-events.md).
:::

## Renamed from the first release

| Before | Now |
|---|---|
| `PushrodEvent::BoundsChange(from, to)` | `PointerExited { id: from }` then `PointerEntered { id: to }` |
| `PushrodEvent::Clicked(id, clicks)` | `Clicked { id, button, count }` |
| `PushrodEvent::SystemEvent(id, event)` | `PointerPressed`, `PointerReleased`, or `PointerMoved` (`Raw` for an unmapped SDL2 event) |
