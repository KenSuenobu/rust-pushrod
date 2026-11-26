---
sidebar_position: 4
title: Installing on macOS
description: Step-by-step installation of Rust, SDL2, SDL2_ttf, and SDL2_image on macOS with Homebrew.
---

# Installing on macOS

These steps start from a clean machine. They match the macOS jobs in CI.

## 1. Install the Xcode command-line tools

They provide the C compiler and linker:

```bash
xcode-select --install
```

## 2. Install Rust

Install Rust with [rustup](https://rustup.rs), then check the version is 1.100 or newer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
```

## 3. Install the SDL2 libraries

Install [Homebrew](https://brew.sh), then:

```bash
brew install sdl2 sdl2_image sdl2_ttf
```

## 4. Point the linker at Homebrew

Homebrew's library folder isn't on the linker's default search path. Add it to your shell profile (`~/.zshrc`):

```bash
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"
```

Open a new terminal so the change takes effect. `brew --prefix` is `/opt/homebrew` on Apple Silicon and `/usr/local` on Intel Macs.

## 5. Check the installation

```bash
git clone https://github.com/KenSuenobu/rust-pushrod
cd rust-pushrod
cargo run -p pushrod-sdl2 --example base
```

A window opens. Close it, and you're ready for the [Quick start](./quick-start.mdx).

## Optional: build SDL2 from source

To compile SDL2 instead of using Homebrew's, install CMake (`brew install cmake`), then enable the [`bundled` feature](./installation.md#building-sdl2-from-source-the-bundled-feature). You still need `sdl2_ttf` and `sdl2_image` from step 3 and `LIBRARY_PATH` from step 4. Homebrew's SDL2_ttf and SDL2_image load Homebrew's SDL2 at run time, so `bundled` doesn't yet remove the need for it.

## Troubleshooting

| Error | Fix |
|---|---|
| `ld: library 'SDL2' not found` (or `SDL2_ttf`, `SDL2_image`) | `LIBRARY_PATH` doesn't include `$(brew --prefix)/lib`. Do step 4, then open a new terminal. |
| `xcrun: error: invalid active developer path` | The command-line tools are missing. Do step 1. |
| `is cmake not installed?` | You enabled `bundled` without CMake. Run `brew install cmake`, or remove the feature. |
