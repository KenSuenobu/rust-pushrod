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

//! # Custom Paint
//!
//! A `Widget` drawn entirely by a closure the application supplies, for a one-off drawing (a
//! chart, a preview) that does not deserve its own `Widget` implementation.
//!
//! The closure draws through the backend-neutral [`Painter`], so it works with every backend and
//! with the headless test painter.  It runs only when the widget is invalidated, into the
//! widget's own texture; frames are composed by blitting the cached texture.
//!
//! An optional event closure receives the events delivered to the widget and can change the
//! widget's data, emit events with [`EventCtx::emit`], or post custom events with
//! [`EventCtx::post`].

use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Point, Size};
use pushrod::paint::Painter;
use pushrod::widget::{Widget, WidgetCore};
use std::fmt;

/// The closure that draws the widget: the painter, the widget's size, and its data.
type PaintFn<S> = dyn Fn(&mut dyn Painter, Size, &S);

/// The closure that receives events: the widget's data, the event, and the event context.
type EventFn<S> = dyn FnMut(&mut S, &Event, &mut EventCtx) -> Propagation;

/// Returns the event classes a widget with an event closure receives unless
/// [`CustomPaint::with_event_mask`] says otherwise: pointer, scroll, keyboard, text, and focus input.
///
/// # Returns
///
/// The default `EventMask`.
fn default_events() -> EventMask {
    EventMask::POINTER_BUTTONS
        | EventMask::POINTER_MOTION
        | EventMask::HOVER
        | EventMask::SCROLL
        | EventMask::KEYBOARD
        | EventMask::TEXT
        | EventMask::FOCUS
}

/// A widget drawn by an application closure.
///
/// `S` is the data the closures share: the paint closure reads it, and the event closure may
/// change it.  [`CustomPaint::new`] creates a widget with no data (`S` is `()`);
/// [`CustomPaint::with_data`] attaches some.
///
/// # Examples
///
/// ```rust
/// use pushrod::color::Color;
/// use pushrod::geometry::{Rect, point, size};
/// use pushrod::paint;
/// use pushrod_widgets::custom_paint::CustomPaint;
///
/// // A bar chart of fictional monthly totals.
/// let widget = CustomPaint::with_data(
///     point(10, 10),
///     size(120, 60),
///     vec![10u32, 30, 20],
///     |p, size, bars| {
///         paint::fill(p, Color::WHITE);
///         for (i, bar) in bars.iter().enumerate() {
///             let height = (*bar).min(size.h);
///             p.fill_rect(
///                 Rect::new(10 + i as i32 * 30, (size.h - height) as i32, 20, height),
///                 Color::BLUE,
///             );
///         }
///         paint::border(p, size, Color::BLACK, 1);
///     },
/// );
/// ```
pub struct CustomPaint<S: 'static = ()> {
    core: WidgetCore,
    data: S,
    paint: Box<PaintFn<S>>,
    on_event: Option<Box<EventFn<S>>>,
}

impl<S> fmt::Debug for CustomPaint<S> {
    /// Formats the widget's shared state; the closures cannot be printed.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomPaint")
            .field("core", &self.core)
            .field("has_event_closure", &self.on_event.is_some())
            .finish_non_exhaustive()
    }
}

impl<S> Widget for CustomPaint<S> {
    /// Returns the widget's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the widget's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Runs the paint closure with the painter, the widget's size, and its data.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        (self.paint)(p, self.get_size(), &self.data);
    }

    /// Passes the event to the event closure, if there is one, then invalidates the widget so
    /// the paint closure runs again and shows whatever the closure changed.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the widget.
    /// * `ctx` - The context the closure uses to emit or post events.
    ///
    /// # Returns
    ///
    /// What the event closure returns; [`Propagation::Continue`] when there is none.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        let Some(on_event) = self.on_event.as_mut() else {
            return Propagation::Continue;
        };
        let propagation = on_event(&mut self.data, event, ctx);

        self.set_invalidated(true);
        propagation
    }
}

impl CustomPaint<()> {
    /// Creates a widget with no data, drawn by `paint`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent.
    /// * `size` - The width and height in pixels.
    /// * `paint` - Draws the widget: the painter, the widget's size, and the (empty) data.  It
    ///   runs only when the widget is invalidated.
    ///
    /// # Returns
    ///
    /// The new `CustomPaint`.
    pub fn new(
        origin: Point,
        size: Size,
        paint: impl Fn(&mut dyn Painter, Size, &()) + 'static,
    ) -> Self {
        Self::with_data(origin, size, (), paint)
    }
}

impl<S> CustomPaint<S> {
    /// Creates a widget that carries `data`, drawn by `paint`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent.
    /// * `size` - The width and height in pixels.
    /// * `data` - The data the closures share.
    /// * `paint` - Draws the widget: the painter, the widget's size, and the data.  It runs only
    ///   when the widget is invalidated.
    ///
    /// # Returns
    ///
    /// The new `CustomPaint`.
    pub fn with_data(
        origin: Point,
        size: Size,
        data: S,
        paint: impl Fn(&mut dyn Painter, Size, &S) + 'static,
    ) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            data,
            paint: Box::new(paint),
            on_event: None,
        }
    }

    /// Sets the closure that receives the widget's events, and subscribes the widget to pointer,
    /// scroll, keyboard, text, and focus events (change that with [`CustomPaint::with_event_mask`]).
    ///
    /// The widget is invalidated after every event it handles, so the paint closure always shows
    /// the closure's changes.
    ///
    /// # Arguments
    ///
    /// * `on_event` - Receives the data, the event, and the context; returns whether the event
    ///   keeps travelling to the widget's parents.
    ///
    /// # Returns
    ///
    /// The widget, for chaining.
    #[must_use]
    pub fn on_event(
        mut self,
        on_event: impl FnMut(&mut S, &Event, &mut EventCtx) -> Propagation + 'static,
    ) -> Self {
        self.on_event = Some(Box::new(on_event));
        self.core.set_event_mask(default_events());
        self
    }

    /// Chooses which classes of events the widget receives.  Call it after
    /// [`CustomPaint::on_event`].
    ///
    /// # Arguments
    ///
    /// * `mask` - The event classes to receive.
    ///
    /// # Returns
    ///
    /// The widget, for chaining.
    #[must_use]
    pub fn with_event_mask(mut self, mask: EventMask) -> Self {
        self.core.set_event_mask(mask);
        self
    }

    /// Returns the data the closures share.
    ///
    /// # Returns
    ///
    /// A reference to the data.
    pub fn data(&self) -> &S {
        &self.data
    }

    /// Returns the data for modification, and invalidates the widget so the paint closure runs
    /// again.
    ///
    /// # Returns
    ///
    /// A mutable reference to the data.
    pub fn data_mut(&mut self) -> &mut S {
        self.set_invalidated(true);
        &mut self.data
    }

    /// Replaces the data and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `data` - The new data.
    pub fn set_data(&mut self, data: S) {
        self.data = data;
        self.set_invalidated(true);
    }
}
