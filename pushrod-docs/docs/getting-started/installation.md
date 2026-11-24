---
sidebar_position: 2
title: Installation
---

# Installation

Pushrod needs Rust 1.100 or newer (edition 2024) and the SDL2, SDL2_ttf, and SDL2_image libraries. Install them from your system's package manager as below. The [`bundled` feature](#building-sdl2-from-source-the-bundled-feature) can build SDL2 itself from source.

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

Pushrod is two crates:

| Crate | What it contains |
|---|---|
| `pushrod` | The core: the engine, the widget tree and IDs, events, the `paint` helpers, and texture and font caches. It contains no widgets except the window's root. |
| `pushrod-widgets` | The standard widgets: buttons, text, boxes, panels, and the rest of the widget set. Simple and flat, per the [design philosophy](../design-philosophy.md). |

Most applications use both. A different widget library can extend or replace `pushrod-widgets`; see [Widget libraries](../concepts/widget-libraries.md).

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-widgets = { git = "https://github.com/KenSuenobu/rust-pushrod" }

[dependencies.sdl2]
version = "0.38"
features = ["ttf", "image", "unsafe_textures"]
```

## Building SDL2 from source: the `bundled` feature

Enable `bundled` and Cargo compiles SDL2 from source instead of using the system library. You need a C compiler and CMake.

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod", features = ["bundled"] }
pushrod-widgets = { git = "https://github.com/KenSuenobu/rust-pushrod" }
```

`bundled` covers SDL2 only: SDL2_ttf and SDL2_image still come from the system (the `sdl2` crate doesn't build them from source). Add `static-link` as well (`features = ["bundled", "static-link"]`) to link SDL2 into your executable.

:::note
Pushrod uses the `sdl2` crate's 0.38 series. Version 0.39.0 doesn't compile with the `unsafe_textures` feature that Pushrod needs, so stay on 0.38 if you also depend on `sdl2` directly.
:::

## Running the examples

```bash
git clone https://github.com/KenSuenobu/rust-pushrod
cd rust-pushrod
cargo run -p pushrod-widgets --example base
```
