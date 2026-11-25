// Controllers
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

//! # Controllers
//!
//! Reusable input controllers.  A controller owns one input pattern (click, drag, scroll, hover,
//! long-press, key), watches the raw [`Event`]s a `Widget` receives, and reports high-level
//! [`ControllerEvent`]s, so a `Widget` never re-implements "press, track, release inside",
//! dragging thresholds, or press timing.
//!
//! A `Widget` keeps its controllers in its [`WidgetCore`](crate::widget::WidgetCore) with
//! [`WidgetCore::add_controller`](crate::widget::WidgetCore::add_controller).  The default
//! [`Widget::handle_event`](crate::widget::Widget::handle_event) feeds every event to them and calls
//! [`Widget::on_controller_event`](crate::widget::Widget::on_controller_event) once per result:
//!
//! ```rust,no_run
//! use pushrod::controllers::{ControllerEvent, DragController};
//! use pushrod::event::{EventCtx, Propagation};
//! use pushrod::geometry::{point, size};
//! use pushrod::paint::{self, Painter};
//! use pushrod::widget::{Widget, WidgetCore};
//!
//! pub struct Handle {
//!     core: WidgetCore,
//! }
//!
//! impl Handle {
//!     pub fn new() -> Self {
//!         let mut core = WidgetCore::new(point(0, 0), size(10, 10));
//!         core.add_controller(DragController::new());
//!         Self { core }
//!     }
//! }
//!
//! impl Widget for Handle {
//!     fn core(&self) -> &WidgetCore { &self.core }
//!     fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }
//!     fn paint(&self, p: &mut dyn Painter) { paint::fill(p, self.get_color()); }
//!
//!     fn on_controller_event(&mut self, event: &ControllerEvent, _ctx: &mut EventCtx) -> Propagation {
//!         if let ControllerEvent::DragUpdated { offset, .. } = event {
//!             println!("dragged by {}, {}", offset.x, offset.y);
//!             return Propagation::Stop;
//!         }
//!
//!         Propagation::Continue
//!     }
//! }
//! ```
//!
//! Controllers are plain state machines: they never touch the `Widget`, the clock, or SDL2, so
//! they are tested by feeding events in.  Time comes from [`Event::Tick`] only.

use crate::event::{Event, EventMask};
use crate::geometry::{Point, point};
use crate::widget::WidgetId;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use std::time::Duration;

/// A high-level result reported by a controller.
#[derive(Debug, Clone, PartialEq)]
pub enum ControllerEvent {
    /// [`ClickController`]: the button went down on the `Widget`.
    ClickPressed,
    /// [`ClickController`]: while pressed, the pointer re-entered (`true`) or left (`false`) the
    /// `Widget`.  A release is a click only while armed.
    ClickArmed(bool),
    /// [`ClickController`]: the button was released inside the `Widget` it was pressed on.
    Clicked {
        /// The button used.
        button: MouseButton,
        /// The click count (1 single, 2 double).
        count: u8,
    },
    /// [`ClickController`]: the button was released outside the `Widget`; no click.
    ClickCancelled,

    /// [`DragController`]: the pointer moved past the threshold while pressed.
    DragStarted {
        /// Where the press happened, in window coordinates.
        start: Point,
    },
    /// [`DragController`]: the pointer moved while dragging.
    DragUpdated {
        /// Where the press happened.
        start: Point,
        /// The movement since the press (`x` and `y` can be negative).
        offset: Point,
    },
    /// [`DragController`]: the button was released after a drag.
    DragEnded {
        /// Where the press happened.
        start: Point,
        /// The total movement.
        offset: Point,
    },

    /// [`ScrollController`]: the wheel or trackpad scrolled.
    Scrolled {
        /// How far it scrolled, horizontally (`x`) and vertically (`y`).
        delta: Point,
    },

    /// [`HoverController`]: the pointer entered the `Widget`.
    HoverEntered,
    /// [`HoverController`]: the pointer left the `Widget`.
    HoverExited,

    /// [`LongPressController`]: the button was held, without moving, for the delay.
    LongPressed {
        /// Where the press happened.
        position: Point,
    },

    /// [`KeyController`]: a key went down (or repeated).
    KeyPressed {
        /// The key.
        key: Keycode,
        /// The modifier keys held.
        modifiers: Mod,
        /// `true` if this is an auto-repeat.
        repeat: bool,
    },
    /// [`KeyController`]: a key went up.
    KeyReleased {
        /// The key.
        key: Keycode,
        /// The modifier keys held.
        modifiers: Mod,
    },
}

/// An input pattern that turns raw [`Event`]s into [`ControllerEvent`]s.
///
/// Implement it to add a pattern of your own; the `Widget` side is unchanged.
pub trait Controller {
    /// Examines one event.
    ///
    /// # Arguments
    ///
    /// * `widget` - The ID of the `Widget` that owns the controller.
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where to push the results, in order.
    fn handle(&mut self, widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>);

    /// Returns the event classes the controller needs.
    ///
    /// # Returns
    ///
    /// The mask [`WidgetCore::add_controller`](crate::widget::WidgetCore::add_controller) adds to
    /// the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask;
}

/// The list of controllers a `Widget` owns.
#[derive(Default)]
pub struct Controllers {
    list: Vec<Box<dyn Controller>>,
}

impl Controllers {
    /// Adds a controller.  Controllers see events in the order they were added.
    ///
    /// # Arguments
    ///
    /// * `controller` - The controller to add.
    pub fn push(&mut self, controller: Box<dyn Controller>) {
        self.list.push(controller);
    }

    /// Returns how many controllers there are.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Returns whether there are no controllers.
    ///
    /// # Returns
    ///
    /// `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Feeds one event to every controller.
    ///
    /// # Arguments
    ///
    /// * `widget` - The ID of the owning `Widget`.
    /// * `event` - The event it received.
    ///
    /// # Returns
    ///
    /// The results of every controller, in order.  Empty if no controller reacted.
    pub fn feed(&mut self, widget: WidgetId, event: &Event) -> Vec<ControllerEvent> {
        let mut out = Vec::new();

        for controller in &mut self.list {
            controller.handle(widget, event, &mut out);
        }

        out
    }
}

/// Reports clicks: press, track in and out of the `Widget`, release.
///
/// The press is "captured" by the engine, so the release arrives even when the pointer is outside.
/// Reports [`ControllerEvent::ClickPressed`], [`ControllerEvent::ClickArmed`] as the pointer
/// leaves and re-enters, then [`ControllerEvent::Clicked`] if released while armed, otherwise
/// [`ControllerEvent::ClickCancelled`].  Only events addressed to the owning `Widget` count, so a
/// press that bubbles up from a child is not a click on the parent.
pub struct ClickController {
    button: MouseButton,
    pressed: bool,
    armed: bool,
}

impl ClickController {
    /// Creates a controller for the left mouse button.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn new() -> Self {
        Self::for_button(MouseButton::Left)
    }

    /// Creates a controller for a given mouse button.
    ///
    /// # Arguments
    ///
    /// * `button` - The button to watch.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn for_button(button: MouseButton) -> Self {
        Self {
            button,
            pressed: false,
            armed: false,
        }
    }
}

impl Default for ClickController {
    /// Creates the controller with its defaults.
    ///
    /// # Returns
    ///
    /// The controller.
    fn default() -> Self {
        Self::new()
    }
}

impl Controller for ClickController {
    /// Examines one event and reports any result.
    ///
    /// # Arguments
    ///
    /// * `widget` - The ID of the owning `Widget`.
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where results are pushed.
    fn handle(&mut self, widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>) {
        match event {
            Event::PointerPressed { id, button, .. } if *id == widget && *button == self.button => {
                self.pressed = true;
                self.armed = true;
                out.push(ControllerEvent::ClickPressed);
            }

            Event::PointerEntered { id } if *id == widget && self.pressed && !self.armed => {
                self.armed = true;
                out.push(ControllerEvent::ClickArmed(true));
            }

            Event::PointerExited { id } if *id == widget && self.pressed && self.armed => {
                self.armed = false;
                out.push(ControllerEvent::ClickArmed(false));
            }

            Event::PointerReleased {
                id, button, count, ..
            } if *id == widget && *button == self.button && self.pressed => {
                out.push(if self.armed {
                    ControllerEvent::Clicked {
                        button: *button,
                        count: *count,
                    }
                } else {
                    ControllerEvent::ClickCancelled
                });
                self.pressed = false;
                self.armed = false;
            }

            _ => {}
        }
    }

    /// Returns the event classes this controller needs.
    ///
    /// # Returns
    ///
    /// The mask added to the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask {
        EventMask::POINTER_BUTTONS | EventMask::HOVER
    }
}

/// Reports drags: a press followed by movement past a threshold.
///
/// Reports [`ControllerEvent::DragStarted`] followed by [`ControllerEvent::DragUpdated`] when the
/// pointer first moves `threshold` pixels from the press, then an update per move, then
/// [`ControllerEvent::DragEnded`] on release.  A press and release without crossing the threshold
/// reports nothing, so it can still be a click.
pub struct DragController {
    button: MouseButton,
    threshold: u32,
    start: Option<Point>,
    dragging: bool,
}

impl DragController {
    /// The default distance, in pixels, the pointer must move before a drag starts.
    pub const DEFAULT_THRESHOLD: u32 = 4;

    /// Creates a controller for the left mouse button and the default threshold.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn new() -> Self {
        Self::with_threshold(Self::DEFAULT_THRESHOLD)
    }

    /// Creates a controller for the left mouse button.
    ///
    /// # Arguments
    ///
    /// * `threshold` - The distance in pixels, measured in a straight line, that starts a drag.
    ///   `0` starts the drag on the first movement.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn with_threshold(threshold: u32) -> Self {
        Self {
            button: MouseButton::Left,
            threshold,
            start: None,
            dragging: false,
        }
    }

    /// Returns whether a drag is in progress.
    ///
    /// # Returns
    ///
    /// `true` between `DragStarted` and `DragEnded`.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }
}

impl Default for DragController {
    /// Creates the controller with its defaults.
    ///
    /// # Returns
    ///
    /// The controller.
    fn default() -> Self {
        Self::new()
    }
}

impl Controller for DragController {
    /// Examines one event and reports any result.
    ///
    /// # Arguments
    ///
    /// * `widget` - The ID of the owning `Widget`.
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where results are pushed.
    fn handle(&mut self, widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>) {
        match event {
            Event::PointerPressed {
                id,
                button,
                position,
                ..
            } if *id == widget && *button == self.button => {
                self.start = Some(*position);
                self.dragging = false;
            }

            Event::PointerMoved { id, position } if *id == widget => {
                let Some(start) = self.start else { return };
                let offset = point(position.x - start.x, position.y - start.y);

                if !self.dragging {
                    let (dx, dy) = (offset.x as i64, offset.y as i64);
                    let limit = self.threshold as i64;

                    if dx * dx + dy * dy < limit * limit || (dx == 0 && dy == 0) {
                        return;
                    }

                    self.dragging = true;
                    out.push(ControllerEvent::DragStarted { start });
                }

                out.push(ControllerEvent::DragUpdated { start, offset });
            }

            Event::PointerReleased {
                id,
                button,
                position,
                ..
            } if *id == widget && *button == self.button => {
                if let (Some(start), true) = (self.start, self.dragging) {
                    out.push(ControllerEvent::DragEnded {
                        start,
                        offset: point(position.x - start.x, position.y - start.y),
                    });
                }

                self.start = None;
                self.dragging = false;
            }

            _ => {}
        }
    }

    /// Returns the event classes this controller needs.
    ///
    /// # Returns
    ///
    /// The mask added to the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask {
        EventMask::POINTER_BUTTONS | EventMask::POINTER_MOTION
    }
}

/// Reports wheel and trackpad scrolling.
///
/// Scrolls bubble, so a scrolling container sees scrolls over its children too.
#[derive(Default)]
pub struct ScrollController;

impl ScrollController {
    /// Creates the controller.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn new() -> Self {
        Self
    }
}

impl Controller for ScrollController {
    /// Examines one event and reports any result.
    ///
    /// # Arguments
    ///
    /// * `_widget` - The ID of the owning `Widget` (unused).
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where results are pushed.
    fn handle(&mut self, _widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>) {
        if let Event::Scrolled { delta, .. } = event {
            out.push(ControllerEvent::Scrolled { delta: *delta });
        }
    }

    /// Returns the event classes this controller needs.
    ///
    /// # Returns
    ///
    /// The mask added to the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask {
        EventMask::SCROLL
    }
}

/// Reports the pointer entering and leaving the `Widget`.
#[derive(Default)]
pub struct HoverController {
    hovered: bool,
}

impl HoverController {
    /// Creates the controller.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Controller for HoverController {
    /// Examines one event and reports any result.
    ///
    /// # Arguments
    ///
    /// * `widget` - The ID of the owning `Widget`.
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where results are pushed.
    fn handle(&mut self, widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>) {
        match event {
            Event::PointerEntered { id } if *id == widget && !self.hovered => {
                self.hovered = true;
                out.push(ControllerEvent::HoverEntered);
            }

            Event::PointerExited { id } if *id == widget && self.hovered => {
                self.hovered = false;
                out.push(ControllerEvent::HoverExited);
            }

            _ => {}
        }
    }

    /// Returns the event classes this controller needs.
    ///
    /// # Returns
    ///
    /// The mask added to the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask {
        EventMask::HOVER
    }
}

/// Reports a press held in place for a delay.
///
/// Time advances only through [`Event::Tick`], so the `Widget` must receive ticks (the
/// [`EventMask::TIMER`] class).  The press is cancelled by a release, or by moving more than
/// `slop` pixels.  It reports [`ControllerEvent::LongPressed`] at most once per press.
pub struct LongPressController {
    button: MouseButton,
    delay: Duration,
    slop: u32,
    press: Option<Point>,
    held: Duration,
    fired: bool,
}

impl LongPressController {
    /// The default hold time.
    pub const DEFAULT_DELAY: Duration = Duration::from_millis(500);
    /// The default distance, in pixels, the pointer may wander during a long press.
    pub const DEFAULT_SLOP: u32 = 6;

    /// Creates a controller for the left mouse button with the default delay and slop.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn new() -> Self {
        Self::with_delay(Self::DEFAULT_DELAY)
    }

    /// Creates a controller for the left mouse button with the default slop.
    ///
    /// # Arguments
    ///
    /// * `delay` - How long the button must be held.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn with_delay(delay: Duration) -> Self {
        Self {
            button: MouseButton::Left,
            delay,
            slop: Self::DEFAULT_SLOP,
            press: None,
            held: Duration::ZERO,
            fired: false,
        }
    }
}

impl Default for LongPressController {
    /// Creates the controller with its defaults.
    ///
    /// # Returns
    ///
    /// The controller.
    fn default() -> Self {
        Self::new()
    }
}

impl Controller for LongPressController {
    /// Examines one event and reports any result.
    ///
    /// # Arguments
    ///
    /// * `widget` - The ID of the owning `Widget`.
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where results are pushed.
    fn handle(&mut self, widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>) {
        match event {
            Event::PointerPressed {
                id,
                button,
                position,
                ..
            } if *id == widget && *button == self.button => {
                self.press = Some(*position);
                self.held = Duration::ZERO;
                self.fired = false;
            }

            Event::PointerMoved { id, position } if *id == widget => {
                if let Some(start) = self.press {
                    let (dx, dy) = ((position.x - start.x) as i64, (position.y - start.y) as i64);
                    let limit = self.slop as i64;

                    if dx * dx + dy * dy > limit * limit {
                        self.press = None;
                    }
                }
            }

            Event::PointerReleased { id, button, .. }
                if *id == widget && *button == self.button =>
            {
                self.press = None;
            }

            Event::Tick { elapsed } if !self.fired => {
                if let Some(position) = self.press {
                    self.held += *elapsed;

                    if self.held >= self.delay {
                        self.fired = true;
                        out.push(ControllerEvent::LongPressed { position });
                    }
                }
            }

            _ => {}
        }
    }

    /// Returns the event classes this controller needs.
    ///
    /// # Returns
    ///
    /// The mask added to the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask {
        EventMask::POINTER_BUTTONS | EventMask::POINTER_MOTION | EventMask::TIMER
    }
}

/// Reports key presses and releases, optionally only for chosen keys.
///
/// Key events bubble, so a container can handle keys its focused child ignored.
#[derive(Default)]
pub struct KeyController {
    keys: Option<Vec<Keycode>>,
}

impl KeyController {
    /// Creates a controller that reports every key.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a controller that reports only the given keys.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to report.
    ///
    /// # Returns
    ///
    /// The controller.
    pub fn for_keys(keys: &[Keycode]) -> Self {
        Self {
            keys: Some(keys.to_vec()),
        }
    }

    /// Returns whether `key` is reported.
    fn wants(&self, key: Keycode) -> bool {
        self.keys.as_ref().is_none_or(|keys| keys.contains(&key))
    }
}

impl Controller for KeyController {
    /// Examines one event and reports any result.
    ///
    /// # Arguments
    ///
    /// * `_widget` - The ID of the owning `Widget` (unused).
    /// * `event` - The event the `Widget` received.
    /// * `out` - Where results are pushed.
    fn handle(&mut self, _widget: WidgetId, event: &Event, out: &mut Vec<ControllerEvent>) {
        match event {
            Event::KeyPressed {
                key,
                modifiers,
                repeat,
                ..
            } if self.wants(*key) => out.push(ControllerEvent::KeyPressed {
                key: *key,
                modifiers: *modifiers,
                repeat: *repeat,
            }),

            Event::KeyReleased { key, modifiers, .. } if self.wants(*key) => {
                out.push(ControllerEvent::KeyReleased {
                    key: *key,
                    modifiers: *modifiers,
                })
            }

            _ => {}
        }
    }

    /// Returns the event classes this controller needs.
    ///
    /// # Returns
    ///
    /// The mask added to the `Widget`'s event mask.
    fn event_mask(&self) -> EventMask {
        EventMask::KEYBOARD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: WidgetId = 7;

    fn press(position: Point) -> Event {
        Event::PointerPressed {
            id: ID,
            button: MouseButton::Left,
            position,
            count: 1,
        }
    }

    fn release(position: Point, count: u8) -> Event {
        Event::PointerReleased {
            id: ID,
            button: MouseButton::Left,
            position,
            count,
        }
    }

    fn moved(position: Point) -> Event {
        Event::PointerMoved { id: ID, position }
    }

    fn run(controller: &mut dyn Controller, events: &[Event]) -> Vec<ControllerEvent> {
        let mut out = Vec::new();

        for event in events {
            controller.handle(ID, event, &mut out);
        }

        out
    }

    #[test]
    fn click_inside_reports_pressed_then_clicked() {
        let mut c = ClickController::new();
        let out = run(&mut c, &[press(point(1, 1)), release(point(1, 1), 2)]);

        assert_eq!(
            out,
            vec![
                ControllerEvent::ClickPressed,
                ControllerEvent::Clicked {
                    button: MouseButton::Left,
                    count: 2
                }
            ]
        );
    }

    #[test]
    fn click_released_outside_is_cancelled() {
        let mut c = ClickController::new();
        let out = run(
            &mut c,
            &[
                press(point(1, 1)),
                Event::PointerExited { id: ID },
                release(point(99, 99), 1),
            ],
        );

        assert_eq!(
            out,
            vec![
                ControllerEvent::ClickPressed,
                ControllerEvent::ClickArmed(false),
                ControllerEvent::ClickCancelled
            ]
        );
    }

    #[test]
    fn click_rearms_when_pointer_returns() {
        let mut c = ClickController::new();
        let out = run(
            &mut c,
            &[
                press(point(1, 1)),
                Event::PointerExited { id: ID },
                Event::PointerEntered { id: ID },
                release(point(1, 1), 1),
            ],
        );

        assert_eq!(out[2], ControllerEvent::ClickArmed(true));
        assert!(matches!(out[3], ControllerEvent::Clicked { .. }));
    }

    #[test]
    fn click_ignores_other_buttons_other_widgets_and_stray_releases() {
        let mut c = ClickController::new();
        let right = Event::PointerPressed {
            id: ID,
            button: MouseButton::Right,
            position: point(0, 0),
            count: 1,
        };
        let child = Event::PointerPressed {
            id: ID + 1,
            button: MouseButton::Left,
            position: point(0, 0),
            count: 1,
        };

        assert!(run(&mut c, &[right, child, release(point(0, 0), 1)]).is_empty());
    }

    #[test]
    fn drag_starts_past_threshold_and_ends_on_release() {
        let mut c = DragController::with_threshold(5);
        let start = point(10, 10);
        let out = run(
            &mut c,
            &[
                press(start),
                moved(point(12, 12)),
                moved(point(20, 10)),
                moved(point(25, 8)),
                release(point(25, 8), 1),
            ],
        );

        assert_eq!(
            out,
            vec![
                ControllerEvent::DragStarted { start },
                ControllerEvent::DragUpdated {
                    start,
                    offset: point(10, 0)
                },
                ControllerEvent::DragUpdated {
                    start,
                    offset: point(15, -2)
                },
                ControllerEvent::DragEnded {
                    start,
                    offset: point(15, -2)
                },
            ]
        );
        assert!(!c.is_dragging());
    }

    #[test]
    fn drag_below_threshold_reports_nothing() {
        let mut c = DragController::new();
        let out = run(
            &mut c,
            &[
                press(point(0, 0)),
                moved(point(1, 1)),
                release(point(1, 1), 1),
            ],
        );

        assert!(out.is_empty());
    }

    #[test]
    fn drag_needs_a_press_first() {
        let mut c = DragController::with_threshold(0);

        assert!(run(&mut c, &[moved(point(50, 50))]).is_empty());
    }

    #[test]
    fn drag_with_zero_threshold_starts_on_first_movement_only() {
        let mut c = DragController::with_threshold(0);
        let out = run(
            &mut c,
            &[press(point(0, 0)), moved(point(0, 0)), moved(point(1, 0))],
        );

        assert_eq!(out.len(), 2);
        assert!(matches!(out[0], ControllerEvent::DragStarted { .. }));
    }

    #[test]
    fn scroll_reports_delta_even_for_bubbled_events() {
        let mut c = ScrollController::new();
        let out = run(
            &mut c,
            &[Event::Scrolled {
                id: ID + 5,
                delta: point(0, -3),
            }],
        );

        assert_eq!(
            out,
            vec![ControllerEvent::Scrolled {
                delta: point(0, -3)
            }]
        );
    }

    #[test]
    fn hover_reports_enter_and_exit_once() {
        let mut c = HoverController::new();
        let out = run(
            &mut c,
            &[
                Event::PointerEntered { id: ID },
                Event::PointerEntered { id: ID },
                Event::PointerExited { id: ID },
                Event::PointerExited { id: ID },
            ],
        );

        assert_eq!(
            out,
            vec![ControllerEvent::HoverEntered, ControllerEvent::HoverExited]
        );
    }

    fn tick(ms: u64) -> Event {
        Event::Tick {
            elapsed: Duration::from_millis(ms),
        }
    }

    #[test]
    fn long_press_fires_once_after_the_delay() {
        let mut c = LongPressController::with_delay(Duration::from_millis(300));
        let out = run(
            &mut c,
            &[press(point(3, 4)), tick(200), tick(100), tick(100)],
        );

        assert_eq!(
            out,
            vec![ControllerEvent::LongPressed {
                position: point(3, 4)
            }]
        );
    }

    #[test]
    fn long_press_is_cancelled_by_release_or_movement() {
        let delay = Duration::from_millis(100);
        let mut released = LongPressController::with_delay(delay);
        let mut wandered = LongPressController::with_delay(delay);

        assert!(
            run(
                &mut released,
                &[press(point(0, 0)), release(point(0, 0), 1), tick(500)]
            )
            .is_empty()
        );
        assert!(
            run(
                &mut wandered,
                &[press(point(0, 0)), moved(point(40, 0)), tick(500)]
            )
            .is_empty()
        );
        assert!(run(&mut LongPressController::new(), &[tick(5000)]).is_empty());
    }

    #[test]
    fn long_press_allows_small_movement() {
        let mut c = LongPressController::with_delay(Duration::from_millis(100));
        let out = run(&mut c, &[press(point(0, 0)), moved(point(2, 2)), tick(100)]);

        assert_eq!(out.len(), 1);
    }

    #[test]
    fn long_press_can_fire_again_on_the_next_press() {
        let mut c = LongPressController::with_delay(Duration::from_millis(10));
        let out = run(
            &mut c,
            &[
                press(point(0, 0)),
                tick(10),
                release(point(0, 0), 1),
                press(point(0, 0)),
                tick(10),
            ],
        );

        assert_eq!(out.len(), 2);
    }

    #[test]
    fn key_reports_all_keys_or_only_the_chosen_ones() {
        let down = |key| Event::KeyPressed {
            id: ID,
            key,
            modifiers: Mod::NOMOD,
            repeat: false,
        };
        let up = Event::KeyReleased {
            id: ID,
            key: Keycode::Return,
            modifiers: Mod::NOMOD,
        };

        assert_eq!(run(&mut KeyController::new(), &[down(Keycode::A)]).len(), 1);

        let mut only_return = KeyController::for_keys(&[Keycode::Return]);
        let out = run(
            &mut only_return,
            &[down(Keycode::A), down(Keycode::Return), up],
        );

        assert_eq!(out.len(), 2);
        assert!(matches!(
            out[1],
            ControllerEvent::KeyReleased {
                key: Keycode::Return,
                ..
            }
        ));
    }

    #[test]
    fn controllers_feed_in_order_and_report_masks() {
        let mut list = Controllers::default();

        assert!(list.is_empty());

        list.push(Box::new(HoverController::new()));
        list.push(Box::new(ClickController::new()));

        assert_eq!(list.len(), 2);
        assert_eq!(
            list.feed(ID, &Event::PointerEntered { id: ID }),
            vec![ControllerEvent::HoverEntered]
        );
        assert!(
            LongPressController::new()
                .event_mask()
                .contains(EventMask::TIMER)
        );
        assert_eq!(ScrollController::new().event_mask(), EventMask::SCROLL);
    }
}
