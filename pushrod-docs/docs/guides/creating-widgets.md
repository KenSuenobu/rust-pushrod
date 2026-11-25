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
| `paint(&self, canvas, font_cache)` | Draws the widget into its own texture, from `(0, 0)` to `get_size()`. |

Everything else (`get_id`, `get_origin`, `get_size`, `set_color`, `set_size`, `set_hidden`, `set_invalidated`, ...) has a default implementation. Interactive widgets also implement `handle_event`.

## Example: a color swatch

This widget switches between two colors each time it is clicked and emits `Clicked` so applications can react.

![The swatch widget](/img/screenshots/custom-widget.png)

```rust
use pushrod::engine::Engine;
use pushrod::event::{Event, EventCtx};
use pushrod::font::FontCache;
use pushrod::geometry::{point, size, Point, Size};
use pushrod::paint;
use pushrod::widget::{Widget, WidgetCore};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

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
    /// * `c` - The canvas, targeting the swatch's texture.
    /// * `_fc` - The font cache (unused).
    fn paint(&self, c: &mut Canvas<Window>, _fc: &mut FontCache) {
        paint::fill(c, self.current_color());
        paint::border(c, self.get_size(), Color::BLACK, 1);
    }

    /// Switches colors on each left-button release and emits `Clicked`.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the swatch.
    /// * `ctx` - The context used to emit `Clicked`.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) {
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
        }
    }
}
```

### Painting

`paint` runs only when the widget is *invalidated* (after creation, or after you call `set_invalidated(true)` or any setter that changes its look). The canvas already targets the widget's own GPU texture, so you always draw from `(0, 0)`. Compose the look from the helpers in `pushrod::paint`:

| Helper | Draws |
|---|---|
| `paint::fill(canvas, color)` | Fills the whole texture. |
| `paint::border(canvas, size, color, width)` | A border `width` pixels wide. |
| `paint::text(canvas, font_cache, bounds, &style, color, text)` | Text in `bounds`, justified by the `TextStyle`, centered vertically when it fits. |

Pushrod's own widgets are built the same way: a box is `fill` + `border`, a button is `fill` + `text` + `border`.

### Handling events

`handle_event` receives each event routed to the widget (see [Handling events](./handling-events.md)). Change the widget's state, call `set_invalidated(true)` if its look changed, and queue events for the application with `ctx.emit(...)`. Mouse events arrive as `Event::PointerPressed`, `Event::PointerReleased`, and `Event::PointerMoved`, plus `Event::PointerEntered` and `Event::PointerExited` when the pointer crosses the widget's edge. See the [Event reference](../events/event-reference.md).

## Using the widget

```rust
fn main() {
    let sdl = sdl2::init().unwrap();
    let window = sdl.video().unwrap().window("Swatch", 200, 140).position_centered().build().unwrap();
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

    engine.run(sdl, window, ());
}
```

`ui.get::<SwatchWidget>(id)` returns your concrete type, so callbacks can call your own methods.

## Testing a widget

`Engine::dispatch` runs an SDL2 event through routing, `handle_event`, and callbacks without a window:

```rust
// tests/swatch.rs (SwatchWidget is the widget defined above)
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;

fn mouse(down: bool, x: i32, y: i32) -> Event {
    if down {
        Event::MouseButtonDown { timestamp: 0, window_id: 0, which: 0, mouse_btn: MouseButton::Left, clicks: 1, x, y }
    } else {
        Event::MouseButtonUp { timestamp: 0, window_id: 0, which: 0, mouse_btn: MouseButton::Left, clicks: 1, x, y }
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
- Draw only with `paint` helpers (or SDL2 canvas calls) inside `paint`; never draw elsewhere.
- Call `set_invalidated(true)` whenever the look changes.
- Emit events with `ctx.emit` rather than calling application code.
- Add a page for it under **Widgets** in this site (see [Writing documentation](../writing-documentation.md)).
