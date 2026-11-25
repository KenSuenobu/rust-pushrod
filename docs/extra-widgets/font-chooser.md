# Font chooser: widget roadmap

> **Issue:** [#129](https://github.com/KenSuenobu/rust-pushrod/issues/129) (extra-widgets 3.5) · **Epic 3:** Pickers and special fields · **Milestone:** Extra widgets 2: everyday controls and pickers · **Size:** M
> **Crate:** `pushrod-extra-widgets` · **User docs:** `pushrod-docs/docs/extra-widgets/font-chooser.md` · **Overview:** [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md) · **Rules:** [design philosophy](../../DESIGN_PHILOSOPHY.md)

This file is the specification for this widget: what it is, what it does, how it looks, and how it responds. To change it, edit this file in a pull request (and its mockup in [`docs/mockups/generate_extra_widgets.py`](../mockups/generate_extra_widgets.py) if the picture changes), then bring the issue in line. If this file and the issue disagree, this file wins.

## What it is

A font button that opens a chooser with family, style, size, and a preview.

Editors and settings screens let users pick a font. Originally parked because most applications don't; it belongs in the extra widgets.

*Ideas from:* GTK GtkFontButton and GtkFontChooser, Qt QFontDialog and QFontComboBox, KDE KFontRequester and KFontChooser, AmigaOS getfont.gadget, GEM's font selector (fnts), Motif XmFontSelector. Names and behavior are Pushrod's own.

## What it does

`FontButton { font }` shows the font's name in that font. Clicking opens a popover (1.3) with a `FontChooser`: a filter field, a family list (each name in its own face), a style list, a size field with common sizes, and an editable preview line. Families come from Pushrod's font registry (Pushrod 1.24). `FontChooser` can be used in a window or a modal dialog directly.

## How it looks

Each image shows the widget step by step; blue notes name the event emitted.

![3.5: Font chooser](../mockups/xw-3.5-1.png)

Drawn flat, in the standard Pushrod style: fills, one-pixel borders, and text; inverted when pressed or selected, dithered when disabled, a dotted outline when focused. No bevels, gradients, shadows, glass, or rounded corners.

## How it responds

### Events and callbacks

| Event emitted | When | Application callback |
|---|---|---|
| `FontChanged { id, font }` | Family, style, or size chosen | `on_font_change(id, ..)` |

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

- [ ] Family, style, and size lists with filtering
- [ ] Preview text editable
- [ ] Popover and dialog use
- [ ] Registered as `extra.font_chooser` and shown in the gallery (1.2)

## Depends on

Extra widgets: 1.3; Pushrod: Pushrod 4.5 [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) List view, Pushrod 1.24 [#137](https://github.com/KenSuenobu/rust-pushrod/issues/137) Fonts: shared font handles and a built-in default font, Pushrod 2.6 [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) Text field: single-line entry with input masks, validation, and numeric mode, Pushrod 6.7 [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) Modal dialogs: run a form until a dismissing control is chosen

## Implementation notes

- **Crate:** `pushrod-extra-widgets`, using only the public APIs of `pushrod` and `pushrod-widgets`. Anything missing from the core is added to the core first.
- **Modules:** `pushrod-extra-widgets/src/font_chooser.rs (new)`, `pushrod-docs`
- **Drawing:** paints into its own GPU texture only when invalidated; frames are composed by blitting. No OS-native controls; no per-frame CPU pixel work.
- **Documentation:** full doc headers on every non-private item, and the user page in the same pull request.

