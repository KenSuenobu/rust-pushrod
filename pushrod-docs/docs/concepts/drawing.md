---
sidebar_position: 2
title: Drawing
---

# Drawing

Pushrod draws everything itself through SDL2 and keeps the work on the GPU:

1. Each widget has its own GPU texture.
2. When a widget is invalidated, Pushrod calls its `paint` once, drawing into that texture.
3. Each frame where something changed, Pushrod clears the screen and copies every visible widget's texture to it in tree order, then presents the frame. Copying cached textures is cheap; repainting only happens for widgets that changed.
4. If nothing changed, nothing is drawn and the previous frame stays on screen.

This is why widgets never draw directly to the screen and why changing a widget only requires marking it invalidated.

## Rules for widget authors

- Draw only inside `paint`, from `(0, 0)` to `get_size()`.
- Use the `paint` helpers (`fill`, `border`, `text`) so widgets share one look: flat, with no gradients, bevels, shadows, or rounded corners ([design philosophy](../design-philosophy.md)).
- Pushrod never uses operating-system menus, dialogs, or controls: everything on screen is a Pushrod widget, so applications look the same everywhere.
