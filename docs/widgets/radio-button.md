# Radio Button: widget roadmap

> **Issue:** [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) (2.5) · **Epic 2:** Core dialog widgets · **Release:** MVP (v1) · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/radio-button.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

One-of-many selection among sibling widgets.

Choosing exactly one of a few options is everywhere; selecting one radio button must clear its siblings, which needs core support.

*Ideas from:* Radio button groups on every system. Names and behavior are Pushrod's own.

## What it does

Add `RadioButton` (circle + label) with the `EXCLUSIVE` behavior. When one becomes checked, exclusive siblings under the same parent are cleared and `Toggled` is emitted for the newly checked one (and `Toggled(false)` for the one cleared). `RadioGroup::new(labels)` builds a labeled group in one call; `ui.checked_in_group(parent)`. Arrow keys move within the group ([1.4](https://github.com/KenSuenobu/rust-pushrod/issues/20)).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**1. Medium checked 2. Pressing Large 3. Released: Large checked, Medium cleared**

![2.5 step set 1: 1. Medium checked 2. Pressing Large 3. Released: Large checked, Medium cleared](../mockups/2.5-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `Toggled { id, checked }` | A radio becomes checked or is cleared | `on_toggle(id, ..)`; `ui.checked_in_group(parent)` |

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

- [ ] Exactly one exclusive sibling is checked after any change
- [ ] Works for any widget with `EXCLUSIVE`
- [ ] Arrow keys move within the group

## Depends on

Requires [1.1](https://github.com/KenSuenobu/rust-pushrod/issues/19), [1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22).

## Implementation notes

- **Modules:** `radio_button.rs (new)`, `cache.rs`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [AmigaOS GadTools autodoc (gadget kinds, underlined keyboard equivalents)](https://amigadev.grimore.org/Includes_and_Autodocs_2._guide/node05a3.html)
