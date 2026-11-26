---
sidebar_position: 3
title: Creating widgets
---

# Creating widgets

A widget is a struct that stores a `WidgetCore` and implements the `Widget` trait. Widgets contributed to Pushrod document every non-private function and type with a full header (summary, `# Arguments`, `# Returns`, and so on), as the example below does; see [Writing documentation](../writing-documentation.md#documenting-the-code). Only three functions are required. Everything a widget needs is in the `pushrod` core, so your widgets don't depend on `pushrod-widgets`: the standard widgets are built exactly the same way, with the same public API.

| Function | Purpose |
|---|---|
| `core(&self) -> &WidgetCore` | Gives Pushrod access to the shared state (ID, origin, size, color, visibility, texture). |
| `core_mut(&mut self) -> &mut WidgetCore` | Mutable access to the same. |
| `paint(&self, painter)` | Draws the widget through the backend-neutral [`Painter`](../platforms/backend-neutral-paint-api.md), from `(0, 0)` to `get_size()`. |

Everything else (`get_id`, `get_origin`, `get_size`, `set_color`, `set_size`, `set_hidden`, `set_invalidated`, ...) has a default implementation. Interactive widgets also implement `handle_event`.

## Example: a color swatch

This widget switches between two colors each time it is clicked and emits `Clicked` so applications can react.

![The swatch widget](/img/screenshots/custom-widget.png)

```rust
use pushrod::engine::Engine;
use pushrod::event::{Event, EventCtx, Propagation};
use pushrod::geometry::{point, size, Point, Size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};
use pushrod::input::MouseButton;
use pushrod::color::Color;
use pushrod_sdl2::WindowSettings;

/// A square that switches between two colors each time it is clicked.
pub struct SwatchWidget {
    core: WidgetCore,
    colors: [Color; 2],
    current: usize,
}

impl SwatchWidget {
    /// Creates a swatch showing `first` until it is clicked.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent widget.
    /// * `size` - The width and height in pixels.
    /// * `first` - The color shown first.
    /// * `second` - The color shown after the first click.
    ///
    /// # Returns
    ///
    /// The new `SwatchWidget`.
    pub fn new(origin: Point, size: Size, first: Color, second: Color) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            colors: [first, second],
            current: 0,
        }
    }

    /// Returns the color currently shown.
    ///
    /// # Returns
    ///
    /// The current color.
    pub fn current_color(&self) -> Color {
        self.colors[self.current]
    }
}

impl Widget for SwatchWidget {
    /// Returns the swatch's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the swatch's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the swatch with its current color and draws a one-pixel black frame.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the swatch.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.current_color());
        paint::border(p, self.get_size(), Color::BLACK, 1);
    }

    /// Switches colors on each left-button release and emits `Clicked`.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the swatch.
    /// * `ctx` - The context used to emit `Clicked`.
    ///
    /// # Returns
    ///
    /// `Stop` after a click, so parents do not see the release; otherwise `Continue`.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::PointerReleased {
            button: MouseButton::Left,
            count,
            ..
        } = event
        {
            self.current = 1 - self.current;
            self.set_invalidated(true);
            ctx.emit(Event::Clicked {
                id: self.get_id(),
                button: MouseButton::Left,
                count: *count,
            });
            return Propagation::Stop;
        }

        Propagation::Continue
    }
}
```

### Painting

`paint` runs only when the widget is *invalidated* (after creation, or after you call `set_invalidated(true)` or any setter that changes its look). The painter already targets the widget's own surface (a GPU texture with the SDL2 backend), so you always draw from `(0, 0)`. Compose the look from the helpers in `pushrod::paint`:

| Helper | Draws |
|---|---|
| `paint::fill(painter, color)` | Fills the whole texture. |
| `paint::border(painter, size, color, width)` | A border `width` pixels wide. |
| `paint::text(painter, bounds, &style, color, text)` | Text in `bounds`, justified by the `TextStyle`, centered vertically when it fits. |

Pushrod's own widgets are built the same way: a box is `fill` + `border`, a button is `fill` + `text` + `border`.

### Handling events

`handle_event` receives each event routed to the widget (see [Handling events](./handling-events.md)). Change the widget's state, call `set_invalidated(true)` if its look changed, and queue events for the application with `ctx.emit(...)`. Mouse events arrive as `Event::PointerPressed`, `Event::PointerReleased`, and `Event::PointerMoved`, plus `Event::PointerEntered` and `Event::PointerExited` when the pointer crosses the widget's edge. `handle_event` returns a [`Propagation`](../events/event-propagation.md): `Stop` consumes an input event, `Continue` lets parents see it. See the [Event reference](../events/event-reference.md).

### Using input controllers

Instead of tracking presses, drags, and timing yourself, attach a controller with `core.add_controller(ClickController::new())` and implement `on_controller_event`; the default `handle_event` feeds the controllers. See [Reusable input controllers](../events/reusable-input-controllers.md).

## Using the widget

```rust
fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<()> = Engine::new(size(200, 140), 60);

    let swatch = engine.add_widget(SwatchWidget::new(
        point(50, 20),
        size(100, 100),
        Color::RGB(70, 130, 200),
        Color::RGB(200, 200, 200),
    ));

    engine.on_click(swatch, move |_, ui| {
        let color = ui.get::<SwatchWidget>(swatch).unwrap().current_color();
        println!("Swatch is now {:?}", color);
    });

    pushrod_sdl2::run(engine, WindowSettings::new("Swatch", size(200, 140)), ())?;

    Ok(())
}
```

`ui.get::<SwatchWidget>(id)` returns your concrete type, so callbacks can call your own methods.

## Testing a widget

`Engine::dispatch` runs an `InputEvent` through routing, `handle_event`, and callbacks without a window:

```rust
// tests/swatch.rs (SwatchWidget is the widget defined above)
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::color::Color;

fn mouse(down: bool, x: i32, y: i32) -> InputEvent {
    let (position, button, clicks) = (point(x, y), MouseButton::Left, 1);

    if down {
        InputEvent::MouseDown { position, button, clicks }
    } else {
        InputEvent::MouseUp { position, button, clicks }
    }
}

#[test]
fn swatch_toggles_and_reports_clicks() {
    let mut engine: Engine<u32> = Engine::new(size(200, 140), 60);
    let swatch = engine.add_widget(SwatchWidget::new(point(50, 20), size(100, 100), Color::RED, Color::BLUE));

    engine.on_click(swatch, |clicks, _ui| *clicks += 1);

    let mut clicks = 0;
    engine.dispatch(mouse(true, 60, 30), &mut clicks);
    engine.dispatch(mouse(false, 60, 30), &mut clicks);

    assert_eq!(clicks, 1);
    assert_eq!(engine.widgets().get_as::<SwatchWidget>(swatch).unwrap().current_color(), Color::BLUE);
}
```

## Checklist for a new widget

- Store a `WidgetCore`; create it with `WidgetCore::new(origin, size)`.
- Draw only with `paint` helpers (or `Painter` calls) inside `paint`; never draw elsewhere.
- Call `set_invalidated(true)` whenever the look changes.
- Emit events with `ctx.emit` rather than calling application code.
- For a one-off drawing, skip the struct: [Custom Paint](../widgets/custom-paint.md) draws with a closure.
- To build a widget out of other widgets (a title and buttons, arrows and a thumb), make it a [composite](composite-widgets.md): its parts stay ordinary widgets applications can change.
- Add a page for it under **Widgets** in this site (see [Writing documentation](../writing-documentation.md)).
