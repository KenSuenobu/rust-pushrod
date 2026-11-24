---
sidebar_position: 1
title: Creating an application
---

# Creating an application

A Pushrod application has three parts: an **engine** that owns the window's widgets and runs the event loop, an **application state** type of your choosing, and **callbacks** that connect widget events to your code.

## The engine and application state

```rust
#[derive(Default)]
struct AppState {
    saves: u32,
}

let mut engine: Engine<AppState> = Engine::new(size(320, 200), 60);
```

`Engine::new(size, frame_rate)` creates the window's root widget (ID `0`, white) at the given size. The second argument is the target frames per second. `AppState` is any type you like; use `()` if you don't need state.

Run the engine with an SDL2 context and window. `run` blocks until the window closes (or a callback calls `ui.shutdown()`) and then returns your state:

```rust
let sdl = sdl2::init().unwrap();
let window = sdl.video().unwrap()
    .window("Events", 320, 200)
    .position_centered()
    .build()
    .unwrap();

let state = engine.run(sdl, window, AppState::default());
```

## Adding widgets

`add_widget` adds a widget to the window's root and returns its `WidgetId`. `add_child_widget(parent, widget)` adds it inside another widget; a child's origin is **relative to its parent**, and later siblings are drawn on top of earlier ones.

```rust
// A gray panel; its children are positioned relative to it.
let mut panel = BaseWidget::new(point(10, 10), size(300, 180));
panel.set_color(Color::RGB(220, 220, 220));
let panel_id = engine.add_widget(panel);

let indicator = engine.add_child_widget(
    panel_id,
    BoxWidget::new(point(20, 20), size(40, 40), Color::BLACK, 1),
);
```

Keep the IDs you need in callbacks. IDs are assigned in the order widgets are added and never change.

## Connecting callbacks

```rust
engine.on_click(save, move |state, ui| {
    state.saves += 1;
    ui.widgets().get_mut(indicator).unwrap().set_color(Color::RGB(70, 160, 70));
});

engine.on_click(quit, |_, ui| ui.shutdown());
```

Callbacks receive `&mut AppState` and a `Ui` handle. See [Handling events](./handling-events.md) for everything `Ui` can do.

## A complete application

![The application below, after clicking Save](/img/screenshots/events.png)

```rust title="src/main.rs"
use pushrod::base_widget::BaseWidget;
use pushrod::box_widget::BoxWidget;
use pushrod::button_widget::ButtonWidget;
use pushrod::engine::Engine;
use pushrod::event::PushrodEvent;
use pushrod::geometry::{point, size};
use pushrod::text_widget::TextJustify;
use pushrod::widget::Widget;
use sdl2::pixels::Color;
use sdl2::ttf::FontStyle;

const FONT: &str = "assets/OpenSans-Regular.ttf";

#[derive(Default)]
struct AppState {
    saves: u32,
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let window = sdl.video().unwrap().window("Events", 320, 200).position_centered().build().unwrap();
    let mut engine: Engine<AppState> = Engine::new(size(320, 200), 60);

    // A gray panel; its children are positioned relative to it.
    let mut panel = BaseWidget::new(point(10, 10), size(300, 180));
    panel.set_color(Color::RGB(220, 220, 220));
    let panel_id = engine.add_widget(panel);

    let indicator = engine.add_child_widget(panel_id, BoxWidget::new(point(20, 20), size(40, 40), Color::BLACK, 1));

    let save = engine.add_child_widget(
        panel_id,
        ButtonWidget::new(point(20, 80), size(100, 28), FONT.into(), FontStyle::NORMAL, 14,
            Color::BLACK, TextJustify::Center, "Save".into(), 2),
    );
    let quit = engine.add_child_widget(
        panel_id,
        ButtonWidget::new(point(140, 80), size(100, 28), FONT.into(), FontStyle::NORMAL, 14,
            Color::BLACK, TextJustify::Center, "Quit".into(), 2),
    );

    // Per-widget callback: runs only when `save` is clicked.
    engine.on_click(save, move |state, ui| {
        state.saves += 1;
        ui.widgets().get_mut(indicator).unwrap().set_color(Color::RGB(70, 160, 70));
    });

    // Stop the engine from a callback.
    engine.on_click(quit, |_, ui| ui.shutdown());

    // Catch-all callback: sees every event any widget generates.
    engine.on_event(|_, _, event| {
        if let PushrodEvent::Clicked(id, clicks) = event {
            println!("widget {} clicked ({} click(s))", id, clicks);
        }
    });

    let state = engine.run(sdl, window, AppState::default());
    println!("saved {} time(s)", state.saves);
}
```

## Tips

- Changing a widget (`set_text`, `set_color`, `set_size`, ...) marks it for repainting; Pushrod repaints it and refreshes the screen on the next frame. You never draw directly.
- Keep callbacks short; they run on the UI thread between frames.
- Hide a widget and its children with `set_hidden(true)`; hidden widgets are neither drawn nor clicked.
