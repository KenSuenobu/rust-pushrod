# rust-pushrod

[![CI](https://github.com/KenSuenobu/rust-pushrod/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/KenSuenobu/rust-pushrod/actions/workflows/ci.yml)

Pushrod is a library that uses the SDL2 library to interact with the operating system, and uses
OpenGL for the drawing layer.

## Documentation

The documentation site lives in [`pushrod-docs/`](pushrod-docs/) (Docusaurus). It has a five-minute quick start, guides for creating applications, handling events, and creating widgets, and a page for every widget. To preview it: `cd pushrod-docs && npm install && npm start`.

## Summary

Pushrod is small, simple, and lightweight.  It combines the two classic ways of writing GUI
applications:

- **An event queue, like GEM on the Atari ST.**  SDL2 events are routed to the `Widget` under
  the mouse.  `Widget`s change their own state, and queue any events they generate (such as
  `Clicked`).
- **Event procedures, like Visual Basic.**  After the queue is processed, each event is delivered
  to the callbacks registered for that `Widget`.  Callbacks receive mutable access to your
  application state and to every `Widget` in the window, by ID.

An earlier version of Pushrod called callbacks directly from inside the `Widget`s, which made it
nearly impossible to pass mutable references to the rest of the application.  Queueing events
first, and only then calling the callbacks once no `Widget` is borrowed, removes that problem.

`Widget`s are stored in a flat list and addressed by ID, with parent and child links by ID, like
the `OBJECT` tree of a GEM resource.  This keeps forms simple to save, load, and edit visually.

## Crates

Pushrod is a Cargo workspace with two crates:

| Crate | What it contains |
|---|---|
| [`pushrod`](src/lib.rs) (this directory) | The core: engine, widget tree and IDs, events and callbacks, `paint` helpers, texture and font caches. No widgets except the window's root. |
| [`pushrod-widgets`](pushrod-widgets/) | The standard widgets: simple, flat, and predictable. |

```toml
[dependencies]
pushrod = { git = "https://github.com/KenSuenobu/rust-pushrod" }
pushrod-widgets = { git = "https://github.com/KenSuenobu/rust-pushrod" }
```

SDL2 support is moving into its own backend crate, `pushrod-sdl2` (roadmap 10.6), so the core and
the widgets won't depend on SDL at all and an SDL3 backend can drop in later.

`pushrod-widgets` uses only the core's public API, so another widget library can extend it or
replace it entirely: drop shadows, gradients, tree and graph views, 3D containers, whatever an
application needs, without changing the core.

## Writing an application

```rust
#[derive(Default)]
struct AppState {
    clicks: u32,
}

let mut engine: Engine<AppState> = Engine::new(size(800, 600), 30);

let label = engine.add_widget(TextWidget::new(/* ... */));
let button = engine.add_widget(ButtonWidget::new(/* ... */));

engine.on_click(button, move |state, ui| {
    state.clicks += 1;
    ui.get_mut::<TextWidget>(label)
        .unwrap()
        .set_text(format!("Clicked {} times", state.clicks));
});

let state = engine.run(sdl_context, window, AppState::default());
```

Prefer to run the loop yourself? Ask the engine for each event instead of registering callbacks:

```rust
let mut events = engine.event_loop(&sdl_context, window);

while let Some(event) = engine.next_event(&mut events) {
    if let Event::Clicked { id, .. } = event {
        if id == button { /* ... */ }
    }
}
```

See [`pushrod-widgets/examples/base.rs`](pushrod-widgets/examples/base.rs) for a complete application, and the
[quick start](pushrod-docs/docs/getting-started/quick-start.mdx) for both styles side by side.

## Writing a widget

A `Widget` stores a `WidgetCore` (ID, origin, size, color, visibility, texture) and implements
three functions.  Everything else has a default implementation.  Appearance is composed from the
functions in the `paint` module (`fill`, `border`, `text`):

```rust
pub struct PanelWidget {
    core: WidgetCore,
}

impl Widget for PanelWidget {
    fn core(&self) -> &WidgetCore { &self.core }
    fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }

    fn paint(&self, c: &mut Canvas<Window>, _fc: &mut FontCache) {
        paint::fill(c, self.get_color());
        paint::border(c, self.get_size(), Color::BLACK, 1);
    }
}
```

Interactive `Widget`s also implement `handle_event`, and call `ctx.emit(...)` to generate events.

Pushrod requires Rust 1.100 or newer (edition 2024).

## Design philosophy

Pushrod follows one rule: **Keep It Stupid Simple.** Widgets are flat: fills, one-pixel borders,
and text. No glass, no beveled edges, no rounded corners, no shadows. Making things look fancy is up to the
application, or to a separate widget library that extends or replaces `pushrod-widgets`. The reasons are performance, a clean and consistent look, usability, and predictable
behavior. See [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md).

## Roadmap

The [roadmap](docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md) covers the core, events, forms, and
release plan. Each widget has its own roadmap in [docs/widgets](docs/widgets/README.md) that
specifies what it is, what it does, how it looks, and how it responds.

## Installing on Ubuntu Linux

```shell
sudo apt install libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

## Installing on Redhat Linux

```shell
sudo yum install libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

## Installing on Mac OS X

Install Brew.  [Go here to get instructions on how to install.](https://www.brew.sh)

Once installed, run the following commands:

```shell
brew update
brew upgrade
brew install sdl2 sdl2_image sdl2_ttf
```

Then modify your .profile and add:

```shell
export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/lib"
```

And you should be able to not only build the application, but you should be able to run the
examples (`cargo run -p pushrod-widgets --example base`).

### State of the app

This is the current state of the example app.

![App image](pushrod-widgets/examples/example-app.png)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Demos, examples, tests, mockups, and documentation use fictional sample data only: no real names or personal information.
