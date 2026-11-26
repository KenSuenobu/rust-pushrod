// ProgressBar Widget
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

//! # ProgressBar
//!
//! A display-only bar showing how far along a task is, with the same semantics as the HTML
//! `<progress>` element.
//!
//! * With a value between `0` and `max` the bar is *determinate*: a black fill covers
//!   `value / max` of the white track.
//! * With no value (`set_indeterminate()`, or `value: None`) the bar is *indeterminate*: a black
//!   block sweeps back and forth ([`SWEEP`] each way) to show the application is busy.
//! * In *outline* mode (`set_outline_mode(true)`, which the drawing-mode switch of a later
//!   release will call) the indeterminate bar is a **static** striped bar and asks for no ticks.
//!
//! The animation is driven by the core's animation ticks (`Event::Tick`), so it needs no timer
//! from the application.  The widget cannot start its own animation: after adding a bar that is
//! indeterminate, or after switching a bar to indeterminate, ask for the first tick with
//! `engine.widgets().request_animation(id)` (or `ctx.request_animation()` from inside event handling).  The
//! bar keeps requesting ticks by itself while it is indeterminate, and stops when it is not.
//!
//! The bar emits no events and takes no input; set it with [`ProgressBar::set_value`], often from
//! a custom event.

use pushrod::color::Color;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Rect, point, size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};
use std::time::Duration;

/// How long the busy block takes to cross the bar in one direction.
pub const SWEEP: Duration = Duration::from_millis(1500);

/// The width of one stripe (and of the gap between stripes) in the static outline-mode bar.
const STRIPE: i32 = 4;

/// A progress bar, determinate or indeterminate.
///
/// See the [module documentation](self) for its behavior.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::progress_bar::ProgressBar;
///
/// let download = ProgressBar::new(100.0).at(20, 20).size(200, 16).value(40.0);
/// assert_eq!(download.fraction(), Some(0.4));
///
/// let busy = ProgressBar::indeterminate();
/// assert_eq!(busy.fraction(), None);
/// ```
#[derive(Debug)]
pub struct ProgressBar {
    core: WidgetCore,
    max: f64,
    value: Option<f64>,
    phase: Duration,
    outline: bool,
}

impl Widget for ProgressBar {
    /// Returns the progress bar's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the progress bar's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws the white track with a black frame, then the fill (determinate), the moving block
    /// (indeterminate), or the stripes (indeterminate in outline mode).
    ///
    /// Colors set with [`Widget::set_style`] for the current state override the frame (border
    /// color) and the fill (foreground).
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the progress bar.
    fn paint(&self, p: &mut dyn Painter) {
        let bounds = self.get_size();
        let style = self.style();
        let frame = style.border_color.unwrap_or(Color::BLACK);
        let ink = style.foreground.unwrap_or(Color::BLACK);

        paint::fill(p, style.background.unwrap_or(Color::WHITE));
        paint::border(p, bounds, frame, 1);

        if bounds.w <= 2 || bounds.h <= 2 {
            return;
        }

        let inner = Rect::new(1, 1, bounds.w - 2, bounds.h - 2);

        match self.fraction() {
            Some(fraction) => {
                let width = (inner.w as f64 * fraction).round() as u32;

                if width > 0 {
                    p.fill_rect(Rect::new(inner.x, inner.y, width, inner.h), ink);
                }
            }
            None if self.outline => Self::paint_stripes(p, inner, ink),
            None => {
                let block = (inner.w / 4).max(1);
                let x = inner.x + ((inner.w - block) as f64 * self.sweep_position()).round() as i32;

                p.fill_rect(Rect::new(x, inner.y, block, inner.h), ink);
            }
        }
    }

    /// Advances the busy block on each `Tick` and asks for the next one.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the progress bar.
    /// * `ctx` - The context used to request the next animation tick.
    ///
    /// # Returns
    ///
    /// Always [`Propagation::Continue`]; the bar handles nothing else.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::Tick { elapsed } = event
            && self.is_animating()
        {
            self.advance(*elapsed);
            ctx.request_animation();
        }

        Propagation::Continue
    }
}

impl ProgressBar {
    /// Creates an empty determinate bar (value `0`) with the given maximum.
    ///
    /// It is 200 by 16 pixels at the upper left-hand corner of its parent.  It has the `TIMER`
    /// event mask so it can animate, and nothing else: it takes no pointer or keyboard input.
    ///
    /// # Arguments
    ///
    /// * `max` - The value at which the bar is full.  A `max` that is not a positive, finite
    ///   number leaves the bar empty.
    ///
    /// # Returns
    ///
    /// The new `ProgressBar`.
    pub fn new(max: f64) -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(200, 16));

        core.set_event_mask(EventMask::TIMER);

        Self {
            core,
            max,
            value: Some(0.0),
            phase: Duration::ZERO,
            outline: false,
        }
    }

    /// Creates an indeterminate (busy) bar with a maximum of `1.0`.
    ///
    /// Call `engine.widgets().request_animation(id)` after adding it to start the animation.
    ///
    /// # Returns
    ///
    /// The new `ProgressBar`.
    pub fn indeterminate() -> Self {
        let mut bar = Self::new(1.0);

        bar.value = None;
        bar
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
    /// The progress bar, for chaining.
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
    /// The progress bar, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Starts the bar at a determinate value.
    ///
    /// # Arguments
    ///
    /// * `value` - The progress, from `0` to the maximum (values outside are clamped when drawn).
    ///
    /// # Returns
    ///
    /// The progress bar, for chaining.
    #[must_use]
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    /// Returns the value at which the bar is full.
    ///
    /// # Returns
    ///
    /// The maximum.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Sets the value at which the bar is full, and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `max` - The new maximum.
    pub fn set_max(&mut self, max: f64) {
        self.max = max;
        self.set_invalidated(true);
    }

    /// Returns the current value.
    ///
    /// # Returns
    ///
    /// `Some(value)` when determinate, `None` when indeterminate.
    pub fn get_value(&self) -> Option<f64> {
        self.value
    }

    /// Sets the value, or makes the bar indeterminate with `None`, and invalidates the widget.
    ///
    /// Switching to `None` does not start the animation by itself: request a tick (see the
    /// [module documentation](self)).
    ///
    /// # Arguments
    ///
    /// * `value` - The progress, or `None` for busy.
    pub fn set_value(&mut self, value: Option<f64>) {
        self.value = value;
        self.phase = Duration::ZERO;
        self.set_invalidated(true);
    }

    /// Makes the bar indeterminate (busy).  Same as `set_value(None)`.
    pub fn set_indeterminate(&mut self) {
        self.set_value(None);
    }

    /// Returns whether the bar is indeterminate.
    ///
    /// # Returns
    ///
    /// `true` when it has no value.
    pub fn is_indeterminate(&self) -> bool {
        self.value.is_none()
    }

    /// Returns how full the bar is, clamped to the range `0.0` to `1.0`.
    ///
    /// A value that is not a number counts as `0`, and so does any value when `max` is not a
    /// positive, finite number.
    ///
    /// # Returns
    ///
    /// `Some(fraction)` when determinate, `None` when indeterminate.
    pub fn fraction(&self) -> Option<f64> {
        let value = self.value?;

        if !(self.max.is_finite() && self.max > 0.0) || value.is_nan() {
            return Some(0.0);
        }

        Some((value / self.max).clamp(0.0, 1.0))
    }

    /// Switches outline drawing mode.  In outline mode the indeterminate bar is drawn as static
    /// stripes and stops requesting animation ticks.  Invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `outline` - `true` for the static look, `false` for the animation.
    pub fn set_outline_mode(&mut self, outline: bool) {
        self.outline = outline;
        self.set_invalidated(true);
    }

    /// Returns whether the bar is in outline drawing mode.
    ///
    /// # Returns
    ///
    /// `true` for the static look.
    pub fn is_outline_mode(&self) -> bool {
        self.outline
    }

    /// Returns whether the bar is animating: indeterminate and not in outline mode.
    ///
    /// # Returns
    ///
    /// `true` while it wants ticks.
    pub fn is_animating(&self) -> bool {
        self.is_indeterminate() && !self.outline
    }

    /// Where the busy block is along its travel, from `0.0` (left) to `1.0` (right), going there
    /// and back over two [`SWEEP`]s.
    ///
    /// # Returns
    ///
    /// The position as a fraction.
    pub fn sweep_position(&self) -> f64 {
        let t = self.phase.as_secs_f64() / SWEEP.as_secs_f64();

        if t <= 1.0 { t } else { 2.0 - t }
    }

    /// Moves the busy block on by `elapsed` and invalidates the widget.  The position wraps after
    /// a full there-and-back trip.
    fn advance(&mut self, elapsed: Duration) {
        let cycle = SWEEP * 2;
        let total = self.phase.as_nanos() + elapsed.as_nanos();

        self.phase = Duration::from_nanos((total % cycle.as_nanos()) as u64);
        self.set_invalidated(true);
    }

    /// Draws black vertical stripes (the static busy look) across `inner`.
    fn paint_stripes(p: &mut dyn Painter, inner: Rect, color: Color) {
        let mut x = 0;

        while x < inner.w as i32 {
            let width = STRIPE.min(inner.w as i32 - x) as u32;

            p.fill_rect(Rect::new(inner.x + x, inner.y, width, inner.h), color);
            x += STRIPE * 2;
        }
    }
}
