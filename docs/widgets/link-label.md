# Link Label: widget roadmap

> **Issue:** [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) (2.22) · **Epic 2:** Core dialog widgets · **Release:** after the MVP · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/link-label.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Clickable text that looks like a link and opens a URL or runs a callback.

'Learn more', 'Forgot password?', and similar secondary actions read better as links than as buttons.

*Ideas from:* Link labels in Windows, Qt, and GTK. Names and behavior are Pushrod's own.

## What it does

`LinkLabel { text, target: Option<String> }`: underlined text in the link color, pointer cursor ([9.9](https://github.com/KenSuenobu/rust-pushrod/issues/101)) on hover, focusable with a dotted focus outline, activated by click, Return, or Space. With a URL target it opens the system browser (SDL's open-URL support) after the application's `on_activate` callback has had a chance to cancel; without a target it only emits `Activated`. Visited state optional. Appearance per state through [1.17](https://github.com/KenSuenobu/rust-pushrod/issues/104).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Normal, hovered, focused, visited**

![2.22 step set 1: Normal, hovered, focused, visited](../mockups/2.22-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `Activated { id }` | Clicked, or Return/Space when focused | `on_activate(id, ..)` |

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

- [ ] Click, Return, and Space activate
- [ ] URL targets open the system browser unless the callback cancels
- [ ] Hover cursor and focus outline

## Depends on

Requires [1.4](https://github.com/KenSuenobu/rust-pushrod/issues/20), [1.6](https://github.com/KenSuenobu/rust-pushrod/issues/22).

## Implementation notes

- **Modules:** `link_label.rs (new)`
- **Stack:** Rust, SDL2 (open URL)
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
