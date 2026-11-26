# pushrod-sdl2

The SDL2 backend for [Pushrod](https://pushrod.nobudata.com). The `pushrod` core and
`pushrod-widgets` contain no SDL2 code. This crate opens the window, reads input, and draws.
Each widget gets its own GPU texture, which is repainted only when the widget is invalidated.
Frames are composed by blitting those textures, and text is drawn from a glyph atlas.

```toml
[dependencies]
pushrod = "0.1"
pushrod-widgets = "0.1"
pushrod-sdl2 = "0.1"
```

The system needs SDL2, SDL2_ttf, and SDL2_image. The
[installation guide](https://pushrod.nobudata.com/getting-started/installation) has
step-by-step instructions for Linux, macOS, and Windows, and lists the crate's features
(`bundled`, `static-link`, `parley`).

Run the example from the repository root:

```bash
cargo run -p pushrod-sdl2 --example base
```

The full documentation, with guides and examples, is at <https://pushrod.nobudata.com>. The API
reference is on [docs.rs](https://docs.rs/pushrod-sdl2).
