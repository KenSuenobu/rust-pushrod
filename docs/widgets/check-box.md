# Check Box: widget roadmap

> **Issue:** [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) (2.4) · **Epic 2:** Core dialog widgets · **Release:** MVP (v1) · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/check-box.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Box with a check mark and label; toggles checked; optional mixed state.

A first-class check box is expected everywhere, including a 'mixed' state when it summarizes several items.

*Ideas from:* Check boxes on every system; tri-state check boxes in Qt, GTK, and Windows. Names and behavior are Pushrod's own.

## What it does

Add `CheckBox`: square + check mark when `CHECKED`, a dash when `INDETERMINATE`, followed by a label. Clicking the box or label toggles and emits `Toggled`; Space toggles when focused; clicking an indeterminate box makes it checked.

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**1. Unchecked 2. Pressed 3. Checked 4. Indeterminate 5. Focused 6. Disabled**

![2.4 step set 1: 1. Unchecked 2. Pressed 3. Checked 4. Indeterminate 5. Focused 6. Disabled](../mockups/2.4-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `Toggled { id, checked }` | Click, Space, or mnemonic | `on_toggle(id, ..)` |
| `FocusGained / FocusLost { id }` | Keyboard focus moves | `on_focus_change(id, ..)` |

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

- [ ] Click, Space, and mnemonic toggle
- [ ] Indeterminate renders and resolves on click
- [ ] Label click toggles

## Depends on

Requires [1.1](https://github.com/KenSuenobu/rust-pushrod/issues/19), [1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22).

## Implementation notes

- **Crate:** `pushrod-widgets`, using only the public API of the `pushrod` core. Shared behavior (states, focus, events) comes from the core; anything the widget needs that the core doesn't expose is added to the core first.
- **Modules:** `pushrod-widgets/src/check_box.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
