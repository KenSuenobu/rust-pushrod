# Spec: painting

Conformance tests: `tests/conformance/paint.rs`.

## The Painter

`Painter` is the only way a widget draws. Coordinates are relative to the widget's upper
left-hand corner, in logical pixels. A painter reports:

- `cache_policy()`: how the backend keeps painted widgets;
- `scale_factor()`: physical pixels per logical pixel, `1.0` by default. An unusable value
  (zero, negative, not a number) **must** read as `1.0`.

Every drawing call a widget makes **must** reach the painter, in call order.
`RecordingPainter` records the calls, so a widget can be tested without a window.

## Cache policy

| Policy | Meaning |
|---|---|
| `CachePolicy::TextureCache` (default) | Each widget paints into its own cached surface (a GPU texture); a frame blits those surfaces. |
| `CachePolicy::DirectDraw` | Invalidated widgets paint straight to the screen with clipping, for memory-poor systems. |

A `Backend` reports its policy with `cache_policy()`.

## Rules for a backend

1. **Paint only when invalidated.** The engine asks the backend to paint a widget only when that
   widget is invalidated. A frame with nothing invalidated draws and presents nothing.
2. **Frames are blits.** A frame draws each visible widget's cached surface; parents come before
   children.
3. **An expose re-blits; it does not repaint.** `invalidate_all` copies every texture again;
   only `repaint_all` (used when the scale factor changes) paints every widget again.
4. **Release removed widgets.** The surface of a removed widget is released and never drawn
   again.
