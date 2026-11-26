# Writing widgets and composites

This is the guide to extending Pushrod with your own widgets. The full text, with a sample for
every section, is the documentation site's
[Creating widgets](../pushrod-docs/docs/guides/creating-widgets.md) page. Its samples are marked
`rust checked`, so the `pushrod-doc-samples` crate compiles every one of them (and runs the test
samples) on each `cargo test --workspace`. This file is the map and the rules; the code is on that
page so it cannot go stale.

## Sections

| Section | What it shows | Sample |
|---|---|---|
| [Where the code goes](../pushrod-docs/docs/guides/creating-widgets.md#where-the-code-goes) | Core, `pushrod-widgets`, or your own widget library | (diagram) |
| [A paint-only widget](../pushrod-docs/docs/guides/creating-widgets.md#a-paint-only-widget) | `WidgetCore`, `paint`, invalidation | `Meter` |
| [An interactive widget with events and callbacks](../pushrod-docs/docs/guides/creating-widgets.md#an-interactive-widget-with-events-and-callbacks) | Event masks, `handle_event`, `ctx.emit`, `engine.on_click` | `SwatchWidget` |
| [Behavior and state](../pushrod-docs/docs/guides/creating-widgets.md#behavior-and-state) | `Behavior` and `VisualState`, one `paint` for every state | `Toggle` |
| [Using input controllers](../pushrod-docs/docs/guides/creating-widgets.md#using-input-controllers) | `add_controller`, `on_controller_event` | `SwatchWidget` with `ClickController` |
| [Composites](../pushrod-docs/docs/guides/creating-widgets.md#composites) | `Composite`, named parts, `handle_part_event` | `LabelledButton` |
| [Custom events](../pushrod-docs/docs/guides/creating-widgets.md#custom-events) | `ctx.post`, `engine.on_custom` | `Rating` |
| [Registry and properties](../pushrod-docs/docs/guides/creating-widgets.md#registry-and-properties) | Widgets created and edited by name | Arrives with #61 in v0.2 |
| [Testing a widget](../pushrod-docs/docs/guides/creating-widgets.md#testing-a-widget) | `Engine::dispatch`, `RecordingPainter`, golden images | `tests/swatch.rs`, `tests/meter_paint.rs` |

## Where the code goes

- **`pushrod` (the core):** behavior every widget can share, such as events, input controllers,
  `paint` helpers, and the `Widget` trait. It contains no widgets except the window's `RootWidget`.
- **`pushrod-widgets`:** the standard widgets, simple and flat, following
  [DESIGN_PHILOSOPHY.md](../DESIGN_PHILOSOPHY.md).
- **A separate widget library:** anything fancier (gradients, animation, charts, a whole new
  look) goes in its own crate that depends on `pushrod`, and on `pushrod-widgets` if it wants.

A widget uses only the core's public API, exactly as the standard widgets do. If a widget needs
something the core does not expose, open an issue for the core rather than working around it.

## Rules

- **Rendering:** all drawing goes through the `Painter` (SDL2 with the `pushrod-sdl2` backend) into
  the widget's own GPU texture, and only when the widget is invalidated. Frames are composed by
  blitting cached textures. No OS-native menus, dialogs, or controls; no per-frame CPU pixel work.
- **Events:** a widget reports what happened by emitting an event (`ctx.emit` or `ctx.post`),
  never by calling application code. Any internal action added to the core must also emit an
  engine event (`Event::Engine`, see the *Engine events* page).
- **Sample data:** examples, tests, mockups, and docs use fictional placeholders only, such as
  "Jane Doe", `jane.doe@example.com`, and "Anytown". No real names or personal information.
  See [CONTRIBUTING.md](../CONTRIBUTING.md).
- **Documentation:** every non-private function and type has a full doc header (summary,
  `# Arguments`, `# Returns`, and `# Panics` or `# Errors` where they apply).

## Checklist

- Store a `WidgetCore`; set an event mask or attach a controller if the widget handles events.
- Draw only inside `paint`, and call `set_invalidated(true)` whenever the look changes.
- Choose the look from `VisualState`, so one `paint` covers every state.
- Test with `Engine::dispatch` and a `RecordingPainter`; a standard widget also gets golden images
  in `pushrod-sdl2/tests/visual.rs`.
- Add a page for it under **Widgets** on the documentation site.
