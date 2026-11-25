---
sidebar_position: 4
title: Event masks for the run loop and for widgets
---

# Event masks for the run loop and for widgets

An **event mask** lists the kinds of events something wants. Pushrod keeps two:

- The **run-loop mask** on the `Engine` filters input for the whole window.
- Each **widget mask** filters what that widget receives.

An event is handled only if its class is in **both** masks. Everything else is dropped early, so an application that does not need an event class never pays for it.

## Event classes

| `EventMask` class | Events |
|---|---|
| `POINTER_BUTTONS` | `PointerPressed`, `PointerReleased` |
| `POINTER_MOTION` | `PointerMoved` |
| `HOVER` | `PointerEntered`, `PointerExited` |
| `SCROLL` | `Scrolled` |
| `KEYBOARD` | `KeyPressed`, `KeyReleased` |
| `TEXT` | `TextEntered` |
| `FOCUS` | `FocusGained`, `FocusLost` |
| `WINDOW` | `WindowResized`, `WindowFocusChanged` |
| `LIFECYCLE` | `Shown`, `Hidden`, `Resized`, `Moved`, `Removed`, `PropertyChanged`, `ViewportChanged` |
| `TIMER` | `Tick` |
| `CUSTOM` | `Custom`, `Raw` |

`EventMask::NONE` and `EventMask::ALL` are the empty and full sets; combine classes with `|`, and remove one with `& !`. The events a widget *emits* for your application (`Clicked`, `Activated`, `ValueChanged`, ...), `CloseRequested`, and `Engine(..)` events are never filtered.

## The run-loop mask

`Engine::set_event_mask(mask)` (or `ui.set_event_mask(mask)` inside a callback) changes the mask at any time; it applies from the next event. The default is `EventMask::ALL`.

- Events outside the mask are dropped **before** routing and callbacks.
- SDL event types that feed only masked-out classes are **disabled in SDL** (`EventPump::disable_event`), so SDL never queues them, and re-enabled when the mask changes. Pointer motion stays enabled while either `POINTER_MOTION` or `HOVER` is in the mask, because hover comes from motion.
- **Close requests are always delivered**, and window exposure and resizing keep repainting the window whatever the mask says.
- With `TIMER` out of the mask and nothing to repaint, the loop **blocks** waiting for input instead of polling, so an idle application uses almost no CPU.

```rust checked title="src/main.rs"
use pushrod::engine::Engine;
use pushrod::event::EventMask;
use pushrod::geometry::size;

fn main() {
    let mut engine: Engine = Engine::new(size(320, 200), 60);

    // No hover effects in this window: stop pointer motion at the source.
    engine.set_event_mask(EventMask::ALL & !EventMask::POINTER_MOTION & !EventMask::HOVER);

    assert!(!engine.event_mask().intersects(EventMask::HOVER));
}
```

## The widget mask

`Widget::event_mask()` returns the classes a widget wants. The default is `EventMask::NONE`, so a `BoxWidget` or `BaseWidget` receives nothing, while a `ButtonWidget` asks for `POINTER_BUTTONS | HOVER` and still clicks. A widget that handles events sets its mask when it is built:

```rust checked title="src/main.rs"
use pushrod::event::{Event, EventCtx, EventMask};
use pushrod::geometry::{point, size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};

struct Clickable {
    core: WidgetCore,
}

impl Clickable {
    fn new() -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(100, 20));

        core.set_event_mask(EventMask::POINTER_BUTTONS);

        Self { core }
    }
}

impl Widget for Clickable {
    fn core(&self) -> &WidgetCore { &self.core }
    fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventCtx) {}
}

fn main() {
    assert_eq!(Clickable::new().event_mask(), EventMask::POINTER_BUTTONS);
}
```

To change a mask once the widget is in the window, call `WidgetCache::set_event_mask(id, mask)` (`engine.widgets()`) or `ui.set_widget_event_mask(id, mask)`.

:::note
A widget with an empty mask still counts as the target under the pointer, so it blocks the widgets beneath it, as before.
:::

## Mask change events

Changing either mask emits `Event::Engine(EngineEvent::EventMaskChanged { scope, old, new })`, once per real change. `scope` is `MaskScope::RunLoop` or `MaskScope::Widget(id)`. It is delivered to your callbacks (or returned by `next_event`) like any other event.

```text
SDL2 ──(types disabled at source for masked classes)──> event pump
                                │
        run-loop mask ── not in mask: dropped
                                │
                      routing to the widget
                                │
        widget mask ── not in mask: skipped
                                │
   Widget::handle_event -> queue -> callbacks
```
