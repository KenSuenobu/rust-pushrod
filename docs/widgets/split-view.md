# Split View: widget roadmap

> **Issue:** [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) (3.9) · **Epic 3:** Value and choice controls · **Release:** after the MVP · **Size:** M
> **User docs:** `pushrod-docs/docs/widgets/split-view.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Two or more panes side by side or stacked, separated by dividers the user drags to resize them.

List-and-detail screens, editors with a sidebar, and browsers with a preview need panes the user can resize.

*Ideas from:* Split views on NeXTSTEP and the Mac, paned windows in Motif, splitters in Qt and GTK. Names and behavior are Pushrod's own.

## What it does

`SplitView` (horizontal or vertical) holding two or more panes; each pane is a container for child widgets (often a Container, [5.3 Container](container.md)). Dividers are drawn between panes; dragging one resizes its neighbors, respecting each pane's minimum size and optional maximum; double-clicking a divider collapses the smaller pane (if collapsible) and restores it. When the split view is resized, extra space goes to panes by stretch factor (default: the last pane). Divider positions can be set (`set_position(divider, px)`), read, and saved by the application. Dragging follows the drawing mode ([10.1](https://github.com/KenSuenobu/rust-pushrod/issues/105)): live repaint or an outline until release. **Parts ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57)):** `panes()`, `dividers()`.

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Dragging the divider between a list and a detail pane**

![3.9 step set 1: Dragging the divider between a list and a detail pane](../mockups/3.9-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `ValueChanged { id }` | A divider moved (on release in outline mode); read `positions()` | `on_value_change(id, ..)` |

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

- [ ] Dragging a divider resizes neighboring panes within their min/max
- [ ] Collapsible panes; double-click to collapse/restore
- [ ] Extra space distributed by stretch factor
- [ ] Positions settable and readable; outline mode supported

## Depends on

Requires [5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57), [5.5](https://github.com/KenSuenobu/rust-pushrod/issues/116), [9.4](https://github.com/KenSuenobu/rust-pushrod/issues/96).

## Implementation notes

- **Crate:** `pushrod-widgets`, using only the public API of the `pushrod` core. Shared behavior (states, focus, events) comes from the core; anything the widget needs that the core doesn't expose is added to the core first.
- **Modules:** `pushrod-widgets/src/split_view.rs (new)`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [OpenStep specification: controls and target/action](https://www.ict.griffith.edu.au/teaching/2501ICT/archive/resources/OpenStepSpec/ApplicationKit/Classes/NSControl.html)
- [ICS: Motif widget descriptions](https://motif.ics.com/book/export/html/341)
