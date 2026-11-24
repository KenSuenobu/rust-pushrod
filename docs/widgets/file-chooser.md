# File Chooser: widget roadmap

> **Issue:** [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) (2.12) · **Epic 2:** Core dialog widgets · **Release:** after the MVP · **Size:** L
> **User docs:** `pushrod-docs/docs/widgets/file-chooser.md` · **Overview:** [main roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, and how it responds. To change its look or behavior, edit this file in a pull request (and its mockup in [`docs/mockups/generate.py`](../mockups/generate.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

Choose a file or directory with a consistent look on every platform.

Applications need to open and save files; native dialogs would break the consistent look.

*Ideas from:* File selectors on TOS, Motif, AmigaOS, Qt, and GTK. Names and behavior are Pushrod's own.

## What it does

Modal form built from the list view ([4.5 List View](list-view.md)), text fields, and buttons: directory, filter, file list, name, OK/Cancel; Open and Save modes. Completion closure receives `Option<PathBuf>`. OS-native dialogs are deliberately not used. **Parts ([5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57)):** `FileChooserParts { folder, filter, list, name, buttons }`; `add_button` (for example 'New Folder'), `add_filter_choice`. Every part can be retrieved and restyled (font, color, text).

## How it looks

Each image shows the widget step by step, drawn in Pushrod's current style; blue notes name the event emitted. Mockups come from [`docs/mockups/generate.py`](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/mockups/generate.py): edit a frame there and re-run it to change the design,.

**Open dialog**

![2.12 step set 1: Open dialog](../mockups/2.12-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners (see the [design philosophy](../../DESIGN_PHILOSOPHY.md)).

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| (completion) | Open or Cancel | Completion closure `FnOnce(&mut S, &mut Ui, Option<PathBuf>)` |

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

- [ ] Navigate, filter, choose; keyboard operable
- [ ] Returns None on cancel
- [ ] Parts are retrievable and modifiable as described

## Depends on

Requires [2.6 Text Field](text-field.md), [4.5 List View](list-view.md), [6.7](https://github.com/KenSuenobu/rust-pushrod/issues/66), [5.2](https://github.com/KenSuenobu/rust-pushrod/issues/57).

## Implementation notes

- **Modules:** `file_chooser.rs (new)`
- **Stack:** Rust std::fs
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.

## Sources

- [ICS: Motif widget descriptions](https://motif.ics.com/book/export/html/341)
