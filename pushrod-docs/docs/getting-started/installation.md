---
sidebar_position: 2
title: Installation
---

# Installation

Pushrod needs Rust 1.86 or newer and the SDL2, SDL2_ttf, and SDL2_image development libraries.

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

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }

[dependencies.sdl2]
version = "^0.36.0"
features = ["ttf", "image", "unsafe_textures"]
```

## Running the examples

```bash
git clone https://github.com/KenSuenobu/rust-pushrod
cd rust-pushrod
cargo run --example base
```
