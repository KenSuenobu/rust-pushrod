# Tooltip: widget roadmap

> **Issue:** [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) (2.13) · **Epic 2:** Core dialog widgets · **Release:** after the MVP · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/tooltip.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Short help text shown after the pointer rests on a widget.

Icon buttons and terse labels are hard to learn without explanation.

*Ideas from:* Balloon Help on the classic Mac; tooltips in Windows, Qt, and GTK. Names and behavior are Pushrod's own.

## What it does

Optional `tooltip` text on `WidgetCore` (actions supply it automatically). After a delay (600 ms), draw a bordered text box in the overlay layer near the pointer; hide it on movement, click, or key press. Emits its engine event ([9.10](https://github.com/KenSuenobu/rust-pushrod/issues/109)).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**1. Pointer rests 2. After 600 ms 3. Pointer moves: hidden**

![2.13 step set 1: 1. Pointer rests 2. After 600 ms 3. Pointer moves: hidden](../mockups/2.13-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| (none) | Purely visual | n/a |

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

- [ ] Appears after the delay and hides correctly
- [ ] Stays inside the window; never under the pointer
- [ ] Emits the corresponding `EngineEvent` ([9.10](https://github.com/KenSuenobu/rust-pushrod/issues/109))

## Depends on

Requires [1.5](https://github.com/KenSuenobu/rust-pushrod/issues/21), [5.1](https://github.com/KenSuenobu/rust-pushrod/issues/56).

## Implementation notes

- **Modules:** `cache.rs`, `tooltip.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
