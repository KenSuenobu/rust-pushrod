# Extra widgets gallery example: widget roadmap

> **Issue:** [#253](https://github.com/KenSuenobu/rust-pushrod/issues/253) (extra-widgets 1.2) · **Epic 1:** Crate and foundations · **Milestone:** Extra widgets 1: crate and Chassis widgets · **Size:** S
> **Crate:** `pushrod-extra-widgets` · **User docs:** `pushrod-docs/docs/extra-widgets/gallery.md` · **Overview:** [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, how it looks, and how it responds. To change it, edit this file in a pull request (and its mockup in [`docs/mockups/generate_extra_widgets.py`](../mockups/generate_extra_widgets.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

A runnable gallery showing every extra widget, used for golden screenshots.

Each widget needs a place to be seen, tried, and screenshotted, and contributors need a quick way to check their work.

*Ideas from:* Qt's widget gallery example and GTK's widget factory. Names and behavior are Pushrod's own.

## What it does

`cargo run -p pushrod-extra-widgets --example gallery`: one page per epic (tab view), each widget in its common states with its events logged in a status line. Golden-image tests (Pushrod 7.2) capture each page. Every new widget adds itself to the gallery in the same PR.

## How it looks

Each image shows the widget step by step; blue notes name the event emitted.

![1.2: Extra widgets gallery example](../mockups/xw-1.2-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners.

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| (per widget) | Any interaction | Logged in the gallery's status line |

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

- [ ] Gallery runs and shows every widget built so far
- [ ] Golden images captured in CI
- [ ] Contributing guide says new widgets must be added to the gallery
- [ ] Documented on the docs site

## Depends on

Extra widgets: 1.1 ([#252](https://github.com/KenSuenobu/rust-pushrod/issues/252)); Pushrod: Pushrod 7.2 [#70](https://github.com/KenSuenobu/rust-pushrod/issues/70) Golden-image rendering tests, Pushrod 7.3 [#71](https://github.com/KenSuenobu/rust-pushrod/issues/71) Widget gallery example, Pushrod 4.4 [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) Tab view

## Implementation notes

- **Crate:** `pushrod-extra-widgets`, using only the public APIs of `pushrod` and `pushrod-widgets`. Anything missing from the core is added to the core first.
- **Modules:** `pushrod-extra-widgets/examples/gallery.rs (new)`, `pushrod-docs`
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
- **Documentation:** full doc headers on every non-private item, and the user page in the same pull request.

