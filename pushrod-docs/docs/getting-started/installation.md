---
sidebar_position: 2
title: Installation
---

# Installation

Pushrod needs Rust 1.100 or newer (edition 2024) and the SDL2, SDL2_ttf, and SDL2_image libraries. Install them from your system's package manager as below. The [`bundled` feature](#building-sdl2-from-source-the-bundled-feature) of `pushrod-sdl2` can build SDL2 itself from source.

## Linux

Ubuntu/Debian:

```bash
sudo apt install libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

Fedora/Red Hat:

```bash
sudo dnf install SDL2-devel SDL2_ttf-devel SDL2_image-devel
```

## macOS

Install [Homebrew](https://brew.sh), then:

```bash
brew install sdl2 sdl2_image sdl2_ttf
```

On Apple Silicon, add Homebrew's library path to your shell profile so the linker finds SDL2:

```bash
export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/lib"
```

## Windows

Follow the [rust-sdl2 Windows instructions](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc) to install the SDL2, SDL2_ttf, and SDL2_image development libraries for your toolchain.

## Adding Pushrod to a project

Pushrod is three crates:

| Crate | What it contains |
|---|---|
| `pushrod` | The core: the engine, the widget tree and IDs, events, input types, the `Backend` and `Painter` traits, the `paint` helpers, and the headless test backend. It has no SDL dependency and contains no widgets except the window's root. |
| `pushrod-widgets` | The standard widgets: buttons, text, boxes, panels, and the rest of the widget set. Simple and flat, per the [design philosophy](../design-philosophy.md). It depends on `pushrod` only. |
| `pushrod-sdl2` | The SDL2 [backend](../platforms/backends.md): the window, input, GPU textures, and text drawing. |

Most applications use all three: pick a backend crate to run on. A different widget library can extend or replace `pushrod-widgets`; see [Widget libraries](../concepts/widget-libraries.md).

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-widgets = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-sdl2 = { git = "https://github.com/KenSuenobu/rust-pushrod" }
```

Your application does not depend on the `sdl2` crate; `pushrod-sdl2` does.

## Building SDL2 from source: the `bundled` feature

Enable `bundled` and Cargo compiles SDL2 from source instead of using the system library. You need a C compiler and CMake.

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-widgets = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-sdl2 = { git = "https://github.com/KenSuenobu/rust-pushrod", features = ["bundled"] }
```

`bundled` covers SDL2 only: SDL2_ttf and SDL2_image still come from the system (the `sdl2` crate doesn't build them from source). Add `static-link` as well (`features = ["bundled", "static-link"]`) to link SDL2 into your executable.

:::note
`pushrod-sdl2` uses the `sdl2` crate's 0.38 series. Version 0.39.0 doesn't compile with the `unsafe_textures` feature that Pushrod needs, so stay on 0.38 if you also depend on `sdl2` directly.
:::

## Running the examples

```bash
git clone https://github.com/KenSuenobu/rust-pushrod
cd rust-pushrod
cargo run -p pushrod-sdl2 --example base
```
