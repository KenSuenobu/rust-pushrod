# pushrod-widgets

The standard widgets for [Pushrod](https://pushrod.nobudata.com): simple, flat, and
predictable. Every widget is drawn with fills, one-pixel borders, and text. No glass, no bevels,
no gradients, no shadows, no rounded corners. See the
[design philosophy](https://github.com/KenSuenobu/rust-pushrod/blob/main/DESIGN_PHILOSOPHY.md).

```toml
[dependencies]
pushrod = "0.1"
pushrod-widgets = "0.1"
pushrod-sdl2 = "0.1"
```

This crate uses only the public API of the `pushrod` core. Another widget library can extend it
with new widgets or replace it with a different look, without changing the core.

Run the example from the repository root:

```bash
cargo run -p pushrod-sdl2 --example base
```

`--example gallery` shows every widget on a labeled tile.

Each widget has a page on the [documentation site](https://pushrod.nobudata.com/widgets). The API
reference is on [docs.rs](https://docs.rs/pushrod-widgets).
