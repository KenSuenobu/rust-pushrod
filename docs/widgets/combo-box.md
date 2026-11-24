# Combo Box: widget roadmap

> **Issue:** [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) (4.6) · **Epic 4:** Lists, tables, progress, spin boxes, and tabs · **Release:** after the MVP · **Size:** M
> **User docs:** `pushrod-docs/docs/widgets/combo-box.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Text field with a drop-down list of suggestions.

Choosing common values while allowing free entry needs a combo box.

*Ideas from:* Combo boxes in Motif, Windows, Qt, and GTK. Names and behavior are Pushrod's own.

## What it does

Composite of `TextField` + arrow + overlay `ListView`; typing filters; choosing fills the field. **Parts ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57)):** `ComboBoxParts { field, button, list }`; `add_item`, `remove_item`. Every part can be retrieved and restyled (font, color, text).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**1. Idle 2. Typing filters 3. Chosen**

![4.6 step set 1: 1. Idle 2. Typing filters 3. Chosen](../mockups/4.6-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `TextChanged { id }` | Typing | `on_text_change(id, ..)` |
| `SelectionChanged { id }` | Item chosen from the list | `on_selection_change(id, ..)` |

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

- [ ] Typing filters; arrow opens; choosing fills
- [ ] Parts are retrievable and modifiable as described

## Depends on

Requires [2.6 Text Field](text-field.md), [4.5 List View](list-view.md), [5.1](https://github.com/KenSuenobu/rust-pushrod/issues/56), [5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57).

## Implementation notes

- **Modules:** `combo_box.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [ICS: Motif widget descriptions](https://motif.ics.com/book/export/html/341)
