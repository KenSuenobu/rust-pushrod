---
sidebar_position: 3
title: Installing on Linux
description: Step-by-step installation of Rust, SDL2, SDL2_ttf, and SDL2_image on Linux.
---

# Installing on Linux

These steps start from a clean machine. They match the Linux job in CI, which runs on Ubuntu.

## 1. Install Rust

Install Rust with [rustup](https://rustup.rs), then check the version is 1.100 or newer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
```

## 2. Install a C toolchain and the SDL2 libraries

Ubuntu/Debian:

```bash
sudo apt update
sudo apt install build-essential pkg-config libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

Fedora/Red Hat:

```bash
sudo dnf install gcc pkgconf-pkg-config SDL2-devel SDL2_ttf-devel SDL2_image-devel
```

The `-dev`/`-devel` packages install the libraries the linker needs, in the standard system paths, so you don't have to set anything else up.

## 3. Check the installation

```bash
git clone https://github.com/KenSuenobu/rust-pushrod
cd rust-pushrod
cargo run -p pushrod-sdl2 --example base
```

A window opens. Close it, and you're ready for the [Quick start](./quick-start.mdx).

## Optional: build SDL2 from source

To compile SDL2 instead of using the system package, install CMake (`sudo apt install cmake` or `sudo dnf install cmake`), then enable the [`bundled` feature](./installation.md#building-sdl2-from-source-the-bundled-feature). SDL2_ttf and SDL2_image still come from step 2.

## Troubleshooting

| Error | Fix |
|---|---|
| `unable to find library -lSDL2` (or `-lSDL2_ttf`, `-lSDL2_image`) | The `-dev`/`-devel` package for that library is missing. Install it as in step 2. |
| `is cmake not installed?` | You enabled `bundled` without CMake. Install CMake, or remove the feature. |
| `Compatibility with CMake < 3.5 has been removed from CMake` | You use CMake 4 with `bundled`. Run `export CMAKE_POLICY_VERSION_MINIMUM=3.5`, or set it [in `.cargo/config.toml`](./installation.md#building-sdl2-from-source-the-bundled-feature). |
| `No available video device` | There is no display, for example over SSH. Run from a desktop session, or under `xvfb-run`. |
