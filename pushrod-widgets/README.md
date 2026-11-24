# pushrod-widgets

The standard widgets for [Pushrod](https://github.com/KenSuenobu/rust-pushrod): simple, flat, and
predictable. Every widget is drawn with fills, one-pixel borders, and text. No glass, no bevels,
no gradients, no shadows, no rounded corners. See the
[design philosophy](https://github.com/KenSuenobu/rust-pushrod/blob/main/DESIGN_PHILOSOPHY.md).

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-widgets = { git = "https://github.com/KenSuenobu/rust-pushrod" }
```

This crate uses only the public API of the `pushrod` core. Another widget library can extend it
with new widgets or replace it with a different look, without changing the core.

Run the example from the repository root:

```bash
cargo run -p pushrod-widgets --example base
```

Each planned widget is specified in [`docs/widgets/`](../docs/widgets/README.md).
