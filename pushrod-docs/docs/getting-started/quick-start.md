---
sidebar_position: 1
title: Quick start (5 minutes)
description: Build and run a Pushrod application in about five minutes.
---

# Quick start

:::tip[TL;DR]
Install SDL2 (with SDL2_ttf and SDL2_image), add `pushrod` and `sdl2` to `Cargo.toml`, copy a font into `assets/`, paste the program below into `src/main.rs`, and run `cargo run`.
:::

You'll build this: a label and a button; each click updates the label.

![The quick-start application](/img/screenshots/quick-start.png)

## 1. Install SDL2

Ubuntu/Debian:

```bash
sudo apt install libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

macOS (Homebrew): `brew install sdl2 sdl2_image sdl2_ttf`. Other systems: see [Installation](./installation.md).

## 2. Create a project

```bash
cargo new hello-pushrod
cd hello-pushrod
```

## 3. Add the dependencies

In `Cargo.toml`:

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }

[dependencies.sdl2]
version = "^0.36.0"
features = ["ttf", "image", "unsafe_textures"]
```

Pushrod requires Rust 1.86 or newer. The `unsafe_textures` feature is required: it lets Pushrod keep one GPU texture per widget.

## 4. Add a font

Pushrod draws text with a TrueType font. Copy the one that ships with Pushrod:

```bash
mkdir assets
curl -L -o assets/OpenSans-Regular.ttf https://github.com/KenSuenobu/rust-pushrod/raw/main/assets/OpenSans-Regular.ttf
```

## 5. Write the program

Replace `src/main.rs` with:

```rust title="src/main.rs"
use pushrod::button_widget::ButtonWidget;
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::text_widget::{TextJustify, TextWidget};
use sdl2::pixels::Color;
use sdl2::ttf::FontStyle;

const FONT: &str = "assets/OpenSans-Regular.ttf";

/// Application state: passed to every callback as `&mut Counter`.
#[derive(Default)]
struct Counter {
    clicks: u32,
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let window = sdl
        .video()
        .unwrap()
        .window("Hello, Pushrod", 400, 140)
        .position_centered()
        .build()
        .unwrap();

    let mut engine: Engine<Counter> = Engine::new(size(400, 140), 60);

    let label = engine.add_widget(TextWidget::new(
        point(20, 20),
        size(360, 24),
        FONT.into(),
        FontStyle::NORMAL,
        16,
        Color::BLACK,
        TextJustify::Left,
        "Not clicked yet.".into(),
    ));

    let button = engine.add_widget(ButtonWidget::new(
        point(20, 70),
        size(120, 30),
        FONT.into(),
        FontStyle::NORMAL,
        16,
        Color::BLACK,
        TextJustify::Center,
        "Click me".into(),
        2,
    ));

    engine.on_click(button, move |state, ui| {
        state.clicks += 1;

        if let Some(text) = ui.get_mut::<TextWidget>(label) {
            text.set_text(format!("Clicked {} time(s).", state.clicks));
        }
    });

    let state = engine.run(sdl, window, Counter::default());

    println!("Clicked {} time(s) in total.", state.clicks);
}
```

## 6. Run it

```bash
cargo run
```

Click the button: the label counts your clicks. Close the window and the final count is printed, because `engine.run` returns your application state.

## What just happened

- `Engine<Counter>` owns the window's widget tree and your state type, `Counter`.
- `add_widget` added each widget to the window and returned its `WidgetId`.
- `on_click(button, ..)` registered a callback for that button. It receives `&mut Counter` and a `Ui` handle, which finds widgets by ID (`ui.get_mut::<TextWidget>(label)`).
- `engine.run` ran the event loop until the window closed, then gave the state back.

Next: [Creating an application](../guides/creating-an-application.md).
