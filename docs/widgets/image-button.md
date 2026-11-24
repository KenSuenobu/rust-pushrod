# Image Button: widget roadmap

> **Issue:** [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) (2.21) · **Epic 2:** Core dialog widgets · **Release:** after the MVP · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/image-button.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

A button that shows an image, or an image with a text label, with the full button behavior.

Toolbars, palettes, and compact dialogs need buttons that show a picture instead of, or next to, text.

*Ideas from:* Icon buttons in toolbars on every desktop system. Names and behavior are Pushrod's own.

## What it does

`ImageButton` with an image (from the image cache, [2.7 Image](image.md)) and optional label; `ImagePosition::{Only, Left, Right, Above, Below}`; optional separate images per state (normal, pressed, disabled, hovered), otherwise the pressed state inverts and disabled dithers like other widgets. Supports every button behavior (default, cancel, toggle, activate-on-press, [2.3 Button](button.md)) and mnemonics. **Parts ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57)):** `ImageButtonParts { image, label }`.

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Image only, image left of label, image above label; normal, pressed, toggled, disabled**

![2.21 step set 1: Image only, image left of label, image above label; normal, pressed, toggled, disabled](../mockups/2.21-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `Clicked { id, button, count }` | Released inside (or pressed, with activate-on-press) | `on_click(id, ..)` |
| `Toggled { id, checked }` | Toggle image buttons | `on_toggle(id, ..)` |

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

- [ ] Image only, and image with label in each position
- [ ] Per-state images or automatic pressed/disabled looks
- [ ] All button behaviors and mnemonics work

## Depends on

Requires [2.3 Button](button.md), [2.7 Image](image.md), [5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57).

## Implementation notes

- **Modules:** `image_button.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
