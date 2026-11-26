// Widget
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

//! # Widget
//!
//! A UI element that exists in a window, whether visible or otherwise, that consumes and
//! alternatively generates events.
//!
//! Every `Widget` owns a [`WidgetCore`], which stores the state that all `Widget`s share: the
//! ID, parent ID, origin, size, color, visibility, and invalidation flag.  The
//! `Widget` trait provides default implementations for all of the getters and setters against
//! the `WidgetCore`, so a new `Widget` only needs to implement three functions:
//!
//! ```rust,no_run
//! use pushrod::paint::{self, Painter};
//! use pushrod::widget::{Widget, WidgetCore};
//!
//! pub struct MyWidget {
//!     core: WidgetCore,
//! }
//!
//! impl Widget for MyWidget {
//!     fn core(&self) -> &WidgetCore { &self.core }
//!     fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }
//!
//!     fn paint(&self, p: &mut dyn Painter) {
//!         paint::fill(p, self.get_color());
//!     }
//! }
//! ```
//!
//! If the compiler is happy, the `Widget` is complete: there are no lists of `Widget` types to
//! update elsewhere in the library.

use crate::appearance::{Appearance, StateStyle};
use crate::color::Color;
use crate::controllers::{Controller, ControllerEvent, Controllers};
use crate::event::{Event, EventCtx, EventMask, Propagation};
use crate::geometry::{Point, Size};
use crate::paint::Painter;
use bitflags::bitflags;
use std::any::Any;

/// The ID of a `Widget`.
///
/// IDs are assigned by the [`WidgetCache`](crate::cache::WidgetCache) in the order in which
/// `Widget`s are added, starting with the window's root `Widget` at ID `0`
/// ([`ROOT_WIDGET_ID`]).  An ID is stable for the lifetime of the `WidgetCache`, which is what
/// allows forms to be designed, stored, and wired to callbacks by ID.
pub type WidgetId = u32;

/// The ID of the window's root `Widget`.
///
/// The root is created by the `Engine` (a [`RootWidget`](crate::root_widget::RootWidget)), fills
/// the window, and is the ancestor of every other `Widget`.  It is its own parent.
pub const ROOT_WIDGET_ID: WidgetId = 0;

bitflags! {
    /// What a `Widget` can do, stored on its [`WidgetCore`].
    ///
    /// Behavior flags describe the `Widget`'s role rather than how it looks right now (that is
    /// [`VisualState`]), so they work the same for every widget and can be saved in a form file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::widget::Behavior;
    ///
    /// let ok_button = Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION | Behavior::DISMISSES_DIALOG;
    ///
    /// assert!(ok_button.contains(Behavior::DEFAULT_ACTION));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Behavior: u16 {
        /// The `Widget` can receive keyboard focus.
        const FOCUSABLE = 1 << 0;
        /// The `Widget` stays pressed after a click, until it is clicked again.
        const TOGGLE = 1 << 1;
        /// The `Widget` is activated by the Return key.
        const DEFAULT_ACTION = 1 << 2;
        /// The `Widget` is activated by the Escape key.
        const CANCEL_ACTION = 1 << 3;
        /// Activating the `Widget` closes the enclosing modal dialog and reports itself.
        const DISMISSES_DIALOG = 1 << 4;
        /// The `Widget` fires when the mouse button is pressed, not when it is released.
        const ACTIVATE_ON_PRESS = 1 << 5;
        /// Only one sibling with this flag is checked at a time (a radio group).
        const EXCLUSIVE = 1 << 6;
        /// The `Widget` and its children are not drawn and do not receive input.
        const HIDDEN = 1 << 7;
        /// The `Widget` is an overlay (menu, tooltip, pop-up): it is not clipped to its ancestors'
        /// bounds, though its own children are still clipped to it.
        const OVERLAY = 1 << 8;
    }
}

bitflags! {
    /// How a `Widget` currently looks, stored on its [`WidgetCore`].
    ///
    /// A `Widget` with no flags set is in its normal state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::widget::VisualState;
    ///
    /// let state = VisualState::CHECKED | VisualState::HOVERED;
    ///
    /// assert!(!state.contains(VisualState::DISABLED));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct VisualState: u16 {
        /// The mouse button is held down on the `Widget`.
        const PRESSED = 1 << 0;
        /// The `Widget` is checked or selected.
        const CHECKED = 1 << 1;
        /// The `Widget` is in a mixed (tri-state) condition.
        const INDETERMINATE = 1 << 2;
        /// The `Widget` is disabled: it is drawn dimmed and receives no input.
        const DISABLED = 1 << 3;
        /// The `Widget` has keyboard focus.
        const FOCUSED = 1 << 4;
        /// The pointer is inside the `Widget`.
        const HOVERED = 1 << 5;
        /// A drag is currently over the `Widget`.
        const DRAG_OVER = 1 << 6;
    }
}

/// The state that every `Widget` shares.
///
/// Store one `WidgetCore` in each `Widget` structure and return it from [`Widget::core`] and
/// [`Widget::core_mut`]; every getter and setter on the `Widget` trait works against it.  It holds
/// the `Widget`'s ID and parent ID (assigned by the `WidgetCache`), its origin relative to its
/// parent, its size, its background color, its [`Behavior`] flags (including hidden) and [`VisualState`] flags, its [`Appearance`] (a style per state), its [`EventMask`], whether it needs repainting, and
/// the texture its backend paints it into.
///
/// # Examples
///
/// ```rust
/// use pushrod::geometry::{point, size};
/// use pushrod::widget::WidgetCore;
///
/// let core = WidgetCore::new(point(10, 20), size(100, 30));
/// ```
#[derive(Debug)]
pub struct WidgetCore {
    /// The `Widget`'s ID, assigned by the `WidgetCache` when the `Widget` is added.
    pub(crate) id: WidgetId,
    /// The ID of the parent `Widget`; the root `Widget` is its own parent.
    pub(crate) parent: WidgetId,
    origin: Point,
    size: Size,
    color: Color,
    behavior: Behavior,
    state: VisualState,
    appearance: Appearance,
    event_mask: EventMask,
    invalidated: bool,
    controllers: Controllers,
}

impl WidgetCore {
    /// Sets the event mask the `Widget` reports from [`Widget::event_mask`].
    ///
    /// Use it while building a `Widget`.  To change a mask after the `Widget` is in a window, use
    /// [`WidgetCache::set_event_mask`](crate::cache::WidgetCache::set_event_mask), which also emits
    /// the change.
    ///
    /// # Arguments
    ///
    /// * `mask` - The classes of events the `Widget` wants.
    pub fn set_event_mask(&mut self, mask: EventMask) {
        self.event_mask = mask;
    }

    /// Adds an input controller, and adds the event classes it needs to the event mask.
    ///
    /// The default [`Widget::handle_event`] feeds every event to the controllers.  Call
    /// [`WidgetCore::set_event_mask`] before this, not after, or the added classes are replaced.
    ///
    /// # Arguments
    ///
    /// * `controller` - The controller, such as [`ClickController`](crate::controllers::ClickController).
    pub fn add_controller(&mut self, controller: impl Controller + 'static) {
        self.event_mask = self.event_mask | controller.event_mask();
        self.controllers.push(Box::new(controller));
    }

    /// Creates the shared state for a new `Widget`.
    ///
    /// The `Widget` starts visible, invalidated (so it is painted on the first frame), with a
    /// transparent white background color.  Its ID and parent are assigned later, when it is added to
    /// a `WidgetCache`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner of the `Widget`, relative to its parent.
    /// * `size` - The width and height of the `Widget` in pixels.
    ///
    /// # Returns
    ///
    /// A new `WidgetCore`, to be stored in a `Widget` structure.
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            id: 0,
            parent: ROOT_WIDGET_ID,
            origin,
            size,
            color: Color::RGBA(255, 255, 255, 0),
            behavior: Behavior::empty(),
            state: VisualState::empty(),
            appearance: Appearance::default(),
            event_mask: EventMask::NONE,
            invalidated: true,
            controllers: Controllers::default(),
        }
    }
}

/// A drawable, interactive object presented in a window.
///
/// `Widget`s display information (text, images, graphics) and provide controls the user can
/// interact with.  Every `Widget` stores a [`WidgetCore`] and implements three functions:
/// [`core`](Widget::core), [`core_mut`](Widget::core_mut), and [`paint`](Widget::paint).
/// Interactive `Widget`s also implement [`handle_event`](Widget::handle_event).  Every other
/// function has a default implementation that works against the `WidgetCore`.
///
/// `Widget` requires `Any` so that the `WidgetCache` can return a `Widget` as its concrete type
/// ([`WidgetCache::get_as`](crate::cache::WidgetCache::get_as)).
pub trait Widget: Any {
    /// Returns the `Widget`'s shared state.
    ///
    /// # Returns
    ///
    /// A reference to the [`WidgetCore`] stored in the `Widget`.
    fn core(&self) -> &WidgetCore;

    /// Returns the `Widget`'s shared state for modification.
    ///
    /// # Returns
    ///
    /// A mutable reference to the [`WidgetCore`] stored in the `Widget`.
    fn core_mut(&mut self) -> &mut WidgetCore;

    /// Paints the `Widget` into its own texture.
    ///
    /// The painter is already targeting the `Widget`'s surface, so drawing always starts at `(0, 0)`
    /// and is bounded by [`get_size`](Widget::get_size).  Compose the look of the `Widget` with the
    /// functions in the [`paint`](crate::paint) module (`fill`, `border`, `text`).  The `WidgetCache`
    /// only calls this when the `Widget` is invalidated; between repaints, the texture is reused.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting this `Widget`.  It is a Pushrod [`Painter`], never an `SDL2`
    ///   type, so the `Widget` works with any backend.
    fn paint(&self, p: &mut dyn Painter);

    /// Handles an event sent to this `Widget`.
    ///
    /// The `WidgetCache` sends mouse events to the `Widget` under the pointer (or the `Widget` that
    /// captured the mouse), and `PointerEntered` and `PointerExited` events when the pointer enters or leaves it.  Events
    /// the `Widget` generates in response, such as `Clicked`, are queued with
    /// [`EventCtx::emit`](crate::event::EventCtx::emit) and delivered to the application after event
    /// processing completes.  The default implementation feeds the event to the `Widget`'s controllers
    /// (see [`Widget::feed_controllers`]) and otherwise ignores it.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to this `Widget`.
    /// * `ctx` - The context used to queue events the `Widget` generates;
    ///   [`EventCtx::phase`](crate::event::EventCtx::phase) says whether the event is being
    ///   captured, handled as the target, or bubbled.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] to end delivery of an input event, or [`Propagation::Continue`] (the
    /// default) to let the next `Widget` in the capture, target, bubble path see it.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        self.feed_controllers(event, ctx)
    }

    /// Feeds an event to the `Widget`'s controllers and calls
    /// [`Widget::on_controller_event`] for each result, in order.
    ///
    /// The default [`Widget::handle_event`] is this call.  A `Widget` that overrides `handle_event`
    /// calls it to keep its controllers working.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to this `Widget`.
    /// * `ctx` - The context passed on to `on_controller_event`.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] if any `on_controller_event` call returned it, otherwise
    /// [`Propagation::Continue`].
    fn feed_controllers(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        let id = self.get_id();
        let results = self.core_mut().controllers.feed(id, event);
        let mut propagation = Propagation::Continue;

        for result in &results {
            if self.on_controller_event(result, ctx) == Propagation::Stop {
                propagation = Propagation::Stop;
            }
        }

        propagation
    }

    /// Receives a high-level result from one of the `Widget`'s controllers.
    ///
    /// # Arguments
    ///
    /// * `_event` - What the controller reported, such as `ControllerEvent::Clicked`.
    /// * `_ctx` - The context used to queue events the `Widget` generates.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] to end delivery of the raw event that produced it; the default
    /// ignores the result and returns [`Propagation::Continue`].
    fn on_controller_event(
        &mut self,
        _event: &ControllerEvent,
        _ctx: &mut EventCtx,
    ) -> Propagation {
        Propagation::Continue
    }

    /// Returns `true` if this `Widget` wants input events in the capture phase.
    ///
    /// A capturing `Widget` sees pointer and key events *before* its descendants, and can return
    /// [`Propagation::Stop`] to keep them from reaching the target.  Every `Widget` receives
    /// events in the bubble phase regardless.
    ///
    /// # Returns
    ///
    /// `false` by default.
    fn handles_capture(&self) -> bool {
        false
    }

    /// Returns the classes of events this `Widget` wants to receive.
    ///
    /// The `WidgetCache` skips the `Widget` for every event whose class is not in this mask, so a
    /// `Widget` that handles no events costs nothing.  An event is delivered only when it is also
    /// in the run loop's mask.
    ///
    /// # Returns
    ///
    /// The mask; [`EventMask::NONE`] by default.  A `Widget` that handles events calls
    /// `core_mut().set_event_mask(..)` at creation, or an application calls
    /// [`WidgetCache::set_event_mask`](crate::cache::WidgetCache::set_event_mask).
    fn event_mask(&self) -> EventMask {
        self.core().event_mask
    }

    /// Returns the `Widget`'s ID.
    ///
    /// # Returns
    ///
    /// The ID assigned by the `WidgetCache` when the `Widget` was added, or `0` if it has not been
    /// added yet.
    fn get_id(&self) -> WidgetId {
        self.core().id
    }

    /// Returns the ID of the `Widget`'s parent.
    ///
    /// # Returns
    ///
    /// The parent's ID.  The root `Widget` is its own parent.
    fn get_parent(&self) -> WidgetId {
        self.core().parent
    }

    /// Returns the `Widget`'s origin.
    ///
    /// # Returns
    ///
    /// The upper left-hand corner of the `Widget`, relative to its parent.  Use
    /// [`WidgetCache::absolute_origin`](crate::cache::WidgetCache::absolute_origin) for window
    /// coordinates.
    fn get_origin(&self) -> Point {
        self.core().origin
    }

    /// Returns the `Widget`'s size.
    ///
    /// # Returns
    ///
    /// The width and height of the `Widget` in pixels.
    fn get_size(&self) -> Size {
        self.core().size
    }

    /// Returns the `Widget`'s main (background) color.
    ///
    /// # Returns
    ///
    /// The background color.
    fn get_color(&self) -> Color {
        self.core().color
    }

    /// Moves the `Widget`, and invalidates it.
    ///
    /// # Arguments
    ///
    /// * `point` - The new upper left-hand corner, relative to the parent `Widget`.
    fn set_origin(&mut self, point: Point) {
        self.core_mut().origin = point;
        self.set_invalidated(true);
    }

    /// Resizes the `Widget`, and invalidates it.  Its texture is resized on the next repaint.
    ///
    /// # Arguments
    ///
    /// * `size` - The new width and height in pixels.
    fn set_size(&mut self, size: Size) {
        self.core_mut().size = size;
        self.set_invalidated(true);
    }

    /// Sets the `Widget`'s main (background) color, and invalidates it.
    ///
    /// # Arguments
    ///
    /// * `color` - The new background color.
    fn set_color(&mut self, color: Color) {
        self.core_mut().color = color;
        self.set_invalidated(true);
    }

    /// Returns whether the `Widget` is hidden.
    ///
    /// Hidden `Widget`s and all of their children are not drawn and do not receive mouse events.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` is hidden, `false` if it is shown.
    fn is_hidden(&self) -> bool {
        self.core().behavior.contains(Behavior::HIDDEN)
    }

    /// Hides or shows the `Widget` and all of its children, and invalidates it.
    ///
    /// # Arguments
    ///
    /// * `state` - `true` to hide the `Widget`, `false` to show it.
    fn set_hidden(&mut self, state: bool) {
        self.core_mut().behavior.set(Behavior::HIDDEN, state);
        self.set_invalidated(true);
    }

    /// Returns what the `Widget` can do.
    ///
    /// # Returns
    ///
    /// The `Widget`'s [`Behavior`] flags; none are set on a new `Widget`.
    fn behavior(&self) -> Behavior {
        self.core().behavior
    }

    /// Replaces the `Widget`'s behavior flags, and invalidates it.
    ///
    /// Setting or clearing [`Behavior::HIDDEN`] hides or shows the `Widget`, the same as
    /// [`set_hidden`](Widget::set_hidden).
    ///
    /// # Arguments
    ///
    /// * `behavior` - The new flags.
    fn set_behavior(&mut self, behavior: Behavior) {
        self.core_mut().behavior = behavior;
        self.set_invalidated(true);
    }

    /// Returns how the `Widget` currently looks.
    ///
    /// # Returns
    ///
    /// The `Widget`'s [`VisualState`] flags; none are set (the normal state) on a new `Widget`.
    fn state(&self) -> VisualState {
        self.core().state
    }

    /// Replaces the `Widget`'s visual state flags, and invalidates it only if the change can alter
    /// how it looks.
    ///
    /// A change to `PRESSED`, `CHECKED`, `INDETERMINATE`, `DISABLED` or `FOCUSED` always repaints,
    /// since built-in looks and the state overlay depend on them.  A change to `HOVERED` or
    /// `DRAG_OVER` repaints only if the effective style (see [`Widget::style`]) changed.
    ///
    /// # Arguments
    ///
    /// * `state` - The new flags.
    fn set_state(&mut self, state: VisualState) {
        let old = self.core().state;

        if old == state {
            return;
        }

        let built_in = VisualState::PRESSED
            | VisualState::CHECKED
            | VisualState::INDETERMINATE
            | VisualState::DISABLED
            | VisualState::FOCUSED;
        let appearance = &self.core().appearance;
        let changed_look = appearance.effective(old) != appearance.effective(state);

        self.core_mut().state = state;

        if changed_look || (old ^ state).intersects(built_in) {
            self.set_invalidated(true);
        }
    }

    /// Returns the `Widget`'s per-state appearance.
    ///
    /// # Returns
    ///
    /// The [`Appearance`] holding one [`StateStyle`] per visual state.
    fn appearance(&self) -> &Appearance {
        &self.core().appearance
    }

    /// Returns the `Widget`'s per-state appearance for modification, and invalidates the `Widget`.
    ///
    /// Use [`Widget::set_style`] to change one state and repaint only when it matters.
    ///
    /// # Returns
    ///
    /// The [`Appearance`], to be edited in place.
    fn appearance_mut(&mut self) -> &mut Appearance {
        self.set_invalidated(true);
        &mut self.core_mut().appearance
    }

    /// Sets the style for one visual state, and invalidates the `Widget` only if the effective
    /// style for its current state changed.
    ///
    /// # Arguments
    ///
    /// * `state` - The visual state flag to style, such as [`VisualState::HOVERED`]; an empty set
    ///   styles the normal state.
    /// * `style` - The overrides for that state.
    fn set_style(&mut self, state: VisualState, style: StateStyle) {
        let current = self.core().state;
        let before = self.core().appearance.effective(current);

        self.core_mut().appearance.set(state, style);

        if self.core().appearance.effective(current) != before {
            self.set_invalidated(true);
        }
    }

    /// Returns the effective style for the `Widget`'s current state.
    ///
    /// The style for the highest-priority active state wins, unset fields fall back to `normal`,
    /// and fields still `None` mean "use the built-in default".  `paint` implementations read
    /// this.
    ///
    /// # Returns
    ///
    /// The resolved [`StateStyle`].
    fn style(&self) -> StateStyle {
        self.core().appearance.effective(self.core().state)
    }

    /// Returns whether the `Widget` is enabled.
    ///
    /// Disabled `Widget`s ([`VisualState::DISABLED`]) receive no input and cannot take focus.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` is enabled, `false` if it is disabled.
    fn is_enabled(&self) -> bool {
        !self.core().state.contains(VisualState::DISABLED)
    }

    /// Enables or disables the `Widget`, and invalidates it if that changed it.
    ///
    /// # Arguments
    ///
    /// * `enabled` - `true` to enable the `Widget`, `false` to disable it.
    fn set_enabled(&mut self, enabled: bool) {
        let mut state = self.core().state;

        state.set(VisualState::DISABLED, !enabled);
        self.set_state(state);
    }

    /// Returns whether the `Widget` can take keyboard focus.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` is [`Behavior::FOCUSABLE`], enabled, and not hidden.
    fn is_focusable(&self) -> bool {
        self.behavior().contains(Behavior::FOCUSABLE) && self.is_enabled() && !self.is_hidden()
    }

    /// Returns whether the `Widget` is a text editor.
    ///
    /// While a `Widget` that returns `true` has the keyboard focus, the `Engine` asks the
    /// operating system for text input, so typed characters arrive as
    /// [`Event::TextEntered`].  Override it in text fields and
    /// editors; the default is `false`.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` accepts typed text.
    fn accepts_text_input(&self) -> bool {
        false
    }

    /// Returns whether the `Widget` needs to be repainted.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` will be repainted into its texture on the next frame.
    fn is_invalidated(&self) -> bool {
        self.core().invalidated
    }

    /// Sets whether the `Widget` needs to be repainted.
    ///
    /// Setters call this automatically.  Call it after changing state that affects how the `Widget`
    /// looks.  An invalidated `Widget` is repainted into its texture on the next frame, and the screen
    /// is refreshed.
    ///
    /// # Arguments
    ///
    /// * `state` - `true` to repaint the `Widget` on the next frame.
    fn set_invalidated(&mut self, state: bool) {
        self.core_mut().invalidated = state;
    }
}
