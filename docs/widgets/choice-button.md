# Choice Button: widget roadmap

> **Issue:** [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) (3.6) · **Epic 3:** Value and choice controls · **Release:** after the MVP · **Size:** M
> **User docs:** `pushrod-docs/docs/widgets/choice-button.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Pick one of several options compactly, as a pop-up list or a click-to-cycle button.

Choosing one option from a list shouldn't always require radio buttons that take up space.

*Ideas from:* Pop-up buttons on NeXTSTEP and the Mac; option menus in Motif; cycle gadgets on AmigaOS; combo boxes in Qt and GTK. Names and behavior are Pushrod's own.

## What it does

`ChoiceButton { items, selected, mode }`. `Popup` mode opens an overlay list positioned so the current item sits under the pointer. `Cycle` mode advances on each click (Shift-click goes back) without a list: the most compact choice for two to five options. Emits `SelectionChanged`. **Parts ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57)):** `items()`, `add_item`, `remove_item`; the pop-up list's rows are parts. Every part can be retrieved and restyled (font, color, text).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Popup mode: 1. closed 2. open, current item under pointer 3. Large chosen**

![3.6 step set 1: Popup mode: 1. closed 2. open, current item under pointer 3. Large chosen](../mockups/3.6-1.png)

**Cycle mode: each click advances; Shift-click goes back**

![3.6 step set 2: Cycle mode: each click advances; Shift-click goes back](../mockups/3.6-2.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `SelectionChanged { id }` | Selection changes; read `selected()` | `on_selection_change(id, ..)` |

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

- [ ] Popup opens over the current item; selects by mouse/keyboard
- [ ] Cycle mode advances and reverses
- [ ] Parts are retrievable and modifiable as described

## Depends on

Requires [1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22), [5.1](https://github.com/KenSuenobu/rust-pushrod/issues/56), [5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57).

## Implementation notes

- **Crate:** `pushrod-widgets`, using only the public API of the `pushrod` core. Shared behavior (states, focus, events) comes from the core; anything the widget needs that the core doesn't expose is added to the core first.
- **Modules:** `pushrod-widgets/src/choice_button.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [AmigaOS GadTools autodoc (gadget kinds, underlined keyboard equivalents)](https://amigadev.grimore.org/Includes_and_Autodocs_2._guide/node05a3.html)
- [ICS: Motif widget descriptions](https://motif.ics.com/book/export/html/341)
- [OpenStep specification: controls and target/action](https://www.ict.griffith.edu.au/teaching/2501ICT/archive/resources/OpenStepSpec/ApplicationKit/Classes/NSControl.html)
