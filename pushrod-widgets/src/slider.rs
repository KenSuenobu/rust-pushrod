// Slider Widget
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

//! # Slider
//!
//! A knob on a track for picking a number from a range, horizontal (the default) or vertical.
//!
//! * **Mouse:** pressing anywhere on the widget moves the knob to the pointer (pressing on the
//!   knob itself grabs it without a jump), and dragging moves it; the value is always clamped to
//!   `min..=max`.  While the button is down the knob is drawn inverted.
//! * **Keyboard** (when the slider has the focus): Left/Down and Right/Up move by one *key step*
//!   (the `step`, or a hundredth of the range when there is none), Page Down/Up by ten, Home and
//!   End jump to the ends.
//! * **Snapping:** with a `step`, every value is a multiple of it counted from `min` (and `max`
//!   is always reachable).  **Ticks** are only marks drawn every `ticks` units; they do not snap
//!   by themselves, so give the slider the same `step` to snap to them.
//! * **Value text:** with `show_value`, the current value is drawn beside the track (to the right
//!   when horizontal, below when vertical).
//!
//! It emits `ValueChanged { id }` each time the value actually changes (continuously while
//! dragging, and for each key press), and `EditingFinished { id }` when the mouse button is
//! released.  Changing the value with [`Slider::set_value`] emits nothing.
//!
//! A horizontal slider has its minimum on the left; a vertical one has it at the bottom.

use pushrod::color::Color;
use pushrod::error::Result;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{Key, MouseButton};
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::properties::{
    Property, core_properties, expect_float, read_only, set_core_property, track,
};
use pushrod::registry::PropertyValue;
use pushrod::widget::{Behavior, Widget, WidgetCore};

/// The thickness of the knob along the track, in pixels.
pub const KNOB_THICKNESS: u32 = 10;

/// The length of the knob across the track, in pixels.
pub const KNOB_LENGTH: u32 = 18;

/// The width reserved for the value text beside a horizontal slider, in pixels.
const VALUE_WIDTH: u32 = 44;

/// The height reserved for the value text below a vertical slider, in pixels.
const VALUE_HEIGHT: u32 = 18;

/// The length of a tick mark, in pixels.
const TICK_LENGTH: u32 = 4;

/// The direction the track runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
}

/// A slider: a knob on a track that picks a value from a range.
///
/// See the [module documentation](self) for its behavior.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::slider::Slider;
///
/// let volume = Slider::new(0.0, 100.0).at(20, 20).size(200, 24).value(40.0).show_value(true);
/// assert_eq!(volume.get_value(), 40.0);
/// ```
#[derive(Debug)]
pub struct Slider {
    core: WidgetCore,
    style: TextStyle,
    orientation: Orientation,
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    ticks: f64,
    show_value: bool,
    pressed: bool,
    grab: i32,
}

impl Widget for Slider {
    /// Returns the slider's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the slider's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Lists the widget's properties: the core properties, then its own.
    ///
    /// # Returns
    ///
    /// The properties.
    fn properties(&self) -> Vec<Property> {
        let mut list = core_properties(self);

        list.push(Property::float("min", self.min));
        list.push(Property::float("max", self.max));
        list.push(Property::float("step", self.get_step()));
        list.push(Property::float("value", self.get_value()));
        list.push(Property::bool("vertical", self.is_vertical()));

        list
    }

    /// Sets a property by name; the widget's own, or else a core property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed, `false` if the widget already had it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Property`](pushrod::error::Error::Property) if there is no such
    /// property, it is read only, or the value is the wrong kind or out of range.
    fn set_property(&mut self, name: &str, value: &PropertyValue) -> Result<bool> {
        match name {
            "min" => {
                let n = expect_float(name, value)?;
                let max = self.max;

                track(self, |w| w.set_range(n, max))
            }
            "max" => {
                let n = expect_float(name, value)?;
                let min = self.min;

                track(self, |w| w.set_range(min, n))
            }
            "step" => {
                let n = expect_float(name, value)?;

                track(self, |w| w.set_step(n))
            }
            "value" => {
                let n = expect_float(name, value)?;

                track(self, |w| w.set_value(n))
            }
            "vertical" => Err(read_only(name)),
            _ => set_core_property(self, name, value),
        }
    }

    /// Draws the track, the tick marks, the knob, and (with `show_value`) the value text.
    ///
    /// The knob is white with a black frame, and inverted (black) while the mouse button is down.
    /// Colors set with [`Widget::set_style`] for the current state override the background, the
    /// frame and track (`border_color`), and the value text (`foreground`).
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the slider.
    fn paint(&self, p: &mut dyn Painter) {
        let bounds = self.get_size();
        let style = self.style();
        let ink = style.border_color.unwrap_or(Color::BLACK);
        let text_color = style.foreground.unwrap_or(self.style.font_color);
        let font = style.font.as_ref().unwrap_or(&self.style);
        let (start, length) = self.track();
        let cross = self.cross_center();
        let horizontal = self.orientation == Orientation::Horizontal;

        paint::fill(p, style.background.unwrap_or(self.get_color()));

        // The track: a two pixel line.
        p.fill_rect(
            if horizontal {
                Rect::new(start, cross - 1, length as u32, 2)
            } else {
                Rect::new(cross - 1, start, 2, length as u32)
            },
            ink,
        );

        // The ticks, on the side of the knob away from the value text.
        if self.ticks > 0.0 && self.max > self.min {
            let count = ((self.max - self.min) / self.ticks + 1e-9).floor() as u32;
            let offset = KNOB_LENGTH as i32 / 2 + 2;

            for index in 0..=count.min(1000) {
                let at = self.position_of(self.min + index as f64 * self.ticks);

                p.fill_rect(
                    if horizontal {
                        Rect::new(at, cross + offset, 1, TICK_LENGTH)
                    } else {
                        Rect::new(cross + offset, at, TICK_LENGTH, 1)
                    },
                    ink,
                );
            }
        }

        // The knob, inverted while pressed.
        let knob = self.knob_bounds();
        let (inside, _) = paint::pressed_colors(self.pressed, Color::WHITE, Color::BLACK);

        p.fill_rect(knob, inside);
        p.rect(knob, ink);

        if self.show_value {
            let area = if horizontal {
                Rect::new(
                    (bounds.w.saturating_sub(VALUE_WIDTH) + 8) as i32,
                    0,
                    VALUE_WIDTH.saturating_sub(8),
                    bounds.h,
                )
            } else {
                Rect::new(
                    0,
                    bounds.h.saturating_sub(VALUE_HEIGHT) as i32,
                    bounds.w,
                    VALUE_HEIGHT.min(bounds.h),
                )
            };

            paint::text(p, area, font, text_color, &self.value_text(), Truncate::End);
        }
    }

    /// Moves the knob for the mouse, and for the arrow, page, Home and End keys.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the slider.
    /// * `ctx` - The context used to emit `ValueChanged` and `EditingFinished`, and to turn
    ///   window coordinates into the slider's own.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the pointer events and keys it handles; otherwise
    /// [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        match event {
            Event::PointerPressed {
                button: MouseButton::Left,
                position,
                ..
            } => {
                let local = point(position.x - ctx.origin().x, position.y - ctx.origin().y);

                // Grabbing the knob keeps the pointer's offset from its center; pressing the
                // track moves the knob's center to the pointer.
                let knob = self.knob_bounds();
                let on_knob = local.x >= knob.x
                    && local.x < knob.x + knob.w as i32
                    && local.y >= knob.y
                    && local.y < knob.y + knob.h as i32;

                self.grab = if on_knob {
                    self.position_of(self.value) - self.main_coordinate(local.x, local.y)
                } else {
                    0
                };
                self.pressed = true;
                self.set_invalidated(true);
                self.follow(local.x, local.y, ctx);
                Propagation::Stop
            }
            Event::PointerMoved { position, .. } if self.pressed => {
                self.follow(
                    position.x - ctx.origin().x,
                    position.y - ctx.origin().y,
                    ctx,
                );
                Propagation::Stop
            }
            Event::PointerReleased {
                button: MouseButton::Left,
                ..
            } if self.pressed => {
                self.pressed = false;
                self.set_invalidated(true);
                ctx.emit(Event::EditingFinished { id: self.get_id() });
                Propagation::Stop
            }
            Event::KeyPressed { key, .. } => self.on_key(*key, ctx),
            _ => Propagation::Continue,
        }
    }
}

impl Slider {
    /// Creates a horizontal slider from `min` to `max`, with the knob at `min`.
    ///
    /// It is 200 by 24 pixels at the upper left-hand corner of its parent, can take keyboard
    /// focus, has no step, no ticks and no value text, and has a transparent background until set
    /// with `set_color`.  If `min` is greater than `max` the two are swapped, and a bound that is
    /// not a finite number becomes `0` (`min`) or `100` (`max`).
    ///
    /// # Arguments
    ///
    /// * `min` - The value at the left (or bottom) end.
    /// * `max` - The value at the right (or top) end.
    ///
    /// # Returns
    ///
    /// The new `Slider`.
    pub fn new(min: f64, max: f64) -> Self {
        let (min, max) = Self::sanitize(min, max);
        let mut core = WidgetCore::new(point(0, 0), size(200, 24));

        // The pointer events drive the knob; the keys adjust it.  Set the mask first.
        core.set_event_mask(
            EventMask::POINTER_BUTTONS | EventMask::POINTER_MOTION | EventMask::KEYBOARD,
        );

        let mut slider = Self {
            core,
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            orientation: Orientation::Horizontal,
            min,
            max,
            value: min,
            step: 0.0,
            ticks: 0.0,
            show_value: false,
            pressed: false,
            grab: 0,
        };

        slider.set_behavior(Behavior::FOCUSABLE);
        slider
    }

    /// Orders and repairs a pair of bounds.
    fn sanitize(min: f64, max: f64) -> (f64, f64) {
        let min = if min.is_finite() { min } else { 0.0 };
        let max = if max.is_finite() { max } else { 100.0 };

        (min.min(max), min.max(max))
    }

    /// Sets the upper left-hand corner, relative to the parent.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
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
    /// The slider, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Makes the slider vertical, with the minimum at the bottom.  Swap its size to suit
    /// (the default 200 by 24 is a horizontal shape).
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    /// Starts the slider at a value (snapped to the step, and clamped to the range).
    ///
    /// # Arguments
    ///
    /// * `value` - The starting value.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn value(mut self, value: f64) -> Self {
        self.value = self.snap(value);
        self
    }

    /// Snaps values to multiples of `step`, counted from the minimum.  `0` (or a negative or
    /// non-finite number) means no snapping.
    ///
    /// # Arguments
    ///
    /// * `step` - The step, in value units.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        self.set_step(step);
        self
    }

    /// Draws a tick mark every `spacing` value units, starting at the minimum.  `0` (or a
    /// negative or non-finite number) draws none.
    ///
    /// # Arguments
    ///
    /// * `spacing` - The distance between marks, in value units.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn ticks(mut self, spacing: f64) -> Self {
        self.ticks = if spacing.is_finite() && spacing > 0.0 {
            spacing
        } else {
            0.0
        };
        self
    }

    /// Shows the value as text beside the track.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show it.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Sets the value text's font.
    ///
    /// # Arguments
    ///
    /// * `font` - The font handle.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Sets the value text's color.
    ///
    /// # Arguments
    ///
    /// * `color` - The text color.
    ///
    /// # Returns
    ///
    /// The slider, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
        self
    }

    /// Returns the current value.
    ///
    /// # Returns
    ///
    /// The value, always within the range and on the step.
    pub fn get_value(&self) -> f64 {
        self.value
    }

    /// Sets the value (snapped and clamped) and invalidates the slider.  Emits no event.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value.
    pub fn set_value(&mut self, value: f64) {
        self.value = self.snap(value);
        self.set_invalidated(true);
    }

    /// Returns the range.
    ///
    /// # Returns
    ///
    /// `(min, max)`.
    pub fn range(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    /// Sets the range (see [`Slider::new`] for how bad bounds are repaired), keeps the value
    /// inside it, and invalidates the slider.  Emits no event.
    ///
    /// # Arguments
    ///
    /// * `min` - The new minimum.
    /// * `max` - The new maximum.
    pub fn set_range(&mut self, min: f64, max: f64) {
        (self.min, self.max) = Self::sanitize(min, max);
        self.value = self.snap(self.value);
        self.set_invalidated(true);
    }

    /// Returns the step.
    ///
    /// # Returns
    ///
    /// The step, or `0` when values are not snapped.
    pub fn get_step(&self) -> f64 {
        self.step
    }

    /// Sets the step (see [`Slider::step`]), re-snaps the value, and invalidates the slider.
    ///
    /// # Arguments
    ///
    /// * `step` - The step, in value units.
    pub fn set_step(&mut self, step: f64) {
        self.step = if step.is_finite() && step > 0.0 {
            step
        } else {
            0.0
        };
        self.value = self.snap(self.value);
        self.set_invalidated(true);
    }

    /// Returns whether the value text is shown.
    ///
    /// # Returns
    ///
    /// `true` when shown.
    pub fn shows_value(&self) -> bool {
        self.show_value
    }

    /// Returns whether the slider runs vertically.
    ///
    /// # Returns
    ///
    /// `true` for a vertical slider.
    pub fn is_vertical(&self) -> bool {
        self.orientation == Orientation::Vertical
    }

    /// Returns whether the mouse button is down on the knob.
    ///
    /// # Returns
    ///
    /// `true` while dragging.
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Returns the value as drawn beside the track: a whole number without decimals, otherwise
    /// at most two decimals with trailing zeros removed.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn value_text(&self) -> String {
        if (self.value - self.value.round()).abs() < 1e-9 {
            format!("{:.0}", self.value)
        } else {
            let text = format!("{:.2}", self.value);

            text.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }

    /// Returns the amount an arrow key moves the value.
    ///
    /// # Returns
    ///
    /// The step, or a hundredth of the range when there is none.
    pub fn key_step(&self) -> f64 {
        if self.step > 0.0 {
            self.step
        } else {
            (self.max - self.min) / 100.0
        }
    }

    /// Clamps a value to the range and snaps it to the step.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to adjust; not a number counts as the minimum.
    ///
    /// # Returns
    ///
    /// The adjusted value.
    pub fn snap(&self, value: f64) -> f64 {
        let value = if value.is_nan() { self.min } else { value };
        let mut value = value.clamp(self.min, self.max);

        if self.step > 0.0 {
            value = self.min + ((value - self.min) / self.step).round() * self.step;
            value = value.clamp(self.min, self.max);
        }

        // Drop the floating-point noise of the arithmetic above (105.00000000000001 -> 105).
        ((value * 1e9).round() / 1e9).clamp(self.min, self.max)
    }

    /// Returns where the knob's center sits along the track, in the slider's own coordinates
    /// (`x` when horizontal, `y` when vertical).
    ///
    /// # Arguments
    ///
    /// * `value` - The value to place.
    ///
    /// # Returns
    ///
    /// The coordinate in pixels.
    pub fn position_of(&self, value: f64) -> i32 {
        let (start, length) = self.track();
        let fraction = if self.max > self.min {
            ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let along = match self.orientation {
            Orientation::Horizontal => fraction,
            Orientation::Vertical => 1.0 - fraction,
        };

        start + (length as f64 * along).round() as i32
    }

    /// Returns the value at a position along the track, snapped and clamped.
    ///
    /// # Arguments
    ///
    /// * `position` - The coordinate in the slider's own coordinates (`x` when horizontal, `y`
    ///   when vertical).
    ///
    /// # Returns
    ///
    /// The value.
    pub fn value_at(&self, position: i32) -> f64 {
        let (start, length) = self.track();
        let along = if length > 0 {
            ((position - start) as f64 / length as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let fraction = match self.orientation {
            Orientation::Horizontal => along,
            Orientation::Vertical => 1.0 - along,
        };

        self.snap(self.min + fraction * (self.max - self.min))
    }

    /// Returns the knob's rectangle in the slider's own coordinates.
    ///
    /// # Returns
    ///
    /// The rectangle.
    pub fn knob_bounds(&self) -> Rect {
        let at = self.position_of(self.value);
        let cross = self.cross_center();
        let (thick, long) = (KNOB_THICKNESS as i32, KNOB_LENGTH as i32);

        match self.orientation {
            Orientation::Horizontal => Rect::new(
                at - thick / 2,
                cross - long / 2,
                KNOB_THICKNESS,
                KNOB_LENGTH,
            ),
            Orientation::Vertical => Rect::new(
                cross - long / 2,
                at - thick / 2,
                KNOB_LENGTH,
                KNOB_THICKNESS,
            ),
        }
    }

    /// Returns where the track starts and how long it is, along its direction.  The knob's
    /// center travels over it, so it is inset by half the knob, and by the value text's room.
    fn track(&self) -> (i32, i32) {
        let bounds = self.get_size();
        let (extent, reserved) = match self.orientation {
            Orientation::Horizontal => (bounds.w, if self.show_value { VALUE_WIDTH } else { 0 }),
            Orientation::Vertical => (bounds.h, if self.show_value { VALUE_HEIGHT } else { 0 }),
        };
        let start = KNOB_THICKNESS as i32 / 2;
        let length = extent as i32 - reserved as i32 - KNOB_THICKNESS as i32;

        (start, length.max(0))
    }

    /// Returns the coordinate, across the track, of its center line.
    fn cross_center(&self) -> i32 {
        let bounds = self.get_size();

        match self.orientation {
            Orientation::Horizontal => bounds.h as i32 / 2,
            Orientation::Vertical => bounds.w as i32 / 2,
        }
    }

    /// Picks the coordinate along the track out of a point in the slider's own coordinates.
    fn main_coordinate(&self, x: i32, y: i32) -> i32 {
        match self.orientation {
            Orientation::Horizontal => x,
            Orientation::Vertical => y,
        }
    }

    /// Moves the knob to follow the pointer at (`x`, `y`) in the slider's own coordinates.
    fn follow(&mut self, x: i32, y: i32, ctx: &mut EventCtx) {
        let target = self.value_at(self.main_coordinate(x, y) + self.grab);

        self.change_to(target, ctx);
    }

    /// Sets a new value and, if it differs from the current one, invalidates the slider and
    /// emits `ValueChanged`.
    fn change_to(&mut self, value: f64, ctx: &mut EventCtx) {
        let value = self.snap(value);

        if value != self.value {
            self.value = value;
            self.set_invalidated(true);
            ctx.emit(Event::ValueChanged { id: self.get_id() });
        }
    }

    /// Handles the keys that adjust the value.
    fn on_key(&mut self, key: Key, ctx: &mut EventCtx) -> Propagation {
        let step = self.key_step();
        let target = match key {
            Key::Left | Key::Down => self.value - step,
            Key::Right | Key::Up => self.value + step,
            Key::PageDown => self.value - step * 10.0,
            Key::PageUp => self.value + step * 10.0,
            Key::Home => self.min,
            Key::End => self.max,
            _ => return Propagation::Continue,
        };

        self.change_to(target, ctx);
        Propagation::Stop
    }
}
