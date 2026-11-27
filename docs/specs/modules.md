# Spec: module boundaries

Pushrod keeps the **engine** apart from the **design layer**, so look and feel can change without
touching how the tree, events, invalidation, and painting work.

## The engine

The engine is the part every widget library and backend depends on. In the `pushrod` crate:

| Module | Owns |
|---|---|
| `engine`, `cache` (and `cache/`) | The widget tree, IDs, routing, invalidation, drawing order, callbacks |
| `event`, `input`, `timer` | The event vocabulary, raw input, timers |
| `paint`, `backend` | The `Painter` and `Backend` traits and the cache policy |
| `geometry`, `color`, `font`, `text`, `layout`, `composite`, `controllers`, `error` | Shared value types, text, layout, parts, and input controllers |

## The design layer

The design layer decides how things look by default.

- `appearance` (in `pushrod`): per-state styles (`StateStyle`, `Appearance`) that widgets read
  through `Widget::style`.
- `pushrod-widgets`: the standard widgets, their defaults, and their themes.

## Rules

1. **The core depends on no backend and no widget library.** `pushrod` has no `sdl2`,
   `pushrod-sdl2`, or `pushrod-widgets` dependency, and its code never names them.
2. **The engine modules never use the design layer.** `engine`, `cache`, `event`, `input`,
   `timer`, and `backend` do not reference `appearance`. Only `widget` (which stores the
   `Appearance` on `WidgetCore`) connects the two.
3. **The standard widgets depend on the core only.** `pushrod-widgets` uses the public API of
   `pushrod`, so any other widget library can do everything it does, and it never reaches a
   backend.
4. **Backends implement `Backend`.** A backend crate (such as `pushrod-sdl2`) depends on the core
   and implements `Backend` and `Painter`; the engine never knows which one it runs on.

```text
application ──> pushrod-widgets ──> pushrod (engine + design data)
      └────────> pushrod-sdl2 ─────> pushrod
```

Conformance tests: `tests/conformance/boundaries.rs` reads the manifests and sources and fails
when a rule is broken.
