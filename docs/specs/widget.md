# Spec: widgets

A widget is a value that stores a `WidgetCore`, implements `Widget`, and lives in the
`WidgetCache` tree. Conformance tests: `tests/conformance/widget.rs`.

## Core

A widget **must**:

- return its shared `WidgetCore` from `core` and `core_mut`;
- implement `paint(&self, &mut dyn Painter)` using only the `Painter` API (see [paint.md](paint.md));
- handle events in `handle_event`, returning `Propagation::Continue` unless it consumed the
  event (see [events.md](events.md)).

A new widget starts **visible, enabled, in the normal visual state, not focusable, and
invalidated**, so the first frame paints it.

## Identity

- The cache hands out IDs in order: the root widget is `0`, the next widget is `1`, and so on.
- An ID is stable for the life of the widget.
- **A removed widget's ID is never reused.**
- `get_as::<T>(id)` returns the widget only if its concrete type is `T`.

## Behavior and visual state

`Behavior` flags (hidden, focusable, ...) say what a widget can do; `VisualState` flags
(hovered, pressed, focused, disabled, ...) say what it is doing. Changing the hidden flag
invalidates the widget. A disabled or hidden widget receives no input.

## Invalidation

A widget is painted into its cached surface **only when it is invalidated**. A frame in which no
widget is invalidated draws nothing and presents nothing. Calling `set_invalidated(true)` asks
for one repaint of that widget.

## Parts

A composite builds named parts in `Composite::build` with `PartsBuilder::add`.

- A part is an ordinary child widget with its own ID, in the order built.
- A part is found by name (`"sample"`) or by list (`"row"` finds `"row:0"`, `"row:1"`, ...).
- `typed_parts` (optional) gives the composite a typed view of its structure.

## Properties

*(not yet tested)* Property introspection arrives with the widget registry (roadmap item 6.2).
Until then a widget's configurable state is its methods and its `Appearance`.
