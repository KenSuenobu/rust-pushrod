# Wizard (step-by-step assistant): widget roadmap

> **Issue:** 6.4 (extra-widgets 6.4) · **Epic 6:** Dialogs and assistants · **Milestone:** Extra widgets 3: specialized views and assistants · **Size:** M
> **Crate:** `pushrod-extra-widgets` · **User docs:** `pushrod-docs/docs/extra-widgets/wizard.md` · **Overview:** [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, how it looks, and how it responds. To change it, edit this file in a pull request (and its mockup in [`docs/mockups/generate_extra_widgets.py`](../mockups/generate_extra_widgets.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Pages shown one at a time with Back, Next, and Finish, and a list of steps.

Setup, import, and export flows ask a few questions in order, and every toolkit ships the same assistant.

*Ideas from:* GTK GtkAssistant, Qt QWizard, KDE KAssistantDialog, Windows wizards, Mac OS X setup assistants. Names and behavior are Pushrod's own.

## What it does

`Wizard { pages }`: a modal dialog (Pushrod 6.7) with an optional step list on the left (current step bold, completed steps checked), the page title and the page in a stack layout (Pushrod 5.9), and Back, Next, Finish, and Cancel. Each page can block Next until it is complete (`is_complete`) and can skip the next page (`next_page`). Finish runs on the last page; a summary page is optional.

## How it looks

Each image shows the widget step by step; blue notes name the event emitted.

![6.4: Wizard (step-by-step assistant)](../mockups/xw-6.4-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners.

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `PageChanged { id, index }` | Back or Next | `on_page_change(id, ..)` |
| (completion) | Finish or Cancel | Completion closure receives finished or cancelled |

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

- [ ] Back, Next, Finish, and Cancel with page validation
- [ ] Step list and conditional pages
- [ ] Summary page optional
- [ ] Registered as `extra.wizard` and shown in the gallery (1.2)

## Depends on

Extra widgets: 1.1; Pushrod: Pushrod 5.9 [#120](https://github.com/KenSuenobu/rust-pushrod/issues/120) Stack layout (one child visible at a time), Pushrod 6.7 [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) Modal dialogs: run a form until a dismissing control is chosen, Pushrod 2.3 [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog

## Implementation notes

- **Crate:** `pushrod-extra-widgets`, using only the public APIs of `pushrod` and `pushrod-widgets`. Anything missing from the core is added to the core first.
- **Modules:** `pushrod-extra-widgets/src/wizard.rs (new)`, `pushrod-docs`
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
- **Documentation:** full doc headers on every non-private item, and the user page in the same pull request.

