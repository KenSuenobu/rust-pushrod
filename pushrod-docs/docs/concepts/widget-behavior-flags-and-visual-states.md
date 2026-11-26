---
sidebar_position: 3
title: Widget behavior flags and visual states
---

# Widget behavior flags and visual states

Every widget stores two sets of flags in its `WidgetCore`:

- `Behavior`: what the widget **can do**. Set by whoever builds the widget, and rarely changes.
- `VisualState`: how the widget **looks right now**. Changes as the user interacts.

Because the flags live on the widget itself, selection, default buttons, and disabled appearance work the same for every widget, and a form file can store them generically.

| `Behavior` (what it does) | `VisualState` (how it looks now) |
|---|---|
| `FOCUSABLE` | `PRESSED` (mouse button held on it; no flags = normal) |
| `TOGGLE` (stays pressed after a click) | `CHECKED` |
| `DEFAULT_ACTION` (Return) | `INDETERMINATE` (mixed / tri-state) |
| `CANCEL_ACTION` (Escape) | `DISABLED` |
| `DISMISSES_DIALOG` | `FOCUSED` |
| `ACTIVATE_ON_PRESS` (fires on press, not release) | `HOVERED` (pointer inside) |
| `EXCLUSIVE` (one-of-many among siblings) | `DRAG_OVER` (a drag is over it) |
| `HIDDEN` | |
| `OVERLAY` (not clipped to its parent) | |

:::note
This release stores the flags and acts on `HIDDEN`, `OVERLAY`, and `DISABLED`. The other flags are stored for the features that use them: focus, default and cancel buttons, dialogs, and radio groups.
:::

## Reading and changing flags

Every widget gets these methods from the `Widget` trait:

| Method | What it does |
|---|---|
| `behavior()` / `set_behavior(flags)` | Read or replace the behavior flags. |
| `state()` / `set_state(flags)` | Read or replace the visual state. Invalidates the widget only if it changed. |
| `is_enabled()` / `set_enabled(bool)` | Shortcut for the `DISABLED` state. |
| `is_hidden()` / `set_hidden(bool)` | Shortcut for the `HIDDEN` behavior. Existing code keeps working. |
| `is_focusable()` | `true` if `FOCUSABLE`, enabled, and not hidden. |

```rust checked title="src/main.rs"
use pushrod::geometry::{point, size};
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore};
use pushrod::paint::{self, Painter};

struct MyWidget {
    core: WidgetCore,
}

impl Widget for MyWidget {
    fn core(&self) -> &WidgetCore { &self.core }
    fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }
}

fn main() {
    let mut widget = MyWidget { core: WidgetCore::new(point(0, 0), size(100, 20)) };

    widget.set_behavior(Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION);
    widget.set_state(VisualState::CHECKED);
    widget.set_enabled(false);

    assert!(widget.state().contains(VisualState::DISABLED | VisualState::CHECKED));
    assert!(!widget.is_focusable());
}
```

## Disabled widgets

A disabled widget:

- receives no mouse events (no `PointerPressed`, `Clicked`, and so on). It still blocks widgets beneath it, so a click on it doesn't fall through;
- cannot take focus (`is_focusable()` is `false`);
- is drawn dimmed.

## The generic overlay

After a widget's `paint`, Pushrod calls `paint::state_overlay`, which draws the states that look the same on every widget:

| State | Overlay |
|---|---|
| `DISABLED` | A 50% white dither over the whole widget. |
| `FOCUSED` | A dotted outline, one pixel inside the edge. |
| `INDETERMINATE` | A dotted bar through the middle. |

The other states (`PRESSED`, `CHECKED`, `HOVERED`, `DRAG_OVER`) depend on what the widget is, so the widget draws them itself by reading `self.state()` or, better, the effective `self.style()` in `paint`. Call `set_state` when one changes so the widget is repainted. Applications can restyle every state: see [Per-state appearance](./per-state-appearance.md).

The overlay uses `Painter::dither_overlay`, which blends a color over every other pixel. A custom `Painter` backend must implement it.

## Try it

`pushrod-sdl2/examples/base.rs` shows a disabled button below the **Quit** button. Run it with `cargo run -p pushrod-sdl2 --example base`: the dimmed button does nothing when clicked.
