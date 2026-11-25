---
sidebar_position: 1
title: Widget tree and IDs
---

# Widget tree and IDs

Every widget in a window lives in one tree, stored as a flat list and addressed by `WidgetId` (a `u32`):

```text
ID 0  root (RootWidget, the window)
 ├── ID 1  panel                 origins are relative to the parent
 │    ├── ID 2  label
 │    └── ID 3  button           later siblings draw on top
 └── ID 4  box
```

- **IDs are stable.** They're assigned in the order widgets are added and never change. Callbacks capture them, and in the future, form files will use them to connect designed forms to code.
- **Origins are relative to the parent.** `WidgetCache::absolute_origin(id)` gives window coordinates.
- **Stacking follows insertion order.** Children draw after (on top of) their parent; later siblings draw on top of earlier ones. The pointer always targets the top-most visible widget under it.
- **Hiding hides the subtree.** `set_hidden(true)` stops a widget and all its children from drawing or receiving the mouse. Disabled widgets also receive no input; see [Widget behavior flags and visual states](widget-behavior-flags-and-visual-states.md).

## Finding widgets

| Call | Returns |
|---|---|
| `ui.get::<T>(id)` / `ui.get_mut::<T>(id)` | The widget as its concrete type, if it is a `T`. |
| `widgets.get(id)` / `widgets.get_mut(id)` | The widget as `&dyn Widget` (any type). |
| `widgets.children_of(id)` | The IDs of its children, in drawing order. |
| `widgets.widget_at(x, y)` | The top-most visible widget at window coordinates. |
