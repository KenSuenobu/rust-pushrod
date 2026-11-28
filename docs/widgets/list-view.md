# List View: widget roadmap

> **Issue:** [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) (4.5) · **Epic 4:** Lists, tables, progress, spin boxes, and tabs · **Release:** after the MVP · **Size:** L
> **User docs:** `pushrod-docs/docs/widgets/list-view.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Scrolling list with single/multiple selection, backed by a data source.

Lists are needed by file choosers, combo boxes, and most applications.

*Ideas from:* Scrolling lists on AmigaOS, Motif, the Mac, Qt, and GTK; item models in Qt and GTK. Names and behavior are Pushrod's own.

## What it does

`ListView` in a container, rows from a `ListModel` trait (`len`, `item(i)`, change notifications so the view updates when the model changes); only visible rows painted. Single, multiple, and range selection; keyboard navigation and type-ahead; double-click or Return activates. **Parts ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57)):** `rows()` (visible row parts), per-row style via a `row_style` callback for model-driven rows. Every part can be retrieved and restyled (font, color, text). The `ListModel` reports changes as ranges (`inserted(range)`, `removed(range)`, `changed(range)`, `reset`) so the view updates incrementally, and row widgets are recycled as the list scrolls (virtualization).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**1. Idle 2. Click selects 3. Shift-click extends 4. Double-click activates**

![4.5 step set 1: 1. Idle 2. Click selects 3. Shift-click extends 4. Double-click activates](../mockups/4.5-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `SelectionChanged { id }` | Selection changes; read `selected_rows()` | `on_selection_change(id, ..)` |
| `Activated { id }` | Double-click or Return | `on_activate(id, ..)` |

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

- [x] Smooth with 10,000 rows
- [x] Selection modes and keyboard navigation
- [x] Model change notifications update the view
- [x] Parts are retrievable and modifiable as described
- [x] Range-based model notifications; row widgets recycled while scrolling

## Depends on

Requires [1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22), [5.3 Container](container.md), [5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57).

## Implementation notes

- **Crate:** `pushrod-widgets`, using only the public API of the `pushrod` core. Shared behavior (states, focus, events) comes from the core; anything the widget needs that the core doesn't expose is added to the core first.
- **Modules:** `pushrod-widgets/src/list_view.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [AmigaOS GadTools autodoc (gadget kinds, underlined keyboard equivalents)](https://amigadev.grimore.org/Includes_and_Autodocs_2._guide/node05a3.html)
- [ICS: Motif widget descriptions](https://motif.ics.com/book/export/html/341)
