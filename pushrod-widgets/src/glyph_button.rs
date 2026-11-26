// GlyphButton Widget
// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # GlyphButton
//!
//! A small button that shows one character: a scroll arrow, a close box, or a spin button.  It
//! fills its background, draws the glyph centered, and draws a one-pixel border; while pressed,
//! the colors are inverted.
//!
//! By default it behaves like [`Button`](crate::button::Button): `Clicked { id, button, count }`
//! is emitted when the mouse button is released inside it.  With
//! [`Behavior::ACTIVATE_ON_PRESS`] it emits `Clicked` as soon as it is pressed, then repeats
//! after 400 ms and every 100 ms after that for as long as it is held with the pointer inside.
//! The arrow characters `←` `↑` `→` `↓` (U+2190 to U+2193) are drawn as flat filled triangles, so
//! they look the same whatever font is loaded; any other character is drawn with the font.
//!
//! The repeat is driven by the core's animation ticks (`Event::Tick`), so it needs no timer from
//! the application.

use pushrod::color::Color;
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::MouseButton;
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore};
use std::time::Duration;

/// How long a held, activate-on-press button waits before it starts repeating.
pub const REPEAT_DELAY: Duration = Duration::from_millis(400);

/// The time between repeats once a held button has started repeating.
pub const REPEAT_INTERVAL: Duration = Duration::from_millis(100);

/// The four arrow characters a `GlyphButton` draws itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arrow {
    Left,
    Up,
    Right,
    Down,
}

impl Arrow {
    /// Maps `←` `↑` `→` `↓` to an arrow.
    ///
    /// # Arguments
    ///
    /// * `glyph` - The character to look up.
    ///
    /// # Returns
    ///
    /// The arrow, or `None` for any other character.
    fn from_glyph(glyph: char) -> Option<Self> {
        match glyph {
            '\u{2190}' => Some(Self::Left),
            '\u{2191}' => Some(Self::Up),
            '\u{2192}' => Some(Self::Right),
            '\u{2193}' => Some(Self::Down),
            _ => None,
        }
    }

    /// Draws the arrow as a triangle, centered in a `size` surface, one pixel row (or column)
    /// at a time.  Its depth is a quarter of the smaller side, plus one.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter.
    /// * `size` - The button's size.
    /// * `color` - The fill color.
    fn paint(self, p: &mut dyn Painter, size: pushrod::geometry::Size, color: Color) {
        let depth = (size.w.min(size.h) / 4 + 1) as i32;
        let (cx, cy) = (size.w as i32 / 2, size.h as i32 / 2);

        for step in 0..depth {
            let len = (step * 2 + 1) as u32;
            // `step` counts from the tip; the tip is `depth / 2` before the center.
            let along = step - depth / 2;
            let bounds = match self {
                Self::Up => Rect::new(cx - step, cy + along, len, 1),
                Self::Down => Rect::new(cx - step, cy - along - 1, len, 1),
                Self::Left => Rect::new(cx + along, cy - step, 1, len),
                Self::Right => Rect::new(cx - along - 1, cy - step, 1, len),
            };

            p.fill_rect(bounds, color);
        }
    }
}

/// A small push button showing a single character.
///
/// See the [module documentation](self) for its behavior.
#[derive(Debug)]
pub struct GlyphButton {
    core: WidgetCore,
    style: TextStyle,
    glyph: char,
    selected: bool,
    /// The time the current press has been held with the pointer inside.
    held: Duration,
    /// The held time at which the next repeat fires.
    next_repeat: Duration,
    /// Whether the press is being timed (activate-on-press, pointer down).
    repeating: bool,
    /// `false` until the first tick of a press, whose elapsed time may predate the press.
    primed: bool,
}

impl Widget for GlyphButton {
    /// Returns the button's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the button's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws the button: a fill, the centered glyph, and a one-pixel frame.
    ///
    /// When selected (pressed), the colors are inverted.  Colors, border and font set with
    /// [`Widget::set_style`] for the current state override these defaults.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the button.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let style = self.style();
        let (background, foreground) =
            paint::pressed_colors(self.selected, self.get_color(), self.style.font_color);
        let (background, foreground) = (
            style.background.unwrap_or(background),
            style.foreground.unwrap_or(foreground),
        );
        let font = style.font.as_ref().unwrap_or(&self.style);
        let border = style.border_width.unwrap_or(1);

        paint::fill(p, background);

        // The built-in font has no arrows, so the four arrow characters are drawn as shapes.
        match Arrow::from_glyph(self.glyph) {
            Some(arrow) => arrow.paint(p, size, foreground),
            None => {
                paint::text(
                    p,
                    Rect::new(0, 0, size.w, size.h),
                    font,
                    foreground,
                    &self.glyph.to_string(),
                    Truncate::None,
                );
            }
        }

        paint::border(p, size, style.border_color.unwrap_or(Color::BLACK), border);
    }

    /// Feeds the click controller, times the repeat on `Tick`, and re-emits pointer events.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the button.
    /// * `ctx` - The context used to emit `Clicked` and request animation ticks.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the pointer events the button handles; otherwise
    /// [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::Tick { elapsed } = event {
            self.on_tick(*elapsed, ctx);
            return Propagation::Continue;
        }

        self.feed_controllers(event, ctx);

        if matches!(
            event,
            Event::PointerPressed { .. }
                | Event::PointerReleased { .. }
                | Event::PointerEntered { .. }
                | Event::PointerExited { .. }
        ) {
            ctx.emit(event.clone());
            return Propagation::Stop;
        }

        Propagation::Continue
    }

    /// Updates the pressed state from the click controller and emits `Clicked`.
    ///
    /// Without `ACTIVATE_ON_PRESS`, `Clicked` is emitted on release.  With it, `Clicked` is emitted
    /// on press, repeats are timed while the pointer stays inside, and the release emits nothing.
    ///
    /// # Arguments
    ///
    /// * `event` - What the controller reported.
    /// * `ctx` - The context used to emit `Clicked`.
    ///
    /// # Returns
    ///
    /// Always [`Propagation::Continue`].
    fn on_controller_event(&mut self, event: &ControllerEvent, ctx: &mut EventCtx) -> Propagation {
        let on_press = self.behavior().contains(Behavior::ACTIVATE_ON_PRESS);

        match event {
            ControllerEvent::ClickPressed => {
                self.set_selected(true);

                if on_press {
                    self.emit_click(ctx, MouseButton::Left, 1);
                    self.start_repeat(ctx);
                }
            }
            ControllerEvent::ClickArmed(armed) => {
                self.set_selected(*armed);

                // Leaving pauses the repeat; coming back resumes it.
                if on_press && *armed && self.repeating {
                    self.primed = false;
                    ctx.request_animation();
                }
            }
            ControllerEvent::Clicked { button, count } => {
                if !on_press {
                    self.emit_click(ctx, *button, *count);
                }

                self.stop_repeat();
                self.set_selected(false);
            }
            ControllerEvent::ClickCancelled => {
                self.stop_repeat();
                self.set_selected(false);
            }
            _ => {}
        }

        Propagation::Continue
    }
}

impl GlyphButton {
    /// Creates a glyph button showing `glyph`.
    ///
    /// The button is 24 by 24 pixels at the upper left-hand corner of its parent, with the
    /// engine's default font and a black glyph.  Its background is transparent until set with
    /// `set_color`.
    ///
    /// ```rust
    /// use pushrod_widgets::glyph_button::GlyphButton;
    ///
    /// let up = GlyphButton::new('↑').at(20, 60).size(20, 20);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `glyph` - The character to show.
    ///
    /// # Returns
    ///
    /// The new `GlyphButton`.
    pub fn new(glyph: char) -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(24, 24));

        // Tick drives the repeat.  Set the mask first: adding a controller ORs in its own classes.
        core.set_event_mask(EventMask::TIMER);
        // The click controller needs presses, releases and hover.
        core.add_controller(ClickController::new());

        Self {
            core,
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Center),
            glyph,
            selected: false,
            held: Duration::ZERO,
            next_repeat: REPEAT_DELAY,
            repeating: false,
            primed: false,
        }
    }

    /// Sets the upper left-hand corner, relative to the parent `Widget`.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets the glyph font.
    ///
    /// # Arguments
    ///
    /// * `font` - The font handle.
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Sets the glyph color.
    ///
    /// # Arguments
    ///
    /// * `color` - The glyph color.
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
        self
    }

    /// Makes the button fire on press and auto-repeat while held (sets
    /// [`Behavior::ACTIVATE_ON_PRESS`]).
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn activate_on_press(mut self) -> Self {
        let behavior = self.behavior() | Behavior::ACTIVATE_ON_PRESS;

        self.set_behavior(behavior);
        self
    }

    /// Sets the glyph, and invalidates the button.
    ///
    /// # Arguments
    ///
    /// * `glyph` - The new character.
    pub fn set_glyph(&mut self, glyph: char) {
        if self.glyph != glyph {
            self.glyph = glyph;
            self.set_invalidated(true);
        }
    }

    /// Returns the glyph.
    ///
    /// # Returns
    ///
    /// The character shown.
    pub fn get_glyph(&self) -> char {
        self.glyph
    }

    /// Sets the glyph font, and invalidates the button.  The four arrow characters are drawn as
    /// shapes, so the font changes only other glyphs.
    ///
    /// # Arguments
    ///
    /// * `font` - The font handle.
    pub fn set_font(&mut self, font: Font) {
        self.style.font = font;
        self.set_invalidated(true);
    }

    /// Returns the glyph font.
    ///
    /// # Returns
    ///
    /// The font handle.
    pub fn get_font(&self) -> &Font {
        &self.style.font
    }

    /// Sets the glyph color, and invalidates the button.
    ///
    /// # Arguments
    ///
    /// * `color` - The glyph color.
    pub fn set_font_color(&mut self, color: Color) {
        self.style.font_color = color;
        self.set_invalidated(true);
    }

    /// Returns the glyph color.
    ///
    /// # Returns
    ///
    /// The color the glyph is drawn in (when not pressed).
    pub fn get_font_color(&self) -> Color {
        self.style.font_color
    }

    /// Returns whether the button is drawn selected (pressed, with the pointer inside).
    ///
    /// # Returns
    ///
    /// `true` while the button is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Emits `Clicked` for this button.
    fn emit_click(&self, ctx: &mut EventCtx, button: MouseButton, count: u8) {
        ctx.emit(Event::Clicked {
            id: self.get_id(),
            button,
            count,
        });
    }

    /// Begins timing a held press and asks for the first tick.
    fn start_repeat(&mut self, ctx: &mut EventCtx) {
        self.repeating = true;
        self.primed = false;
        self.held = Duration::ZERO;
        self.next_repeat = REPEAT_DELAY;
        ctx.request_animation();
    }

    /// Stops timing the press.
    fn stop_repeat(&mut self) {
        self.repeating = false;
    }

    /// Advances the repeat clock, emitting a `Clicked` for each repeat that came due.
    ///
    /// The first tick of a press only primes the clock: its elapsed time is measured from the
    /// previous tick anywhere in the engine, which may be long before the press.  While the
    /// pointer is outside the button the clock is paused and no further tick is requested.
    fn on_tick(&mut self, elapsed: Duration, ctx: &mut EventCtx) {
        if !self.repeating || !self.selected {
            return;
        }

        if self.primed {
            self.held += elapsed;

            while self.held >= self.next_repeat {
                self.emit_click(ctx, MouseButton::Left, 1);
                self.next_repeat += REPEAT_INTERVAL;
            }
        }

        self.primed = true;
        ctx.request_animation();
    }

    /// Sets the pressed state, mirroring it in the visual state, and invalidates.
    fn set_selected(&mut self, state: bool) {
        if self.selected != state {
            let mut visual = self.state();

            visual.set(VisualState::PRESSED, state);
            self.selected = state;
            self.set_state(visual);
            self.set_invalidated(true);
        }
    }
}
