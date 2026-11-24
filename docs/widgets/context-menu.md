# Context Menu: widget roadmap

> **Issue:** [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) (2.19) · **Epic 2:** Core dialog widgets · **Release:** after the MVP · **Size:** S
> **User docs:** `pushrod-docs/docs/widgets/context-menu.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Right-click (or long-press) pop-up menus for a widget.

Commands that apply to one object (a list row, a text selection) belong in a menu that appears where the user clicked.

*Ideas from:* Contextual menus on the Mac, Windows, Qt, and GTK; fixed mouse-button roles on Intergraph workstations. Names and behavior are Pushrod's own.

## What it does

`widget.set_context_menu(menu)` or `on_context_menu(id, |state, ui, at| -> Option<Menu>)` to build it dynamically. Opens on secondary click, long-press, or the Menu key, at the pointer, in the overlay layer; reuses `Menu` ([2.11 Menu Bar and Menus](menu-bar-and-menus.md)) and actions ([9.5](https://github.com/KenSuenobu/rust-pushrod/issues/97)).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Right-click on a list row**

![2.19 step set 1: Right-click on a list row](../mockups/2.19-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `ContextMenuRequested { id, position }` | Secondary click / long-press / Menu key | `on_context_menu(id, ..)` |

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

- [ ] Opens on secondary click, long-press, and Menu key
- [ ] Dynamic menus via callback

## Depends on

Requires [2.11 Menu Bar and Menus](menu-bar-and-menus.md), [9.4](https://github.com/KenSuenobu/rust-pushrod/issues/96).

## Implementation notes

- **Modules:** `menu.rs`, `cache.rs`
- **Stack:** Rust
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [MAME wiki: Intergraph InterPro / CLIX / Environ V](https://wiki.mamedev.org/index.php/Driver:InterPro)
