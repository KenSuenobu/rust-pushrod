// Cache
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

//! # WidgetCache
//!
//! The `WidgetCache` stores every `Widget` in a window, much like the `OBJECT` tree of a GEM
//! form on the Atari ST: `Widget`s live in a flat list, are addressed by their `WidgetId` (their
//! index in the list), and are linked to their parents and children by ID.  The top-level
//! window `Widget` is always ID `0`.
//!
//! ```text
//!   ID 0  RootWidget (window)
//!    ├── ID 1  Panel
//!    │    ├── ID 2  Text      origins are relative to the parent
//!    │    └── ID 3  Button
//!    └── ID 4  Frame            later siblings are drawn on top
//! ```
//!
//! A flat list addressed by ID avoids reference-counted parent pointers entirely, and allows a
//! form to be stored and reloaded by ID.

use crate::backend::Backend;
use crate::color::Color;
use crate::error::{Error, Result};
use crate::event::{
    CustomEvent, EngineEvent, EngineEventKind, Event, EventCtx, EventMask, EventPhase, EventTarget,
    MaskScope, Propagation, engine_filter_accepts,
};
use crate::geometry::{Point, Rect, make_rect, point};
use crate::input::InputEvent;
use crate::layout::SizeHint;
use crate::paint::{self, CachePolicy};
use crate::registry::PropertyValue;
use crate::widget::{Behavior, ROOT_WIDGET_ID, VisualState, Widget, WidgetId};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

mod access;
mod dnd;
mod focus;
mod layout;
mod modal;
mod parts;
mod scroll;
mod tree;

pub use modal::ModalStyle;

/// The most times one frame lays out and arranges parts before drawing.  A composite whose
/// parts depend on layout (a scrolling container that fits its content) settles in two or
/// three passes; anything still pending waits for the next frame.
const MAX_SETTLE_PASSES: usize = 4;

/// The tree of `Widget`s in a window.
///
/// `Widget`s are stored in a flat list, addressed by ID, with each `Widget`'s children kept in
/// drawing order.  The cache also tracks the `Widget` under the mouse pointer and the `Widget`
/// that captured the mouse with a button press, routes raw input events to `Widget`s, and draws
/// them.
///
/// # Examples
///
/// ```rust
/// use pushrod::cache::WidgetCache;
/// use pushrod::geometry::size;
/// use pushrod::root_widget::RootWidget;
///
/// let mut cache = WidgetCache::new();
/// let root = cache.add(Box::new(RootWidget::new(size(320, 200))));
///
/// assert_eq!(root, 0);
/// assert_eq!(cache.len(), 1);
/// ```
pub struct WidgetCache {
    cache: Vec<Box<dyn Widget>>,
    children: Vec<Vec<WidgetId>>,
    current_widget_id: WidgetId,
    captured_widget_id: Option<WidgetId>,
    /// The modifier keys last reported, for [`EventCtx::modifiers`].
    held_modifiers: crate::input::Modifiers,
    needs_full_redraw: bool,
    cache_policy: CachePolicy,
    pending_events: Vec<Event>,
    loop_mask: EventMask,
    engine_filters: HashMap<WidgetId, Vec<EngineEventKind>>,
    focused_widget_id: Option<WidgetId>,
    text_input_wanted: bool,
    composing: Option<WidgetId>,
    composition_cleared: bool,
    animating: Vec<WidgetId>,
    clipboard_snapshot: Option<String>,
    clipboard_write: Option<String>,
    removed: HashSet<WidgetId>,
    released_textures: Vec<WidgetId>,
    layout_hints: HashMap<WidgetId, SizeHint>,
    modals: Vec<modal::ModalFrame>,
    modal_style: ModalStyle,
    dnd: dnd::DndState,
}

impl std::fmt::Debug for WidgetCache {
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
        f.debug_struct("WidgetCache")
            .field("widgets", &self.cache.len())
            .field("focused", &self.focused_widget_id)
            .field("loop_mask", &self.loop_mask)
            .finish_non_exhaustive()
    }
}

impl WidgetCache {
    /// Creates an empty `WidgetCache`.
    ///
    /// The first `Widget` added becomes the root `Widget` (ID `0`).
    ///
    /// # Returns
    ///
    /// A new, empty `WidgetCache`.
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            children: Vec::new(),
            current_widget_id: ROOT_WIDGET_ID,
            captured_widget_id: None,
            held_modifiers: crate::input::Modifiers::empty(),
            needs_full_redraw: true,
            cache_policy: CachePolicy::TextureCache,
            pending_events: Vec::new(),
            loop_mask: EventMask::ALL,
            engine_filters: HashMap::new(),
            focused_widget_id: None,
            text_input_wanted: false,
            composing: None,
            composition_cleared: false,
            animating: Vec::new(),
            clipboard_snapshot: None,
            clipboard_write: None,
            removed: HashSet::new(),
            released_textures: Vec::new(),
            layout_hints: HashMap::new(),
            modals: Vec::new(),
            modal_style: ModalStyle::default(),
            dnd: dnd::DndState::default(),
        }
    }

    /// Records a copy of the system clipboard for `Widget`s that paste.  The engine calls this
    /// while a text editor has the focus.
    ///
    /// # Arguments
    ///
    /// * `text` - The clipboard text, if any.
    pub fn set_clipboard_snapshot(&mut self, text: Option<String>) {
        self.clipboard_snapshot = text;
    }

    /// Takes the text a `Widget` asked to place on the clipboard since the last call.
    ///
    /// # Returns
    ///
    /// The text, or `None` if nothing was copied.
    pub fn take_clipboard_write(&mut self) -> Option<String> {
        self.clipboard_write.take()
    }

    /// Returns the cache policy of the backend the cache paints with.
    ///
    /// Defaults to [`CachePolicy::TextureCache`].  `draw_loop` implements `TextureCache` only;
    /// [`CachePolicy::DirectDraw`] is declared for backends that paint straight to the screen
    /// with clipping, and is not rendered by `draw_loop`.
    ///
    /// # Returns
    ///
    /// The [`CachePolicy`].
    pub fn cache_policy(&self) -> CachePolicy {
        self.cache_policy
    }

    /// Sets the cache policy, as declared by the backend.
    ///
    /// # Arguments
    ///
    /// * `policy` - The backend's [`CachePolicy`].
    pub fn set_cache_policy(&mut self, policy: CachePolicy) {
        self.cache_policy = policy;
    }

    /// Adds a `Widget` as a child of the root `Widget`.
    ///
    /// If the cache is empty, the `Widget` becomes the root `Widget` instead.
    ///
    /// # Arguments
    ///
    /// * `widget` - The `Widget` to add.
    ///
    /// # Returns
    ///
    /// The ID assigned to the `Widget`.
    pub fn add(&mut self, widget: Box<dyn Widget>) -> WidgetId {
        self.add_child(ROOT_WIDGET_ID, widget)
    }

    /// Adds a `Widget` as a child of another `Widget`.
    ///
    /// The `Widget`'s origin is relative to its parent's origin, and it is drawn after (on top of)
    /// the parent's existing children.  If the `Widget` is a [`Composite`](crate::composite::Composite), its
    /// parts are built right after it, with the next IDs, and laid out.  If the cache is empty, the `Widget` becomes the root `Widget`
    /// and `parent` is ignored.
    ///
    /// # Arguments
    ///
    /// * `parent` - The ID of the parent `Widget`.
    /// * `widget` - The `Widget` to add.
    ///
    /// # Returns
    ///
    /// The ID assigned to the `Widget`.
    ///
    /// # Panics
    ///
    /// Panics if the cache is not empty and `parent` does not exist.
    pub fn add_child(&mut self, parent: WidgetId, widget: Box<dyn Widget>) -> WidgetId {
        self.try_add_child(parent, widget)
            .unwrap_or_else(|error| panic!("[WidgetCache::add_child] {error}"))
    }

    /// Adds a `Widget` as the last child of `parent`, reporting an unknown parent as an error
    /// instead of panicking.  Behaves like [`add_child`](WidgetCache::add_child) otherwise.
    ///
    /// # Arguments
    ///
    /// * `parent` - The ID of the parent `Widget`.
    /// * `widget` - The `Widget` to add.
    ///
    /// # Returns
    ///
    /// The ID assigned to the `Widget`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownWidget`] if the cache is not empty and `parent` does not exist or
    /// was removed.
    pub fn try_add_child(
        &mut self,
        parent: WidgetId,
        mut widget: Box<dyn Widget>,
    ) -> Result<WidgetId> {
        let id = self.cache.len() as WidgetId;
        let parent = if self.cache.is_empty() {
            id
        } else if (parent as usize) < self.cache.len() && !self.removed.contains(&parent) {
            parent
        } else {
            return Err(Error::UnknownWidget { id: parent });
        };

        widget.core_mut().id = id;
        widget.core_mut().parent = parent;
        widget.set_invalidated(true);
        widget.request_layout();

        self.cache.push(widget);
        self.children.push(Vec::new());

        if parent != id {
            self.children[parent as usize].push(id);
        }

        self.needs_full_redraw = true;

        if parent != id {
            self.pending_events
                .push(Event::Engine(EngineEvent::WidgetAdded { id, parent }));
        }

        // A composite creates its parts now, so they follow it in ID order.
        self.build_composite(id);

        Ok(id)
    }

    /// Returns the number of `Widget`s in the cache.
    ///
    /// # Returns
    ///
    /// The number of `Widget`s, including the root `Widget`.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns whether the cache contains no `Widget`s.
    ///
    /// # Returns
    ///
    /// `true` if no `Widget` (not even a root) has been added.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Retrieves a `Widget` by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The `Widget` as a trait object, or `None` if the ID does not exist or the `Widget` was
    /// removed.
    pub fn get(&self, id: WidgetId) -> Option<&dyn Widget> {
        if self.removed.contains(&id) {
            return None;
        }

        self.cache.get(id as usize).map(|widget| widget.as_ref())
    }

    /// Retrieves a `Widget` by ID for modification.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The `Widget` as a mutable trait object, or `None` if the ID does not exist or the `Widget`
    /// was removed.
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        if self.removed.contains(&id) {
            return None;
        }

        match self.cache.get_mut(id as usize) {
            Some(widget) => Some(widget.as_mut()),
            None => None,
        }
    }

    /// Sets a `Widget`'s property by name, and queues
    /// [`EngineEvent::PropertyChanged`] when the value changed.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed, `false` if the `Widget` already had it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownWidget`] if the `Widget` does not exist, or the
    /// [`Error::Property`] from [`Widget::set_property`].
    pub fn set_property(
        &mut self,
        id: WidgetId,
        name: &str,
        value: &PropertyValue,
    ) -> Result<bool> {
        let widget = self.get_mut(id).ok_or(Error::UnknownWidget { id })?;
        let changed = widget.set_property(name, value)?;

        if changed {
            self.queue_event(Event::Engine(EngineEvent::PropertyChanged {
                id,
                name: name.to_string(),
            }));
        }

        Ok(changed)
    }

    /// Retrieves a `Widget` by ID as its concrete type.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The `Widget` as a `T`, or `None` if the ID does not exist or the `Widget` is not a `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::cache::WidgetCache;
    /// use pushrod::geometry::size;
    /// use pushrod::root_widget::RootWidget;
    ///
    /// let mut cache = WidgetCache::new();
    /// cache.add(Box::new(RootWidget::new(size(320, 200))));
    ///
    /// assert!(cache.get_as::<RootWidget>(0).is_some());
    /// ```
    pub fn get_as<T: Widget>(&self, id: WidgetId) -> Option<&T> {
        let widget: &dyn Any = self.cache.get(id as usize)?.as_ref();

        widget.downcast_ref::<T>()
    }

    /// Retrieves a `Widget` by ID as its concrete type, for modification.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The `Widget` as a mutable `T`, or `None` if the ID does not exist or the `Widget` is not a `T`.
    pub fn get_as_mut<T: Widget>(&mut self, id: WidgetId) -> Option<&mut T> {
        let widget: &mut dyn Any = self.cache.get_mut(id as usize)?.as_mut();

        widget.downcast_mut::<T>()
    }

    /// Returns the children of a `Widget`.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the parent `Widget`.
    ///
    /// # Returns
    ///
    /// The IDs of the `Widget`'s direct children in drawing order (the last is drawn on top), or an
    /// empty slice if the `Widget` has no children or does not exist.
    pub fn children_of(&self, id: WidgetId) -> &[WidgetId] {
        self.children
            .get(id as usize)
            .map(|children| children.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the `Widget` currently under the mouse pointer.
    ///
    /// # Returns
    ///
    /// The ID of the top-most visible `Widget` under the pointer, as of the last mouse event.
    pub fn get_current_widget(&self) -> WidgetId {
        self.current_widget_id
    }

    /// Forces the whole window to be redrawn on the next frame.
    ///
    /// Every visible `Widget`'s texture is copied to the screen again (textures are not repainted
    /// unless the `Widget` itself is invalidated).  Used after the window is exposed or resized.
    pub fn invalidate_all(&mut self) {
        self.needs_full_redraw = true;
    }

    /// Invalidates every `Widget`, so each is repainted into its texture on the next frame, and
    /// the whole window is redrawn.  Used when the scale factor changes: every texture has to be
    /// rendered again at the new physical resolution.
    pub fn repaint_all(&mut self) {
        for widget in &mut self.cache {
            widget.set_invalidated(true);
        }

        self.needs_full_redraw = true;
    }

    /// Returns the origin of a `Widget` in window coordinates.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The `Widget`'s upper left-hand corner in window coordinates: its origin plus the origins of
    /// all of its ancestors, less how far each ancestor has scrolled its children
    /// ([`Widget::child_offset`]).  Returns `(0, 0)` if the ID does not exist.
    pub fn absolute_origin(&self, id: WidgetId) -> Point {
        let mut origin = Point::new(0, 0);
        let mut current = id;

        while let Some(widget) = self.get(current) {
            let widget_origin = widget.get_origin();

            origin.x += widget_origin.x;
            origin.y += widget_origin.y;

            if widget.get_parent() == current {
                break;
            }

            current = widget.get_parent();

            if let Some(parent) = self.get(current) {
                let offset = parent.child_offset();

                origin.x -= offset.x;
                origin.y -= offset.y;
            }
        }

        origin
    }

    /// Returns the part of a `Widget` that can be seen and clicked, in window coordinates.
    ///
    /// This is the `Widget`'s own rectangle intersected with each ancestor's rectangle, so a child
    /// that sticks out of its parent is cropped.  A `Widget` with [`Behavior::OVERLAY`] is not
    /// cropped by its ancestors (its own children still are cropped to it).
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The visible rectangle, or `None` if the ID does not exist or nothing of the `Widget` is
    /// visible.
    pub fn visible_rect(&self, id: WidgetId) -> Option<Rect> {
        let widget = self.get(id)?;
        let mut visible = make_rect(self.absolute_origin(id), widget.get_size());
        let mut current = id;

        while !self.cache[current as usize]
            .behavior()
            .contains(Behavior::OVERLAY)
        {
            let parent = self.cache[current as usize].get_parent();

            if parent == current {
                break;
            }

            let bounds = make_rect(
                self.absolute_origin(parent),
                self.cache[parent as usize].get_size(),
            );

            visible = visible.intersection(&bounds)?;
            current = parent;
        }

        if visible.w == 0 || visible.h == 0 {
            return None;
        }

        Some(visible)
    }

    /// Returns the IDs of all visible `Widget`s in drawing order: parents before children, and
    /// siblings in the order in which they were added.  Hidden `Widget`s and their children are
    /// skipped.  Open modal dialogs come last, each after everything beneath it (see
    /// [`WidgetCache::open_modal`]).
    fn draw_order(&self) -> Vec<WidgetId> {
        if self.cache.is_empty() {
            return Vec::new();
        }

        self.draw_layers()
            .into_iter()
            .flat_map(|layer| layer.ids)
            .collect()
    }

    /// Finds the top-most visible `Widget` at a point in the window.
    ///
    /// Hidden `Widget`s and their children are skipped.
    ///
    /// # Arguments
    ///
    /// * `x` - The horizontal window coordinate, in pixels.
    /// * `y` - The vertical window coordinate, in pixels.
    ///
    /// # Returns
    ///
    /// The ID of the top-most visible `Widget` containing the point, or `0` (the root `Widget`) if no
    /// other `Widget` contains it.
    pub fn widget_at(&self, x: i32, y: i32) -> WidgetId {
        for id in self.draw_order().into_iter().rev() {
            if let Some(visible) = self.visible_rect(id)
                && visible.contains(x, y)
            {
                return id;
            }
        }

        ROOT_WIDGET_ID
    }

    /// Sets a `Widget`'s event mask and queues [`EngineEvent::EventMaskChanged`] when it changed.
    ///
    /// Queued events are returned by [`WidgetCache::take_pending_events`]; the `Engine` delivers
    /// them to the callbacks.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    /// * `mask` - The new mask.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` exists and its mask changed.
    pub fn set_event_mask(&mut self, id: WidgetId, mask: EventMask) -> bool {
        let Some(widget) = self.get_mut(id) else {
            return false;
        };
        let old = widget.event_mask();

        if old == mask {
            return false;
        }

        widget.core_mut().set_event_mask(mask);
        self.pending_events
            .push(Event::Engine(EngineEvent::EventMaskChanged {
                scope: MaskScope::Widget(id),
                old,
                new: mask,
            }));

        true
    }

    /// Limits the engine events a `Widget` receives to the listed kinds.
    ///
    /// The `Widget` still needs [`EventMask::ENGINE`] in its mask.  Without a filter it receives
    /// every kind.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    /// * `kinds` - The accepted kinds, or `None` to remove the filter.
    pub fn set_engine_filter(&mut self, id: WidgetId, kinds: Option<Vec<EngineEventKind>>) {
        match kinds {
            Some(kinds) => {
                self.engine_filters.insert(id, kinds);
            }
            None => {
                self.engine_filters.remove(&id);
            }
        }
    }

    /// Broadcasts a completed engine event to the `Widget`s that want it.
    ///
    /// A `Widget` receives it when it is enabled, the run loop's mask and its own mask include
    /// [`EventMask::ENGINE`], and its kind filter (if any) accepts the event.  Anything the
    /// `Widget`s generate in response is returned and is never broadcast again, so engine events
    /// cannot loop.
    ///
    /// # Arguments
    ///
    /// * `event` - The engine event.
    ///
    /// # Returns
    ///
    /// The events the `Widget`s generated while handling it.
    pub fn broadcast_engine_event(&mut self, event: &EngineEvent) -> Vec<Event> {
        let mut ctx = EventCtx::default();
        let wrapped = Event::Engine(event.clone());

        if self.loop_mask.intersects(EventMask::ENGINE) {
            for (index, widget) in self.cache.iter_mut().enumerate() {
                let filter = self.engine_filters.get(&(index as WidgetId));

                if widget.is_enabled()
                    && widget.event_mask().intersects(EventMask::ENGINE)
                    && engine_filter_accepts(filter.map(|k| k.as_slice()), event)
                {
                    ctx.set_source(Some(index as WidgetId));
                    widget.handle_event(&wrapped, &mut ctx);
                }
            }
        }

        self.finish(ctx)
    }

    /// Broadcasts an event to every `Widget` that wants it, in drawing order.
    ///
    /// A `Widget` receives the event when it is visible (neither it nor an ancestor is hidden),
    /// enabled, and both the run loop's mask and its own mask include the event's class.  Events
    /// that are never filtered (see [`EventKind::mask_class`](crate::event::EventKind::mask_class)) reach every visible, enabled
    /// `Widget`.  Delivery order is deterministic: parents before children, siblings in the order
    /// they were added.  Events the `Widget`s generate in response are returned, never
    /// broadcast again.
    ///
    /// Keyboard events are not broadcast; they belong to the focused `Widget`.  Queues
    /// [`EngineEvent::EventBroadcast`] describing the delivery.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to broadcast.
    ///
    /// # Returns
    ///
    /// The events the `Widget`s generated while handling it, in the order emitted.
    pub fn broadcast(&mut self, event: &Event) -> Vec<Event> {
        let mut ctx = EventCtx::default();
        let class = event.kind().mask_class();
        let loop_mask = self.loop_mask;
        let mut recipients = 0;

        if class.is_none_or(|class| loop_mask.intersects(class)) {
            for id in self.draw_order() {
                let widget = &mut self.cache[id as usize];

                if widget.is_enabled()
                    && class.is_none_or(|class| widget.event_mask().intersects(class))
                {
                    ctx.set_source(Some(id));
                    widget.handle_event(event, &mut ctx);
                    recipients += 1;
                }
            }
        }

        self.pending_events
            .push(Event::Engine(EngineEvent::EventBroadcast {
                kind: event.kind(),
                recipients,
            }));

        self.finish(ctx)
    }

    /// Sets the run loop's event mask, which `Widget`s must also pass.  The `Engine` calls this
    /// whenever its own mask changes.
    ///
    /// # Arguments
    ///
    /// * `mask` - The run loop's mask.
    pub(crate) fn set_loop_mask(&mut self, mask: EventMask) {
        self.loop_mask = mask;
    }

    /// Queues an engine event for [`WidgetCache::take_pending_events`].
    ///
    /// # Arguments
    ///
    /// * `event` - The event to queue.
    pub(crate) fn queue_event(&mut self, event: Event) {
        self.pending_events.push(event);
    }

    /// Ends a round of event handling: records the animation requests the `Widget`s made and
    /// returns the events they generated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context the `Widget`s handled events with.
    ///
    /// # Returns
    ///
    /// The events the `Widget`s generated, in the order emitted.
    pub(crate) fn finish(&mut self, mut ctx: EventCtx) -> Vec<Event> {
        self.animating.extend(ctx.take_animation_requests());

        if let Some(text) = ctx.take_clipboard_write() {
            // A paste right after a copy sees the copied text, before the backend has it.
            self.clipboard_snapshot = Some(text.clone());
            self.clipboard_write = Some(text);
        }

        let events = self.apply_exclusive(ctx.into_events());

        self.dismiss_on(&events);

        events
    }

    /// Asks for an [`Event::Tick`] for a `Widget` on the next frame, from outside event handling,
    /// for example to start an animation from a callback.  Inside `handle_event`, use
    /// [`EventCtx::request_animation`].
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` to tick.  Unknown IDs are ignored.
    pub fn request_animation(&mut self, id: WidgetId) {
        if (id as usize) < self.cache.len() {
            self.animating.push(id);
        }
    }

    /// Returns whether any `Widget` has asked for a tick on the next frame.
    ///
    /// # Returns
    ///
    /// `true` while an animation request is waiting.
    pub fn animation_requested(&self) -> bool {
        !self.animating.is_empty()
    }

    /// Delivers [`Event::Tick`] to the `Widget`s that requested animation since the last tick,
    /// and only to those.  The requests are consumed: a `Widget` that keeps animating requests
    /// again while handling the tick.  A `Widget` also needs [`EventMask::TIMER`] in its mask (and
    /// the run loop's), and must be enabled and visible.  Queues [`EngineEvent::AnimationTicked`]
    /// when at least one `Widget` received the tick.
    ///
    /// # Arguments
    ///
    /// * `elapsed` - The time since the previous frame, carried by the tick.
    ///
    /// # Returns
    ///
    /// The events the `Widget`s generated while handling the tick, in order.
    pub fn tick(&mut self, elapsed: Duration) -> Vec<Event> {
        let requested = std::mem::take(&mut self.animating);
        let mut ctx = EventCtx::default();
        let mut recipients = 0;

        if !requested.is_empty() && self.loop_mask.intersects(EventMask::TIMER) {
            let event = Event::Tick { elapsed };

            for id in self.draw_order() {
                let widget = &mut self.cache[id as usize];

                if requested.contains(&id)
                    && widget.is_enabled()
                    && widget.event_mask().intersects(EventMask::TIMER)
                {
                    ctx.set_source(Some(id));
                    widget.handle_event(&event, &mut ctx);
                    recipients += 1;
                }
            }
        }

        if recipients > 0 {
            self.pending_events
                .push(Event::Engine(EngineEvent::AnimationTicked {
                    recipients,
                    elapsed,
                }));
        }

        self.finish(ctx)
    }

    /// Removes and returns the engine events queued by the cache itself, in order.
    ///
    /// # Returns
    ///
    /// The queued events; empty if there are none.
    pub fn take_pending_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns whether engine events are waiting for [`WidgetCache::take_pending_events`].
    ///
    /// # Returns
    ///
    /// `true` if at least one event is queued.
    pub(crate) fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    /// Returns whether the next [`draw_loop`](WidgetCache::draw_loop) has anything to do.
    ///
    /// # Returns
    ///
    /// `true` if a full redraw is pending, any `Widget` is invalidated, or layout is needed.
    pub fn needs_draw(&self) -> bool {
        self.needs_full_redraw
            || self.cache.iter().any(|w| {
                w.is_invalidated() || w.core().layout_dirty() || w.core().arrange_requested()
            })
    }

    /// Sends an event to a `Widget`.  Disabled and hidden `Widget`s, and `Widget`s whose event mask
    /// excludes the event's class, receive nothing.
    ///
    /// # Arguments
    ///
    /// * `id` - The receiving `Widget`.
    /// * `event` - The event.
    /// * `phase` - The phase of delivery, readable through [`EventCtx::phase`].
    /// * `ctx` - The context collecting generated events.
    ///
    /// # Returns
    ///
    /// What the `Widget` returned, or [`Propagation::Continue`] if it received nothing.
    fn send(
        &mut self,
        id: WidgetId,
        event: &Event,
        phase: EventPhase,
        ctx: &mut EventCtx,
    ) -> Propagation {
        let loop_mask = self.loop_mask;
        let origin = self.absolute_origin(id);

        if let Some(widget) = self.get_mut(id)
            && widget.is_enabled()
            && !widget.is_hidden()
            && event.kind().mask_class().is_none_or(|class| {
                loop_mask.intersects(class) && widget.event_mask().intersects(class)
            })
        {
            ctx.set_source(Some(id));
            ctx.set_origin(origin);
            ctx.set_phase(phase);

            let result = widget.handle_event(event, ctx);

            ctx.set_phase(EventPhase::Target);

            return result;
        }

        Propagation::Continue
    }

    /// Sends an input event to a `Widget`, unless a modal dialog makes it inert.  See
    /// [`WidgetCache::send`].
    ///
    /// # Arguments
    ///
    /// * `id` - The receiving `Widget`.
    /// * `event` - The input event.
    /// * `phase` - The phase of delivery.
    /// * `ctx` - The context collecting generated events.
    ///
    /// # Returns
    ///
    /// What the `Widget` returned, or [`Propagation::Continue`] if it received nothing.
    fn send_input(
        &mut self,
        id: WidgetId,
        event: &Event,
        phase: EventPhase,
        ctx: &mut EventCtx,
    ) -> Propagation {
        if self.is_inert(id) {
            return Propagation::Continue;
        }

        self.send(id, event, phase, ctx)
    }

    /// Returns the ancestors of a `Widget`, nearest parent first, ending at the root.
    fn ancestors(&self, id: WidgetId) -> Vec<WidgetId> {
        let mut path = Vec::new();
        let mut current = id;

        while let Some(widget) = self.get(current) {
            let parent = widget.get_parent();

            if parent == current || path.contains(&parent) {
                break;
            }

            path.push(parent);
            current = parent;
        }

        path
    }

    /// Delivers an input event to `target` through the capture, target, and bubble phases.
    ///
    /// 1. **Capture:** from the root down to the target's parent, only `Widget`s whose
    ///    [`handles_capture`](Widget::handles_capture) is `true`.
    /// 2. **Target:** the `Widget` itself.
    /// 3. **Bubble:** from the target's parent up to the root.
    ///
    /// [`Propagation::Stop`] from any `Widget` ends delivery.  Disabled and hidden `Widget`s,
    /// those whose mask excludes the event, and those a modal dialog makes inert (see
    /// [`WidgetCache::is_inert`]) are skipped.  Events that do not propagate (see
    /// [`EventKind::propagates`](crate::event::EventKind::propagates)) go to the target only.
    ///
    /// # Arguments
    ///
    /// * `target` - The `Widget` the event is for: under the pointer, capturing the mouse, or
    ///   focused.
    /// * `event` - The event.
    /// * `ctx` - The context collecting generated events.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] if a `Widget` stopped delivery, otherwise [`Propagation::Continue`].
    pub fn propagate_with(
        &mut self,
        target: WidgetId,
        event: &Event,
        ctx: &mut EventCtx,
    ) -> Propagation {
        if !event.kind().propagates() {
            return self.send_input(target, event, EventPhase::Target, ctx);
        }

        let ancestors = self.ancestors(target);

        for &id in ancestors.iter().rev() {
            if self.get(id).is_some_and(|w| w.handles_capture())
                && self.send_input(id, event, EventPhase::Capture, ctx) == Propagation::Stop
            {
                return Propagation::Stop;
            }
        }

        if self.send_input(target, event, EventPhase::Target, ctx) == Propagation::Stop {
            return Propagation::Stop;
        }

        for &id in &ancestors {
            if self.send_input(id, event, EventPhase::Bubble, ctx) == Propagation::Stop {
                return Propagation::Stop;
            }
        }

        Propagation::Continue
    }

    /// Delivers an input event, such as a key press for the focused `Widget`, through the
    /// capture, target, and bubble phases.  See [`WidgetCache::propagate_with`].
    ///
    /// # Arguments
    ///
    /// * `target` - The `Widget` the event is for.
    /// * `event` - The event.
    ///
    /// # Returns
    ///
    /// The events the `Widget`s generated while handling it, in the order emitted.
    pub fn propagate(&mut self, target: WidgetId, event: &Event) -> Vec<Event> {
        let mut ctx = EventCtx::default();

        self.propagate_with(target, event, &mut ctx);
        self.finish(ctx)
    }

    /// Delivers a custom event to the `Widget`s its target names.
    ///
    /// A `Widget` receives it when [`EventMask::CUSTOM`] is in both the run loop's mask and its
    /// own mask.  Custom events are not input, so disabled `Widget`s still receive them; a
    /// [`Broadcast`](EventTarget::Broadcast) reaches only visible `Widget`s.  Delivery is in
    /// drawing order, parents before children.  The core takes no other action: no focus, mouse
    /// capture, selection, or redraw changes.
    ///
    /// # Arguments
    ///
    /// * `custom` - The custom event.
    ///
    /// # Returns
    ///
    /// The events the `Widget`s generated while handling it, in the order emitted (including any
    /// custom events they posted, which the caller queues).
    pub fn deliver_custom(&mut self, custom: &CustomEvent) -> Vec<Event> {
        let mut ctx = EventCtx::default();

        if !self.loop_mask.intersects(EventMask::CUSTOM) {
            return self.finish(ctx);
        }

        let recipients = match custom.target {
            EventTarget::ApplicationOnly => Vec::new(),
            EventTarget::Widget(id) => vec![id],
            EventTarget::Broadcast => self.draw_order(),
            EventTarget::Subtree(root) => {
                let mut ids = Vec::new();
                let mut stack = vec![root];

                while let Some(id) = stack.pop() {
                    if (id as usize) < self.cache.len() {
                        ids.push(id);
                        stack.extend(self.children[id as usize].iter().rev());
                    }
                }

                ids
            }
        };
        let event = Event::Custom(custom.clone());

        for id in recipients {
            if let Some(widget) = self.cache.get_mut(id as usize)
                && widget.event_mask().intersects(EventMask::CUSTOM)
            {
                ctx.set_source(Some(id));
                widget.handle_event(&event, &mut ctx);
            }
        }

        self.finish(ctx)
    }

    /// Routes a raw input event to the `Widget`s.
    ///
    /// - A mouse button press sends `PointerPressed` to the top-most `Widget` under the pointer,
    ///   which captures the mouse until the button is released.
    /// - Mouse motion sends `PointerExited` to the `Widget` that was left and `PointerEntered` to
    ///   the `Widget` that was entered (when they differ), then `PointerMoved` to the capturing
    ///   `Widget`, or the `Widget` under the pointer.
    /// - A mouse button release sends `PointerReleased` to the capturing `Widget`, then ends the
    ///   capture.
    /// - Presses, releases, and motion travel through the capture, target, and bubble phases (see
    ///   [`WidgetCache::propagate_with`]); hover events go to one `Widget`.
    /// - A mouse button press first focuses the nearest focusable `Widget` at or above the one
    ///   pressed.
    /// - The wheel sends `Scrolled` to the `Widget` under the pointer (as of the last mouse
    ///   event); it bubbles, so a scrolling container sees scrolls over its children.
    /// - Key presses and releases go to the focused `Widget` (or the root with no focus), then
    ///   Tab, the arrow keys, Return, and Escape act as described in [`WidgetCache::focus_next`],
    ///   [`WidgetCache::focus_in_group`], and [`WidgetCache::set_focus`].  Typed text goes to the
    ///   focused `Widget`.
    /// - Other events are ignored.
    /// - Disabled `Widget`s receive none of these events, but still block the `Widget`s beneath them.
    ///
    /// # Arguments
    ///
    /// * `event` - The input event from the backend.
    ///
    /// # Returns
    ///
    /// The `Event`s the `Widget`s generated while handling the event, in the order emitted.
    pub fn handle_event(&mut self, event: InputEvent) -> Vec<Event> {
        let mut ctx = EventCtx::default();

        match &event {
            InputEvent::ModifiersChanged(modifiers)
            | InputEvent::KeyDown { modifiers, .. }
            | InputEvent::KeyUp { modifiers, .. } => self.held_modifiers = *modifiers,
            _ => {}
        }

        ctx.set_modifiers(self.held_modifiers);
        ctx.set_clipboard_snapshot(self.clipboard_snapshot.clone());

        if self.cache.is_empty() {
            return self.finish(ctx);
        }

        // A cleared composition is committed by the text that follows at once (`Text`); any other
        // event means it was cancelled.
        if self.composition_cleared
            && !matches!(event, InputEvent::Text(_) | InputEvent::Composition { .. })
        {
            self.end_composition(false);
        }

        match event {
            InputEvent::MouseDown {
                position: Point { x, y },
                button: mouse_btn,
                clicks,
            } => {
                self.current_widget_id = self.widget_at(x, y);

                let target = self.current_widget_id;

                if mouse_btn == crate::input::MouseButton::Left {
                    self.dnd_pointer_down(target, point(x, y));
                }

                self.captured_widget_id = Some(target);
                self.set_state_flag(target, VisualState::PRESSED, true);
                self.focus_for_click(target);
                self.propagate_with(
                    target,
                    &Event::PointerPressed {
                        id: target,
                        button: mouse_btn,
                        position: point(x, y),
                        count: clicks,
                    },
                    &mut ctx,
                );
            }

            InputEvent::MouseUp {
                position: Point { x, y },
                button: mouse_btn,
                clicks,
            } => {
                if self.dnd_pointer_up(point(x, y), &mut ctx) {
                    return self.finish(ctx);
                }

                self.current_widget_id = self.widget_at(x, y);

                let target = self
                    .captured_widget_id
                    .take()
                    .unwrap_or(self.current_widget_id);

                self.set_state_flag(target, VisualState::PRESSED, false);
                self.propagate_with(
                    target,
                    &Event::PointerReleased {
                        id: target,
                        button: mouse_btn,
                        position: point(x, y),
                        count: clicks,
                    },
                    &mut ctx,
                );
            }

            InputEvent::MouseMove {
                position: Point { x, y },
            } => {
                if self.dnd_pointer_move(point(x, y)) {
                    return self.finish(ctx);
                }

                let previous_widget_id = self.current_widget_id;

                self.current_widget_id = self.widget_at(x, y);

                if self.current_widget_id != previous_widget_id {
                    self.set_state_flag(previous_widget_id, VisualState::HOVERED, false);
                    self.set_state_flag(self.current_widget_id, VisualState::HOVERED, true);
                    self.send(
                        previous_widget_id,
                        &Event::PointerExited {
                            id: previous_widget_id,
                        },
                        EventPhase::Target,
                        &mut ctx,
                    );
                    self.send_input(
                        self.current_widget_id,
                        &Event::PointerEntered {
                            id: self.current_widget_id,
                        },
                        EventPhase::Target,
                        &mut ctx,
                    );
                }

                if let Some(captured) = self.captured_widget_id {
                    // A held button stays pressed only while the pointer is over the widget.
                    self.set_state_flag(
                        captured,
                        VisualState::PRESSED,
                        captured == self.current_widget_id,
                    );
                }

                let target = self.captured_widget_id.unwrap_or(self.current_widget_id);

                self.propagate_with(
                    target,
                    &Event::PointerMoved {
                        id: target,
                        position: point(x, y),
                    },
                    &mut ctx,
                );
            }

            InputEvent::Wheel { delta } => {
                let target = self.current_widget_id;

                self.propagate_with(target, &Event::Scrolled { id: target, delta }, &mut ctx);
            }

            InputEvent::KeyDown {
                key: crate::input::Key::Escape,
                ..
            } if self.dnd_cancel() => {}

            InputEvent::FilesDropped { position, paths } => {
                self.dnd_files(position, paths, &mut ctx);
            }

            InputEvent::KeyDown {
                key,
                modifiers: keymod,
                repeat,
            } => self.handle_key_down(key, keymod, repeat, &mut ctx),

            InputEvent::KeyUp {
                key,
                modifiers: keymod,
            } => {
                let target = self.key_target();

                self.propagate_with(
                    target,
                    &Event::KeyReleased {
                        id: target,
                        key,
                        modifiers: keymod,
                    },
                    &mut ctx,
                );
            }

            InputEvent::Text(text) => {
                self.end_composition(true);

                if let Some(target) = self.focused_widget_id {
                    self.propagate_with(target, &Event::TextEntered { id: target, text }, &mut ctx);
                }
            }

            InputEvent::Composition {
                text,
                start,
                length,
            } => {
                if let Some(target) = self.focused_widget_id {
                    self.track_composition(target, &text, start, length);

                    let cursor =
                        (!text.is_empty()).then(|| (start.max(0) as usize, length.max(0) as usize));

                    self.propagate_with(
                        target,
                        &Event::TextComposing {
                            id: target,
                            text,
                            cursor,
                        },
                        &mut ctx,
                    );
                }
            }

            _ => {}
        }

        self.finish(ctx)
    }

    /// Follows an input method's composition for `target` and queues the composition engine
    /// events: `CompositionStarted` on the first non-empty text, `CompositionUpdated` on every
    /// non-empty text.  An empty text only marks the composition cleared; whether it ended by
    /// commit or by cancel is decided by what comes next (see [`WidgetCache::end_composition`]).
    ///
    /// # Arguments
    ///
    /// * `target` - The focused `Widget`.
    /// * `text` - The composition text.
    /// * `start` - Where the input method's cursor or selection starts, in characters.
    /// * `length` - How many characters the selection covers.
    fn track_composition(&mut self, target: WidgetId, text: &str, start: i32, length: i32) {
        if text.is_empty() {
            self.composition_cleared = self.composing == Some(target);
            return;
        }

        if self.composition_cleared || self.composing.is_some_and(|id| id != target) {
            self.end_composition(false);
        }

        if self.composing != Some(target) {
            self.composing = Some(target);
            self.queue_event(Event::Engine(EngineEvent::CompositionStarted {
                id: target,
            }));
        }

        self.queue_event(Event::Engine(EngineEvent::CompositionUpdated {
            id: target,
            text: text.to_string(),
            cursor: Some((start.max(0) as usize, length.max(0) as usize)),
        }));
    }

    /// Ends the composition in progress, if any, and queues `CompositionEnded`.  Called with
    /// `committed` set when the input method's final text arrives, and cleared when the composition
    /// is cancelled: the keyboard focus moved, or something other than the commit followed an
    /// empty composition.
    ///
    /// # Arguments
    ///
    /// * `committed` - `true` if the text was committed.
    pub(crate) fn end_composition(&mut self, committed: bool) {
        self.composition_cleared = false;

        if let Some(id) = self.composing.take() {
            self.queue_event(Event::Engine(EngineEvent::CompositionEnded {
                id,
                committed,
            }));
        }
    }

    /// Returns where the focused text editor's caret is, in window coordinates, for the platform's
    /// candidate window.
    ///
    /// # Returns
    ///
    /// The rectangle, or `None` when no focused `Widget` is a text editor that reports one (see
    /// [`Widget::text_input_rect`]).
    pub fn focused_text_input_rect(&self) -> Option<Rect> {
        let id = self.focused_widget_id?;
        let local = self.get(id)?.text_input_rect()?;
        let origin = self.absolute_origin(id);

        Some(Rect::new(
            local.x + origin.x,
            local.y + origin.y,
            local.w,
            local.h,
        ))
    }

    /// Returns the `Widget` an input method is composing text for.
    ///
    /// # Returns
    ///
    /// The ID, or `None` when nothing is being composed.
    pub fn composing(&self) -> Option<WidgetId> {
        self.composing
    }

    /// Sets or clears one [`VisualState`] flag on a `Widget`, repainting it only if its look
    /// changes (see [`Widget::set_state`]).  A `Widget` a modal dialog makes inert can have a
    /// flag cleared but not set, so it never looks hovered or pressed.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`; an unknown ID is ignored.
    /// * `flag` - The flag to change.
    /// * `on` - `true` to set the flag, `false` to clear it.
    pub(crate) fn set_state_flag(&mut self, id: WidgetId, flag: VisualState, on: bool) {
        if on && self.is_inert(id) {
            return;
        }

        if let Some(widget) = self.get_mut(id) {
            let mut state = widget.state();

            state.set(flag, on);
            widget.set_state(state);
        }
    }

    /// Paints a single `Widget` into its texture through the backend.
    fn paint(&mut self, id: WidgetId, backend: &mut dyn Backend) {
        let widget = &mut self.cache[id as usize];
        let size = widget.get_size();

        if size.w == 0 || size.h == 0 {
            return;
        }

        let painter: &dyn Widget = widget.as_ref();

        backend.paint_widget(id, size, &mut |p| {
            painter.paint(p);
            paint::state_overlay(p, size, painter.state());
        });
    }

    /// Queues the engine events the backend produced while painting (fonts loaded, glyph atlas
    /// pages added).
    fn queue_backend_notices(&mut self, backend: &mut dyn Backend) {
        for event in backend.take_notices() {
            self.queue_event(Event::Engine(event));
        }
    }

    /// Repaints changed `Widget`s and redraws the window if anything changed.
    ///
    /// The textures of `Widget`s removed since the last frame ([`WidgetCache::remove`]) are
    /// destroyed first, with [`Backend::release_widget`].
    ///
    /// Containers whose layout is out of date are measured and arranged first, and composites
    /// that asked for it lay their parts out again, so painting sees the final sizes (see the
    /// [`layout`](crate::layout) and [`composite`](crate::composite) modules).
    ///
    /// Every invalidated, visible `Widget` is repainted into its texture.  If any `Widget` changed (or
    /// a full redraw was requested), the window is cleared and every visible `Widget`'s texture is
    /// copied to the screen in drawing order.  Copying cached textures is cheap; repainting only
    /// happens for `Widget`s that changed.
    ///
    /// # Arguments
    ///
    /// * `backend` - The backend that owns the window and the `Widget` textures.
    ///
    /// # Returns
    ///
    /// `true` if a frame was drawn and the backend should present it, `false` if nothing
    /// changed.
    pub fn draw_loop(&mut self, backend: &mut dyn Backend) -> bool {
        // Removed `Widget`s never draw again, so their textures are destroyed first.
        for id in self.released_textures.drain(..) {
            backend.release_widget(id);
        }

        self.notify_descendant_changes();

        // Arranging parts can ask for layout (a resized part), and layout for arranging (a
        // container that fits its content), so the two alternate until both are settled.
        for _ in 0..MAX_SETTLE_PASSES {
            self.run_anchors();
            self.run_layout(backend);

            if !self.arrange_requested_parts() {
                break;
            }
        }

        let layers = if self.cache.is_empty() {
            Vec::new()
        } else {
            self.draw_layers()
        };
        let mut visible = vec![false; self.cache.len()];
        let mut redraw = self.needs_full_redraw;

        for id in layers.iter().flat_map(|layer| &layer.ids) {
            visible[*id as usize] = true;
        }

        for (id, is_visible) in visible.into_iter().enumerate() {
            // A `Widget` that is not visible (hidden, or inside a hidden ancestor) is not painted,
            // so it stays invalidated until it is: showing its ancestor invalidates only the
            // ancestor, and the parts below it (a scroll bar's arrows and track) must still be
            // painted the first time they are seen.
            if is_visible && self.cache[id].is_invalidated() {
                self.paint(id as WidgetId, backend);
                self.cache[id].set_invalidated(false);
                redraw = true;
            }
        }

        self.queue_backend_notices(backend);

        if !redraw {
            return false;
        }

        backend.begin_frame(Color::WHITE);

        for layer in layers {
            if let Some(root) = layer.modal {
                self.draw_modal_backdrop(root, backend);
            }

            self.draw_layer(&layer.ids, backend);
        }

        self.needs_full_redraw = false;

        true
    }

    /// Copies the cached textures of one layer's `Widget`s into the frame, cropped to their
    /// ancestors' bounds.
    ///
    /// # Arguments
    ///
    /// * `ids` - The layer's `Widget`s in drawing order.
    /// * `backend` - The backend composing the frame.
    fn draw_layer(&self, ids: &[WidgetId], backend: &mut dyn Backend) {
        for &id in ids {
            let Some(dest) = self.visible_rect(id) else {
                continue;
            };
            let origin = self.absolute_origin(id);
            let source = Rect::new(dest.x - origin.x, dest.y - origin.y, dest.w, dest.h);

            backend.draw_widget(id, source, dest);
        }
    }
}

impl Default for WidgetCache {
    /// Creates an empty `WidgetCache`; the same as [`WidgetCache::new`].
    ///
    /// # Returns
    ///
    /// A new, empty `WidgetCache`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod broadcast_tests {
    use super::*;
    use crate::event::EngineEvent;
    use crate::geometry::{Size, size};
    use crate::root_widget::RootWidget;
    use crate::widget::WidgetCore;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A `Widget` that records its ID whenever it receives an event.
    struct Recorder {
        core: WidgetCore,
        log: Rc<RefCell<Vec<WidgetId>>>,
    }

    impl Widget for Recorder {
        /// Returns the shared state.
        ///
        /// # Returns
        ///
        /// The `WidgetCore`.
        fn core(&self) -> &WidgetCore {
            &self.core
        }

        /// Returns the shared state for modification.
        ///
        /// # Returns
        ///
        /// The `WidgetCore`.
        fn core_mut(&mut self) -> &mut WidgetCore {
            &mut self.core
        }

        /// Draws nothing.
        ///
        /// # Arguments
        ///
        /// * `_p` - The painter.
        fn paint(&self, _p: &mut dyn crate::paint::Painter) {}

        /// Records the widget's ID.
        ///
        /// # Arguments
        ///
        /// * `_event` - The event.
        /// * `_ctx` - The context.
        ///
        /// # Returns
        ///
        /// [`Propagation::Continue`].
        fn handle_event(&mut self, _event: &Event, _ctx: &mut EventCtx) -> Propagation {
            self.log.borrow_mut().push(self.get_id());
            Propagation::Continue
        }
    }

    /// Creates a recording `Widget` with a mask.
    fn recorder(log: &Rc<RefCell<Vec<WidgetId>>>, mask: EventMask) -> Box<Recorder> {
        let mut core = WidgetCore::new(point(0, 0), size(10, 10));

        core.set_event_mask(mask);

        Box::new(Recorder {
            core,
            log: log.clone(),
        })
    }

    /// Creates a cache holding a root `Widget`, with every class enabled in the run loop.
    fn cache() -> WidgetCache {
        let mut cache = WidgetCache::new();

        cache.add(Box::new(RootWidget::new(Size::new(100, 100))));
        cache.set_loop_mask(EventMask::ALL);
        cache
    }

    fn focus() -> Event {
        Event::WindowFocusChanged { focused: true }
    }

    #[test]
    fn delivery_is_in_draw_order_parents_before_children() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut cache = cache();
        let a = cache.add(recorder(&log, EventMask::WINDOW));
        let b = cache.add(recorder(&log, EventMask::WINDOW));
        let a_child = cache.add_child(a, recorder(&log, EventMask::WINDOW));

        cache.broadcast(&focus());

        assert_eq!(*log.borrow(), vec![a, a_child, b]);
    }

    #[test]
    fn an_unknown_parent_is_an_error_not_a_panic() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut cache = cache();
        let before = cache.len();

        let result = cache.try_add_child(99, recorder(&log, EventMask::WINDOW));

        assert_eq!(result, Err(Error::UnknownWidget { id: 99 }));
        assert_eq!(cache.len(), before);
        assert!(
            cache
                .try_add_child(0, recorder(&log, EventMask::WINDOW))
                .is_ok()
        );
    }

    #[test]
    fn only_widgets_with_the_class_in_their_mask_receive_it() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut cache = cache();
        let wants = cache.add(recorder(&log, EventMask::WINDOW));

        cache.add(recorder(&log, EventMask::POINTER_BUTTONS));
        cache.add(recorder(&log, EventMask::NONE));
        cache.broadcast(&Event::WindowResized { size: size(5, 5) });

        assert_eq!(*log.borrow(), vec![wants]);
    }

    #[test]
    fn hidden_and_disabled_widgets_and_hidden_subtrees_are_skipped() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut cache = cache();
        let hidden = cache.add(recorder(&log, EventMask::WINDOW));
        let under_hidden = cache.add_child(hidden, recorder(&log, EventMask::WINDOW));
        let disabled = cache.add(recorder(&log, EventMask::WINDOW));
        let ok = cache.add(recorder(&log, EventMask::WINDOW));

        cache.get_mut(hidden).unwrap().set_hidden(true);
        cache.get_mut(disabled).unwrap().set_enabled(false);
        cache.broadcast(&focus());

        assert_eq!(*log.borrow(), vec![ok]);
        assert_ne!(under_hidden, ok);
    }

    #[test]
    fn the_run_loop_mask_can_silence_a_class() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut cache = cache();

        cache.add(recorder(&log, EventMask::WINDOW));
        cache.set_loop_mask(EventMask::ALL & !EventMask::WINDOW);
        cache.broadcast(&focus());

        assert!(log.borrow().is_empty());
    }

    #[test]
    fn a_broadcast_queues_an_engine_event_with_the_recipient_count() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut cache = cache();

        cache.add(recorder(&log, EventMask::WINDOW));
        cache.add(recorder(&log, EventMask::WINDOW));
        cache.take_pending_events();
        cache.broadcast(&focus());

        match cache.take_pending_events().as_slice() {
            [Event::Engine(EngineEvent::EventBroadcast { kind, recipients })] => {
                assert_eq!(*kind, crate::event::EventKind::WindowFocusChanged);
                assert_eq!(*recipients, 2);
            }
            other => panic!("unexpected pending events: {other:?}"),
        }
    }
    /// A `Widget` that logs `(name, phase)` for every input event, and can capture or stop.
    struct Probe {
        core: WidgetCore,
        name: &'static str,
        capture: bool,
        stops_in: Option<EventPhase>,
        log: Rc<RefCell<Vec<(&'static str, EventPhase)>>>,
    }

    impl Widget for Probe {
        /// Returns the shared state.
        ///
        /// # Returns
        ///
        /// The `WidgetCore`.
        fn core(&self) -> &WidgetCore {
            &self.core
        }

        /// Returns the shared state for modification.
        ///
        /// # Returns
        ///
        /// The `WidgetCore`.
        fn core_mut(&mut self) -> &mut WidgetCore {
            &mut self.core
        }

        /// Draws nothing.
        ///
        /// # Arguments
        ///
        /// * `_p` - The painter.
        fn paint(&self, _p: &mut dyn crate::paint::Painter) {}

        /// Whether this probe captures.
        ///
        /// # Returns
        ///
        /// The configured flag.
        fn handles_capture(&self) -> bool {
            self.capture
        }

        /// Logs the phase; stops in the configured phase.
        ///
        /// # Arguments
        ///
        /// * `_event` - The event.
        /// * `ctx` - The context.
        ///
        /// # Returns
        ///
        /// [`Propagation::Stop`] in the configured phase, otherwise [`Propagation::Continue`].
        fn handle_event(&mut self, _event: &Event, ctx: &mut EventCtx) -> Propagation {
            self.log.borrow_mut().push((self.name, ctx.phase()));

            if self.stops_in == Some(ctx.phase()) {
                Propagation::Stop
            } else {
                Propagation::Continue
            }
        }
    }

    type PhaseLog = Rc<RefCell<Vec<(&'static str, EventPhase)>>>;

    /// Adds a probe with every event class enabled under `parent`.
    fn probe(
        cache: &mut WidgetCache,
        parent: WidgetId,
        name: &'static str,
        capture: bool,
        stops_in: Option<EventPhase>,
        log: &PhaseLog,
    ) -> WidgetId {
        let mut core = WidgetCore::new(point(0, 0), size(50, 50));

        core.set_event_mask(EventMask::ALL);
        cache.add_child(
            parent,
            Box::new(Probe {
                core,
                name,
                capture,
                stops_in,
                log: log.clone(),
            }),
        )
    }

    fn key_press(id: WidgetId) -> Event {
        Event::KeyPressed {
            id,
            key: crate::input::Key::Return,
            modifiers: crate::input::Modifiers::empty(),
            repeat: false,
        }
    }

    #[test]
    fn delivery_is_capture_then_target_then_bubble() {
        use EventPhase::{Bubble, Capture, Target};

        let log: PhaseLog = Rc::default();
        let mut cache = cache();
        let dialog = probe(&mut cache, 0, "dialog", true, None, &log);
        let group = probe(&mut cache, dialog, "group", false, None, &log);
        let field = probe(&mut cache, group, "field", true, None, &log);

        cache.propagate(field, &key_press(field));

        assert_eq!(
            *log.borrow(),
            vec![
                ("dialog", Capture),
                ("field", Target),
                ("group", Bubble),
                ("dialog", Bubble),
            ]
        );
    }

    #[test]
    fn stop_in_each_phase_ends_delivery() {
        use EventPhase::{Bubble, Capture, Target};

        for (phase, expected) in [
            (Capture, vec![("outer", Capture)]),
            (Target, vec![("outer", Capture), ("field", Target)]),
            (
                Bubble,
                vec![("outer", Capture), ("field", Target), ("inner", Bubble)],
            ),
        ] {
            let log: PhaseLog = Rc::default();
            let mut cache = cache();
            let stopper = |name| {
                if name == "outer" && phase == Capture {
                    Some(Capture)
                } else {
                    None
                }
            };
            let outer = probe(&mut cache, 0, "outer", true, stopper("outer"), &log);
            let inner_stop = if phase == Bubble { Some(Bubble) } else { None };
            let inner = probe(&mut cache, outer, "inner", false, inner_stop, &log);
            let field_stop = if phase == Target { Some(Target) } else { None };
            let field = probe(&mut cache, inner, "field", false, field_stop, &log);

            let mut ctx = EventCtx::default();
            let result = cache.propagate_with(field, &key_press(field), &mut ctx);

            assert_eq!(result, Propagation::Stop);
            assert_eq!(*log.borrow(), expected, "stop in {phase:?}");
        }
    }

    #[test]
    fn a_default_button_sees_the_key_a_text_field_ignored() {
        use EventPhase::{Bubble, Target};

        let log: PhaseLog = Rc::default();
        let mut cache = cache();
        let dialog = probe(&mut cache, 0, "dialog", false, Some(Bubble), &log);
        let field = probe(&mut cache, dialog, "field", false, None, &log);

        let mut ctx = EventCtx::default();
        let result = cache.propagate_with(field, &key_press(field), &mut ctx);

        assert_eq!(result, Propagation::Stop);
        assert_eq!(*log.borrow(), vec![("field", Target), ("dialog", Bubble)]);
    }

    #[test]
    fn disabled_hidden_and_masked_widgets_are_skipped() {
        use EventPhase::Target;

        let log: PhaseLog = Rc::default();
        let mut cache = cache();
        let disabled = probe(&mut cache, 0, "disabled", false, None, &log);
        let hidden = probe(&mut cache, disabled, "hidden", false, None, &log);
        let masked = probe(&mut cache, hidden, "masked", false, None, &log);
        let target = probe(&mut cache, masked, "target", false, None, &log);

        cache.get_mut(disabled).unwrap().set_enabled(false);
        cache.get_mut(hidden).unwrap().set_hidden(true);
        cache.set_event_mask(masked, EventMask::POINTER_BUTTONS);
        cache.propagate(target, &key_press(target));

        assert_eq!(*log.borrow(), vec![("target", Target)]);
    }

    #[test]
    fn semantic_and_hover_events_do_not_propagate() {
        let log: PhaseLog = Rc::default();
        let mut cache = cache();
        let parent = probe(&mut cache, 0, "parent", true, None, &log);
        let child = probe(&mut cache, parent, "child", false, None, &log);

        cache.propagate(child, &Event::PointerEntered { id: child });
        cache.propagate(
            child,
            &Event::Clicked {
                id: child,
                button: crate::input::MouseButton::Left,
                count: 1,
            },
        );

        assert_eq!(
            *log.borrow(),
            vec![("child", EventPhase::Target), ("child", EventPhase::Target)],
            "each event reaches the target once, and never the capturing parent"
        );
    }

    #[test]
    fn a_pointer_press_bubbles_to_parents_through_the_cache() {
        let log: PhaseLog = Rc::default();
        let mut cache = cache();
        let parent = probe(&mut cache, 0, "parent", false, None, &log);
        probe(&mut cache, parent, "child", false, None, &log);

        cache.handle_event(InputEvent::MouseDown {
            position: point(5, 5),
            button: crate::input::MouseButton::Left,
            clicks: 1,
        });

        let names: Vec<_> = log.borrow().iter().map(|(n, _)| *n).collect();

        assert_eq!(names, vec!["child", "parent"]);
    }
}
