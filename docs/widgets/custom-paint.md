# Custom Paint: widget roadmap

> **Issue:** [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) (2.10) · **Epic 2:** Core dialog widgets · **Release:** MVP (v1) · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/custom-paint.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

One-off custom drawing without writing a widget struct.

A full `Widget` impl is a lot of ceremony for a one-off drawing (a chart, a preview).

*Ideas from:* Program-drawn objects in GEM; paint callbacks in Qt and GTK drawing areas. Names and behavior are Pushrod's own.

## What it does

`CustomPaint::new(origin, size, |canvas, size, state| ..)` with an optional event closure. The closure draws into the widget's GPU texture with the SDL2 canvas; it only runs when invalidated.

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Drawn entirely by the application's closure**

![2.10 step set 1: Drawn entirely by the application's closure](../mockups/2.10-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| (any) | Whatever the optional event closure emits or posts | `on_click`, `on_value_change`, `on_custom::<T>` as appropriate |

Like every widget, it can also receive application-defined custom events in `handle_event` and emit its own (1.13 (#78)).

### Shared behavior

Like every Pushrod widget, it gets these from the core rather than implementing them itself:

- behavior flags (selectable, default, exit, disabled, hidden, ...) and visual states ([1.1](https://github.com/KenSuenobu/rust-pushrod/issues/19))
- per-state appearance (normal, selected, hovered, pressed, drag-over, disabled), with `MouseEntered` / `MouseExited` ([1.17](https://github.com/KenSuenobu/rust-pushrod/issues/104))
- keyboard focus, Tab order, Return/Escape ([1.4](https://github.com/KenSuenobu/rust-pushrod/issues/20))
- callbacks registered by widget ID ([1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22))
- custom application events with payloads, received in `handle_event` ([1.13](https://github.com/KenSuenobu/rust-pushrod/issues/78))
- composite parts the application can retrieve and modify ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57))
- engine events for every internal action ([9.10](https://github.com/KenSuenobu/rust-pushrod/issues/109))
- live or outline drawing modes, switchable at run time ([10.1](https://github.com/KenSuenobu/rust-pushrod/issues/105))

## Acceptance criteria

- [ ] Closure paints into the texture
- [ ] Optional event closure receives events
- [ ] Example draws a chart

## Depends on

Requires PR [#10](https://github.com/KenSuenobu/rust-pushrod/pull/10).

## Implementation notes

- **Modules:** `custom_paint.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [GEM AES object structure (flags, states, object tree)](https://www.seasip.info/Gem/aestruct.html)
