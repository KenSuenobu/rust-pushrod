# Glyph Button: widget roadmap

> **Issue:** [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) (2.2) · **Epic 2:** Core dialog widgets · **Release:** MVP (v1) · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/glyph-button.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

A small box with an arrow or symbol, used for scroll arrows, close boxes, and spin buttons.

Scroll bars, spin boxes, and title gadgets need a small button showing one symbol, often repeating while held.

*Ideas from:* Character boxes in GEM; arrow buttons in Motif. Names and behavior are Pushrod's own.

## What it does

Add `GlyphButton { glyph: char }`: fill + centered glyph + border; inverted while pressed. With `ACTIVATE_ON_PRESS` it fires on press and auto-repeats while held (400 ms delay, then every 100 ms, via timers).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**1. Idle 2. Pressed (fires immediately) 3. Held: repeats 4. Released 5. Disabled**

![2.2 step set 1: 1. Idle 2. Pressed (fires immediately) 3. Held: repeats 4. Released 5. Disabled](../mockups/2.2-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `Clicked { id }` | On press, then on each auto-repeat | `on_click(id, ..)` |

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

- [ ] Centered glyph
- [ ] Pressed state inverts
- [ ] Activate-on-press with auto-repeat

## Depends on

Requires [1.1](https://github.com/KenSuenobu/rust-pushrod/issues/19), [1.5](https://github.com/KenSuenobu/rust-pushrod/issues/21).

## Implementation notes

- **Modules:** `glyph_button.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [GEM AES object structure (flags, states, object tree)](https://www.seasip.info/Gem/aestruct.html)
- [ICS: Motif widget descriptions](https://motif.ics.com/book/export/html/341)
