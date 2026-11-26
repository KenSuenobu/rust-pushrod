---
sidebar_position: 5
title: Installing on Windows
description: Step-by-step installation of Rust, SDL2, SDL2_ttf, and SDL2_image on Windows with the MSVC toolchain.
---

# Installing on Windows

These steps start from a clean machine and use the MSVC toolchain, the Rust default on Windows. They match the Windows jobs in CI, and use the same library versions.

## 1. Install the C++ build tools and Rust

1. Install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload. They provide the MSVC linker.
2. Install Rust with [rustup](https://rustup.rs) (`rustup-init.exe`), keeping the default `x86_64-pc-windows-msvc` toolchain.
3. In a new terminal, check the version is 1.100 or newer:

```powershell
rustc --version
```

## 2. Download the SDL2 development libraries

Download the `-VC.zip` development package of each library:

| Library | Package |
|---|---|
| SDL2 | [SDL2-devel-2.30.9-VC.zip](https://github.com/libsdl-org/SDL/releases/download/release-2.30.9/SDL2-devel-2.30.9-VC.zip) |
| SDL2_ttf | [SDL2_ttf-devel-2.22.0-VC.zip](https://github.com/libsdl-org/SDL_ttf/releases/download/release-2.22.0/SDL2_ttf-devel-2.22.0-VC.zip) |
| SDL2_image | [SDL2_image-devel-2.8.2-VC.zip](https://github.com/libsdl-org/SDL_image/releases/download/release-2.8.2/SDL2_image-devel-2.8.2-VC.zip) |

Unzip all three into one folder, for example `C:\sdl`. Each zip unpacks to its own folder (`C:\sdl\SDL2-2.30.9`, `C:\sdl\SDL2_ttf-2.22.0`, `C:\sdl\SDL2_image-2.8.2`), with the 64-bit `.lib` and `.dll` files in its `lib\x64` folder.

## 3. Point the linker and Windows at the libraries

The linker finds the `.lib` files through the `LIB` variable, and Windows finds the `.dll` files through `PATH`. In PowerShell, set both for your user account:

```powershell
$dirs = "C:\sdl\SDL2-2.30.9\lib\x64;C:\sdl\SDL2_ttf-2.22.0\lib\x64;C:\sdl\SDL2_image-2.8.2\lib\x64"
[Environment]::SetEnvironmentVariable("LIB", "$dirs;" + [Environment]::GetEnvironmentVariable("LIB", "User"), "User")
[Environment]::SetEnvironmentVariable("PATH", "$dirs;" + [Environment]::GetEnvironmentVariable("PATH", "User"), "User")
```

Open a new terminal so the change takes effect.

:::tip
When you ship your application, put `SDL2.dll`, `SDL2_ttf.dll`, and `SDL2_image.dll` next to the `.exe` instead of relying on `PATH`.
:::

## 4. Check the installation

```powershell
git clone https://github.com/KenSuenobu/rust-pushrod
cd rust-pushrod
cargo run -p pushrod-sdl2 --example base
```

A window opens. Close it, and you're ready for the [Quick start](./quick-start.mdx).

## Optional: build SDL2 from source

To compile SDL2 instead of using the SDL2 zip, install [CMake](https://cmake.org/download/) (tick **Add CMake to the system PATH**), then enable the [`bundled` feature](./installation.md#building-sdl2-from-source-the-bundled-feature). You still need the SDL2_ttf and SDL2_image zips from step 2, and their folders in `LIB` and `PATH` from step 3. `SDL2_ttf.dll` and `SDL2_image.dll` load `SDL2.dll` at run time, so keep `SDL2.dll` on `PATH` or next to the `.exe`. `bundled` doesn't yet remove the need for it.

## Troubleshooting

| Error | Fix |
|---|---|
| `LINK : fatal error LNK1181: cannot open input file 'SDL2.lib'` (or `SDL2_ttf.lib`, `SDL2_image.lib`) | `LIB` doesn't include that library's `lib\x64` folder. Check step 3, then open a new terminal. |
| `error LNK2019: unresolved external symbol` for `TTF_…` or `IMG_…` | You enabled `bundled` together with `static-link`. Use `bundled` alone; see [the `bundled` feature](./installation.md#building-sdl2-from-source-the-bundled-feature). |
| `linker 'link.exe' not found` | The C++ build tools are missing. Do step 1. |
| The program exits at once with *The code execution cannot proceed because SDL2.dll was not found* | `PATH` doesn't include the `lib\x64` folders, or the `.dll` files aren't next to the `.exe`. |
| `is cmake not installed?` | You enabled `bundled` without CMake on `PATH`. Install CMake, or remove the feature. |
| `Compatibility with CMake < 3.5 has been removed from CMake` | You use CMake 4 with `bundled`. Run `$env:CMAKE_POLICY_VERSION_MINIMUM = "3.5"`, or set it [in `.cargo/config.toml`](./installation.md#building-sdl2-from-source-the-bundled-feature). |
