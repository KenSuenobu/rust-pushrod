# Spec: forms

Forms are widget trees run as dialogs. Today Pushrod defines the runtime half; the file half is
not built yet.

## What is specified now

- **Modal dialogs.** `Ui::open_modal` runs a widget tree as a dialog: widgets outside it are
  inert until it closes, and the engine emits `ModalOpened` and `ModalClosed` (see
  [events.md](events.md)). The guide is
  [Modal dialogs](../../pushrod-docs/docs/forms/modal-dialogs.md).
- **Form layout.** `FormLayout` arranges labels and fields in rows (see the
  [layouts](../../pushrod-docs/docs/layouts) pages).

## The form file format

*(not yet specified, not yet tested)* The binary form file format, its loader and writer, and the
generated ID constants are roadmap items 6.3 through 6.5. When the format lands, this spec links
to its definition and `tests/conformance/` gains a conformance test that loads a form written
with fictional sample data and checks the widget tree it produces.
