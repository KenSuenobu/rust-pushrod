# Date picker: widget roadmap

> **Issue:** [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) (extra-widgets 3.2) · **Epic 3:** Pickers and special fields · **Milestone:** Extra widgets 2: everyday controls and pickers · **Size:** S
> **Crate:** `pushrod-extra-widgets` · **User docs:** `pushrod-docs/docs/extra-widgets/date-picker.md` · **Overview:** [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, how it looks, and how it responds. To change it, edit this file in a pull request (and its mockup in [`docs/mockups/generate_extra_widgets.py`](../mockups/generate_extra_widgets.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

A date field with a drop-down month calendar.

Typing dates is error-prone and formats differ; a calendar makes the choice clear. Moved here from the standard set because it is a compound of a field, a popover, and a calendar.

*Ideas from:* Qt QDateEdit, KDE KDateComboBox, Win32 date and time picker, GTK date entries, the Mac OS 8 clock control. Names and behavior are Pushrod's own.

## What it does

`DatePicker { value, format, min, max }`: a masked text field (Pushrod 2.6) plus a glyph button that opens a popover (1.3) holding a calendar (3.1). Typing a valid date updates the calendar; choosing a day fills the field and closes the popover. Up/Down in the field step the part under the caret (day, month, year).

## How it looks

Each image shows the widget step by step; blue notes name the event emitted.

![3.2: Date picker](../mockups/xw-3.2-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners.

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `ValueChanged { id, date }` | A valid date typed or chosen | `on_value_change(id, ..)` |

Like every Pushrod widget, it can also receive application-defined custom events in `handle_event` and emit its own (Pushrod 1.13), and every internal action emits an engine event (Pushrod 9.10).

### Shared behavior

Like every Pushrod widget, it gets these from the core rather than implementing them itself:

- behavior flags and visual states ([Pushrod 1.1](https://github.com/KenSuenobu/rust-pushrod/issues/19))
- per-state appearance ([Pushrod 1.17](https://github.com/KenSuenobu/rust-pushrod/issues/104))
- keyboard focus, Tab order, Return/Escape ([Pushrod 1.4](https://github.com/KenSuenobu/rust-pushrod/issues/20))
- callbacks registered by widget ID ([Pushrod 1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22))
- custom application events with payloads ([Pushrod 1.13](https://github.com/KenSuenobu/rust-pushrod/issues/78))
- composite parts the application can retrieve and modify ([Pushrod 5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57))
- engine events for every internal action ([Pushrod 9.10](https://github.com/KenSuenobu/rust-pushrod/issues/109))

## Acceptance criteria

- [ ] Typed and chosen dates stay in step
- [ ] Min/max enforced
- [ ] Up/Down steps the date part under the caret
- [ ] Registered as `extra.date_picker` with a page in the Pushrod Showcase (1.2)

## Depends on

Extra widgets: 1.3 ([#254](https://github.com/KenSuenobu/rust-pushrod/issues/254)), 3.1 ([#260](https://github.com/KenSuenobu/rust-pushrod/issues/260)); Pushrod: Pushrod 2.6 [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) Text field: single-line entry with input masks, validation, and numeric mode, Pushrod 2.2 [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) Glyph button (small single-character button)

## Implementation notes

- **Crate:** `pushrod-extra-widgets`, using only the public APIs of `pushrod` and `pushrod-widgets`. Anything missing from the core is added to the core first.
- **Modules:** `pushrod-extra-widgets/src/date_picker.rs (new)`, `pushrod-docs`
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
- **Documentation:** full doc headers on every non-private item, and the user page in the same pull request.

