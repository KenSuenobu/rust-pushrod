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

use crate::accessibility::{AccessInfo, Role};
use crate::appearance::{Appearance, StateStyle};
use crate::color::Color;
use crate::composite::Composite;
use crate::controllers::{Controller, ControllerEvent, Controllers};
use crate::error::Result;
use crate::event::{Event, EventCtx, EventMask, Propagation};
use crate::geometry::{Point, Rect, Size};
use crate::layout::{Layout, LayoutParams, SizeHint};
use crate::paint::Painter;
use crate::properties::{Property, core_properties, set_core_property};
use crate::registry::PropertyValue;
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
    /// The tooltip text; empty for none.
    tooltip: String,
    controllers: Controllers,
    /// The size the application asked for; unlike `size`, a layout never changes it, so the
    /// default size hint does not depend on where a layout placed the `Widget`.
    requested_size: Size,
    hint_override: Option<SizeHint>,
    layout_params: LayoutParams,
    layout: Option<LayoutSlot>,
    layout_dirty: bool,
    /// Whether a composite asked for its parts to be laid out again before the next frame.
    arrange_requested: bool,
    /// The point of the child area shown at the `Widget`'s upper left-hand corner; see
    /// [`Widget::child_offset`].
    pub(crate) child_offset: Point,
    /// The size of the area the children are placed on, when it differs from the `Widget`'s own
    /// size; see [`Widget::child_area`].
    pub(crate) child_area: Option<Size>,
}

/// Holds a container's layout manager, so `WidgetCore` can still derive `Debug`.
struct LayoutSlot(Box<dyn Layout>);

impl std::fmt::Debug for LayoutSlot {
    /// Formats the value for debugging.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Layout")
    }
}

impl WidgetCore {
    /// Applies a rectangle chosen by a layout manager.
    ///
    /// A move alone does not invalidate the `Widget`: its texture is still good, and the cache
    /// only copies it to the new place.  A resize invalidates it, so it is repainted at the new
    /// size, and marks it for layout if it is a container.  The size the application asked for
    /// is left alone.
    ///
    /// # Arguments
    ///
    /// * `rect` - The new origin and size, relative to the parent.
    ///
    /// # Returns
    ///
    /// `(moved, resized)`: whether the origin and the size changed.
    pub(crate) fn place(&mut self, rect: Rect) -> (bool, bool) {
        let origin = Point::new(rect.x, rect.y);
        let size = Size::new(rect.w, rect.h);
        let moved = self.origin != origin;
        let resized = self.size != size;

        self.origin = origin;
        self.size = size;

        if resized {
            self.invalidated = true;
            // A composite's parts follow its new size.
            self.arrange_requested = true;
        }

        (moved, resized)
    }

    /// Returns the container's layout manager, if it has one.
    ///
    /// # Returns
    ///
    /// The layout, or `None` for absolute positioning.
    pub(crate) fn layout_ref(&self) -> Option<&dyn Layout> {
        self.layout.as_ref().map(|slot| slot.0.as_ref())
    }

    /// Returns whether the `Widget` changed in a way that needs layout again.
    ///
    /// # Returns
    ///
    /// `true` if layout must run for this `Widget` or its container.
    pub(crate) fn layout_dirty(&self) -> bool {
        self.layout_dirty
    }

    /// Sets or clears the layout-needed flag.
    ///
    /// # Arguments
    ///
    /// * `dirty` - `true` if layout must run.
    pub(crate) fn set_layout_dirty(&mut self, dirty: bool) {
        self.layout_dirty = dirty;
    }

    /// Returns whether the `Widget` (a composite) asked for its parts to be laid out again.
    ///
    /// # Returns
    ///
    /// `true` after [`Widget::request_arrange_parts`], until the `WidgetCache` arranges them.
    pub(crate) fn arrange_requested(&self) -> bool {
        self.arrange_requested
    }

    /// Sets or clears the arrange-parts request.
    ///
    /// # Arguments
    ///
    /// * `requested` - `true` if the parts must be laid out again.
    pub(crate) fn set_arrange_requested(&mut self, requested: bool) {
        self.arrange_requested = requested;
    }

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
            tooltip: String::new(),
            controllers: Controllers::default(),
            requested_size: size,
            hint_override: None,
            layout_params: LayoutParams::default(),
            layout: None,
            layout_dirty: false,
            arrange_requested: false,
            child_offset: Point::new(0, 0),
            child_area: None,
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
    /// The new size is also the `Widget`'s default [`size_hint`](Widget::size_hint), and marks
    /// layout as needed.  A composite's parts are laid out again before the next frame, as if
    /// it had called [`request_arrange_parts`](Widget::request_arrange_parts).
    ///
    /// # Arguments
    ///
    /// * `size` - The new width and height in pixels.
    fn set_size(&mut self, size: Size) {
        let core = self.core_mut();

        core.size = size;
        core.requested_size = size;
        core.layout_dirty = true;
        core.arrange_requested = true;
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

    /// Returns the `Widget`'s tooltip text.
    ///
    /// # Returns
    ///
    /// The text, or an empty string if there is none.
    fn tooltip(&self) -> &str {
        &self.core().tooltip
    }

    /// Sets the `Widget`'s tooltip text.  It does not change how the `Widget` looks.
    ///
    /// # Arguments
    ///
    /// * `text` - The tooltip text; empty for none.
    fn set_tooltip(&mut self, text: &str) {
        self.core_mut().tooltip = text.to_string();
    }

    /// Lists the `Widget`'s properties: name, kind, and current value.
    ///
    /// The default lists the core properties (see [`properties`](crate::properties)).  A widget
    /// with properties of its own overrides this to add them to [`core_properties`], and
    /// [`set_property`](Widget::set_property) to set them.
    ///
    /// # Returns
    ///
    /// The properties.
    fn properties(&self) -> Vec<Property> {
        core_properties(self)
    }

    /// Sets a property by name.  Setting it to the value it already has changes nothing.
    ///
    /// To also emit [`EngineEvent::PropertyChanged`](crate::event::EngineEvent::PropertyChanged),
    /// call [`Engine::set_widget_property`](crate::engine::Engine::set_widget_property).
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed, `false` if the `Widget` already had it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Property`](crate::error::Error::Property) if there is no such property,
    /// it is read only, or the value has the wrong kind or is out of range.
    fn set_property(&mut self, name: &str, value: &PropertyValue) -> Result<bool> {
        set_core_property(self, name, value)
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
        let core = self.core_mut();

        core.behavior.set(Behavior::HIDDEN, state);
        core.layout_dirty = true;
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

    /// Returns the keyboard mnemonic the `Widget`'s label declares.
    ///
    /// A label marks its shortcut with `_` (`"_Save"`).  Alt plus that letter activates the
    /// `Widget` (see [`Widget::mnemonic_activates`]).  Override it in `Widget`s with a label, usually
    /// with [`paint::mnemonic_of`](crate::paint::mnemonic_of); the default is `None`.
    ///
    /// # Returns
    ///
    /// The lowercased mnemonic character, or `None` if the `Widget` has none.
    fn mnemonic(&self) -> Option<char> {
        None
    }

    /// Returns what the `Widget`'s mnemonic does.
    ///
    /// # Returns
    ///
    /// `true` (the default) if the mnemonic activates the `Widget`, which is sent
    /// [`Event::Activated`] (a button clicks, a check box toggles).  `false` if the `Widget` is a
    /// label, so the mnemonic moves keyboard focus to the next focusable `Widget` after it.
    fn mnemonic_activates(&self) -> bool {
        true
    }

    /// Describes the `Widget` to assistive technology such as a screen reader.
    ///
    /// The default is a [`Role::Generic`] node named by the
    /// tooltip, that is checked when the `Widget` is a [`Behavior::TOGGLE`] in the
    /// [`VisualState::CHECKED`] state, and that accepts focus when it is focusable.  Override it
    /// to give the `Widget` a [`Role`], a name, a value, and the
    /// actions it accepts; keep it cheap and free of side effects, since the `Engine` calls it
    /// to detect changes only while assistive technology is active.
    ///
    /// # Returns
    ///
    /// What a screen reader is told about the `Widget`.
    fn accessibility(&self) -> AccessInfo {
        let mut info = AccessInfo::of(self, Role::Generic);

        info.name = info.description.clone();
        info
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

    /// Returns where the text caret is, relative to the `Widget`'s upper left-hand corner.
    ///
    /// The `Engine` gives it to the backend while the `Widget` has the keyboard focus, so the
    /// platform's input-method candidate window appears next to the caret and follows it.  A text
    /// editor reports the caret as of its latest paint; the default is `None`.
    ///
    /// # Returns
    ///
    /// The caret rectangle in logical pixels, or `None` if the `Widget` has no caret.
    fn text_input_rect(&self) -> Option<Rect> {
        None
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

    /// Reports the `Widget`'s smallest, preferred, and largest size to a layout manager.
    ///
    /// The default is the override from [`Widget::set_size_hint`] if there is one; otherwise the
    /// size the application asked for ([`Widget::set_size`]) as the minimum and preferred size,
    /// with no maximum.  `Widget`s whose size depends on their content, such as text, override
    /// this and measure with the painter.
    ///
    /// # Arguments
    ///
    /// * `p` - A painter used only to measure text; nothing is drawn.
    ///
    /// # Returns
    ///
    /// The `Widget`'s [`SizeHint`].
    fn size_hint(&self, p: &mut dyn Painter) -> SizeHint {
        let _ = p;

        self.explicit_size_hint()
            .unwrap_or_else(|| SizeHint::fixed(self.core().requested_size))
    }

    /// Returns the hint set with [`Widget::set_size_hint`].  An overriding
    /// [`size_hint`](Widget::size_hint) returns it in preference to a measured one, so the
    /// application can always have the last word.
    ///
    /// # Returns
    ///
    /// The fixed hint, or `None` if none was set.
    fn explicit_size_hint(&self) -> Option<SizeHint> {
        self.core().hint_override
    }

    /// Sets a fixed [`SizeHint`], replacing the default one, and marks layout as needed.
    ///
    /// # Arguments
    ///
    /// * `hint` - The hint to report from now on.
    fn set_size_hint(&mut self, hint: SizeHint) {
        let core = self.core_mut();

        core.hint_override = Some(hint);
        core.layout_dirty = true;
    }

    /// Returns what the `Widget` asks of its container's layout.
    ///
    /// # Returns
    ///
    /// The [`LayoutParams`]; no stretch, start alignment, and no margin by default.
    fn layout_params(&self) -> LayoutParams {
        self.core().layout_params
    }

    /// Sets what the `Widget` asks of its container's layout, and marks layout as needed.
    ///
    /// # Arguments
    ///
    /// * `params` - The stretch, alignment, and margin.
    fn set_layout_params(&mut self, params: LayoutParams) {
        let core = self.core_mut();

        core.layout_params = params;
        core.layout_dirty = true;
    }

    /// Gives the `Widget` a layout manager, so it places its children instead of the application,
    /// and marks layout as needed.  Any container can have one.
    ///
    /// # Arguments
    ///
    /// * `layout` - The layout manager, such as one an application wrote.
    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        let core = self.core_mut();

        core.layout = Some(LayoutSlot(layout));
        core.layout_dirty = true;
    }

    /// Removes the `Widget`'s layout manager.  Its children keep the places they have, and from now
    /// on are positioned absolutely.
    fn remove_layout(&mut self) {
        let core = self.core_mut();

        core.layout = None;
        core.layout_dirty = true;
    }

    /// Returns whether the `Widget` has a layout manager.
    ///
    /// # Returns
    ///
    /// `true` if [`Widget::set_layout`] was called and the layout was not removed.
    fn has_layout(&self) -> bool {
        self.core().layout.is_some()
    }

    /// Tells the engine that something changed the `Widget`'s size hint, such as its text or font.
    /// Layout runs for the nearest container that has a layout before the next frame is painted.
    /// Setters that change a hint call it automatically.
    fn request_layout(&mut self) {
        self.core_mut().layout_dirty = true;
    }

    /// Asks the `WidgetCache` to lay a composite's parts out again with
    /// [`Composite::layout_parts`] before the next frame is drawn, as
    /// [`WidgetCache::arrange_parts`](crate::cache::WidgetCache::arrange_parts) does.
    ///
    /// A composite calls it when its own state moves a part, such as a scroll bar whose value
    /// changed in `handle_event` and whose thumb must follow; it has no access to the cache
    /// there.  Several requests before a frame arrange the parts once.  On a `Widget` that is not
    /// a composite the request is dropped.
    fn request_arrange_parts(&mut self) {
        self.core_mut().arrange_requested = true;
    }

    /// Returns the point of the child area that is shown at the `Widget`'s upper left-hand
    /// corner: how far its children are scrolled.
    ///
    /// A child at `origin` is drawn at `origin - child_offset`, relative to this `Widget`, and
    /// is still cropped to this `Widget`'s rectangle.  The offset moves the children on screen
    /// without repainting them.  Change it with
    /// [`WidgetCache::set_child_offset`](crate::cache::WidgetCache::set_child_offset).
    ///
    /// # Returns
    ///
    /// The offset; `(0, 0)` unless the `Widget` scrolls its children.
    fn child_offset(&self) -> Point {
        self.core().child_offset
    }

    /// Returns the size of the area the `Widget`'s children are placed on, when it differs from
    /// the `Widget`'s own size: a scrolling viewport's content.  A layout manager arranges the
    /// children in this area instead of in the `Widget`'s rectangle.  Change it with
    /// [`WidgetCache::set_child_area`](crate::cache::WidgetCache::set_child_area).
    ///
    /// # Returns
    ///
    /// The area's size, or `None` when the children are placed on the `Widget`'s own rectangle.
    fn child_area(&self) -> Option<Size> {
        self.core().child_area
    }

    /// Tells the `Widget` that something below it changed: a descendant was added, removed,
    /// moved, resized, repainted, or given the keyboard focus.  The `WidgetCache` calls it
    /// before a frame is laid out and drawn, at most once per frame, and when focus moves.
    ///
    /// A `Widget` that tracks its descendants, such as a scrolling container that grows with
    /// its content, overrides it, usually to call
    /// [`request_arrange_parts`](Widget::request_arrange_parts).  The default does nothing.
    fn descendant_changed(&mut self) {}

    /// Returns this `Widget` as a [`Composite`], if it is one.
    ///
    /// The `WidgetCache` calls this to build a composite's parts when it is added, and to route
    /// its parts' events to it.  A composite overrides it to return `Some(self)`; see the
    /// [`composite`](crate::composite) module.
    ///
    /// # Returns
    ///
    /// `None` by default.
    fn as_composite(&self) -> Option<&dyn Composite> {
        None
    }

    /// Returns this `Widget` as a mutable [`Composite`], if it is one.  A composite overrides it
    /// to return `Some(self)`, like [`Widget::as_composite`].
    ///
    /// # Returns
    ///
    /// `None` by default.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        None
    }
}
