---
slug: /
sidebar_position: 1
title: Introduction
---

# Pushrod

Pushrod is a small, simple GUI library for Rust. It uses SDL2 for graphics and input, draws every widget itself into its own GPU texture, and composes each frame by copying those textures to the screen.

![A Pushrod window with a label and a button](/img/screenshots/quick-start.png)

## What it's like to use

- **Widgets live in a tree and are addressed by ID.** Adding a widget returns a `WidgetId`; callbacks and code refer to widgets by that ID, the way classic resource-file systems did. This is also what lets forms be designed visually and loaded from a file later.
- **Events are queued, then delivered to your code, in the style you prefer.** Widgets react to the mouse and keyboard and queue events such as `Clicked`. Pushrod then either calls the callbacks you registered (`on_click`, `on_event`) with mutable access to your application state and every widget, or hands the events to your own loop (`while let Some(event) = engine.next_event(&mut events)`).
- **Writing a widget is three functions.** A widget stores a `WidgetCore` and implements `core`, `core_mut`, and `paint`; interactive widgets also implement `handle_event`.
- **Everything is drawn by Pushrod.** No native menus, dialogs, or controls, so applications look and behave the same on every platform.

## Where to go next

- [Quick start](./getting-started/quick-start.mdx): a working application in about five minutes.
- [Creating an application](./guides/creating-an-application.md): the engine, application state, and the widget tree.
- [Handling events](./guides/handling-events.md): how events flow and how callbacks work.
- [Creating widgets](./guides/creating-widgets.md): write and test your own widget.
- [Widgets](/widgets): every built-in widget, one page each.

Pushrod is being modernized; see the [roadmap](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) for what's planned.
