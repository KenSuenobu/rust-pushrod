# Progress Bar: widget roadmap

> **Issue:** [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) (4.1) · **Epic 4:** Lists, tables, progress, spin boxes, and tabs · **Release:** MVP (v1) · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/progress-bar.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

A bar showing how far along a task is; with no value it shows a busy animation (like HTML's progress element).

Long operations need visible progress.

## What it does

`ProgressBar { max, value: Option<f64> }` with the same semantics as the HTML `<progress>` element: a value between 0 and `max` shows a determinate bar; no value (`set_indeterminate()` or `value: None`) shows the indeterminate busy animation (a block moving back and forth, driven by timer ticks, [1.5](https://github.com/KenSuenobu/rust-pushrod/issues/21); reduced to a static striped bar in outline drawing mode, [10.1](https://github.com/KenSuenobu/rust-pushrod/issues/105)). Display only.

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**0%, 40%, busy**

![4.1 step set 1: 0%, 40%, busy](../mockups/4.1-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| (none) | Display only; set with `set_value()` (often from a custom event, [1.13](https://github.com/KenSuenobu/rust-pushrod/issues/78)) | n/a |

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

- [ ] Determinate fill proportional to value / max
- [ ] `None` value shows the indeterminate animation without callbacks
- [ ] Static in outline drawing mode

## Depends on

Requires [1.5](https://github.com/KenSuenobu/rust-pushrod/issues/21).

## Implementation notes

- **Modules:** `progress_bar.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
