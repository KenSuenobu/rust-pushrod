---
sidebar_position: 3
title: Widget libraries
---

# Widget libraries

Pushrod separates the core from the widgets:

| Crate | Role |
|---|---|
| `pushrod` | The core: engine, widget tree and IDs, events and callbacks, the `paint` helpers, and texture and font caches. The only widget it contains is the window's `RootWidget`. |
| `pushrod-widgets` | The standard widget set: simple, flat, and predictable, following the [design philosophy](../design-philosophy.md). |

`pushrod-widgets` uses only the core's public API. Anything it does, another crate can do too, so the look of an application isn't fixed by the core.

## Extending the standard widgets

Add a crate (or a module in your application) that depends on `pushrod` and `pushrod-widgets` and defines new widgets alongside the standard ones: a tree view, a graph view, a chart. They live in the same widget tree, use the same IDs, emit the same kinds of events, and work with the same callbacks. See [Creating widgets](../guides/creating-widgets.md).

## Replacing the standard widgets

A crate can depend on `pushrod` alone and provide its own complete widget set: drop shadows, gradients, rounded corners, animation, or 3D containers. The core doesn't care how a widget paints its texture. An application chooses which widget library to use in its `Cargo.toml`, and can mix libraries in one window.

The standard widgets stay simple on purpose. If you want something more graphically ambitious, build it as its own crate rather than changing `pushrod-widgets`.

## What a widget library relies on

- The `Widget` trait and `WidgetCore` (`pushrod::widget`)
- Events and the event context (`pushrod::event`)
- The `paint` helpers and direct SDL2 drawing into the widget's texture
- The font cache (`pushrod::font`)

If a widget library needs something the core doesn't expose, that's a core issue: open one, rather than working around it.
