# Spec: forms

Forms are widget trees run as dialogs. Pushrod defines the runtime half and the file format.

## What is specified now

- **Modal dialogs.** `Ui::open_modal` runs a widget tree as a dialog: widgets outside it are
  inert until it closes, and the engine emits `ModalOpened` and `ModalClosed` (see
  [events.md](events.md)). The guide is
  [Modal dialogs](../../pushrod-docs/docs/forms/modal-dialogs.md).
- **Form layout.** `FormLayout` arranges labels and fields in rows (see the
  [layouts](../../pushrod-docs/docs/layouts) pages).

## The form file format

The binary form file format is defined in [`docs/FORM_FORMAT.md`](../FORM_FORMAT.md) and
implemented by `pushrod::form` (`FormFile::to_bytes` and `FormFile::from_bytes`).

- A form file **must** start with the magic `PRSC` and a `u16` version, and a reader **must**
  reject a newer version than it supports.
- Objects **must** be referred to by number (their index in the object table), and a reader
  **must** reject a file whose links, indexes, or offsets are inconsistent instead of panicking.
- A reader **must** skip property tags it does not know, so new properties do not break old
  readers.
- `Engine::load_form` builds a form through the widget registry, all or nothing, errors name the
  object and property, and it emits `FormLoaded { form, root, widgets }`.
- `FormWriter` saves a widget subtree as a form: geometry, flags, properties, `Column` and `Row`
  layouts, and child layout parameters; composite parts are not written.
- Generated ID constants are roadmap item 6.6.

The guide is [Form file format](../../pushrod-docs/docs/forms/form-file-format.md).
Conformance: `tests/conformance/forms.rs`.
