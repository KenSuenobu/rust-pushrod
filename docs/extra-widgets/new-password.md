# New password (enter, confirm, strength): widget roadmap

> **Issue:** 3.8 (extra-widgets 3.8) · **Epic 3:** Pickers and special fields · **Milestone:** Extra widgets 3: specialized views and assistants · **Size:** S
> **Crate:** `pushrod-extra-widgets` · **User docs:** `pushrod-docs/docs/extra-widgets/new-password.md` · **Overview:** [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, how it looks, and how it responds. To change it, edit this file in a pull request (and its mockup in [`docs/mockups/generate_extra_widgets.py`](../mockups/generate_extra_widgets.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Two password fields with a strength meter and a match check.

Creating a password needs the same three things everywhere: hidden entry, confirmation, and feedback on strength, before the OK button is enabled.

*Ideas from:* KDE KNewPasswordWidget, the macOS password assistant, account setup screens in GTK and Windows. Names and behavior are Pushrod's own.

## What it does

`NewPassword { min_length, rules }`: password and confirm fields (Pushrod 2.6, password mode) with a show/hide toggle, a level indicator (2.5) for strength, and a line of text for the first unmet rule or a mismatch. `is_acceptable()` is true only when rules pass and both fields match; dialogs bind OK's enabled state to it. Strength is a simple, documented score (length and character classes), replaceable by a closure.

## How it looks

Each image shows the widget step by step; blue notes name the event emitted.

![3.8: New password (enter, confirm, strength)](../mockups/xw-3.8-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners.

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `PasswordChanged { id, acceptable }` | Either field edited | `on_password_change(id, ..)` |

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

- [ ] Strength meter and rule messages
- [ ] Match check and show/hide
- [ ] Acceptable state bindable to OK
- [ ] Registered as `extra.new_password` and shown in the gallery (1.2)

## Depends on

Extra widgets: 2.5 ([#54](https://github.com/KenSuenobu/rust-pushrod/issues/54)); Pushrod: Pushrod 2.6 [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) Text field: single-line entry with input masks, validation, and numeric mode, Pushrod 9.6 [#98](https://github.com/KenSuenobu/rust-pushrod/issues/98) Property bindings: keep two properties in sync

## Implementation notes

- **Crate:** `pushrod-extra-widgets`, using only the public APIs of `pushrod` and `pushrod-widgets`. Anything missing from the core is added to the core first.
- **Modules:** `pushrod-extra-widgets/src/new_password.rs (new)`, `pushrod-docs`
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
- **Documentation:** full doc headers on every non-private item, and the user page in the same pull request.

