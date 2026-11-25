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
//!    ├── ID 1  BaseWidget
//!    │    ├── ID 2  TextWidget      origins are relative to the parent
//!    │    └── ID 3  ButtonWidget
//!    └── ID 4  BoxWidget            later siblings are drawn on top
//! ```
//!
//! A flat list addressed by ID avoids reference-counted parent pointers entirely, and allows a
//! form to be stored and reloaded by ID.

use crate::backend::sdl2::{Sdl2Painter, to_sdl_rect};
use crate::event::{
    CustomEvent, EngineEvent, EngineEventKind, Event, EventCtx, EventMask, EventTarget, MaskScope,
    engine_filter_accepts,
};
use crate::font::FontCache;
use crate::geometry::{Point, make_rect, point};
use crate::paint::{self, CachePolicy};
use crate::widget::{ROOT_WIDGET_ID, Widget, WidgetId};
use sdl2::event::Event as SdlEvent;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::any::Any;
use std::collections::HashMap;

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
    needs_full_redraw: bool,
    cache_policy: CachePolicy,
    pending_events: Vec<Event>,
    loop_mask: EventMask,
    engine_filters: HashMap<WidgetId, Vec<EngineEventKind>>,
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
            needs_full_redraw: true,
            cache_policy: Sdl2Painter::CACHE_POLICY,
            pending_events: Vec::new(),
            loop_mask: EventMask::ALL,
            engine_filters: HashMap::new(),
        }
    }

    /// Returns the cache policy of the backend the cache paints with.
    ///
    /// Defaults to the `SDL2` backend's [`CachePolicy::TextureCache`].  `draw_loop` implements
    /// `TextureCache` only; [`CachePolicy::DirectDraw`] is declared for backends that paint
    /// straight to the screen with clipping, and is not rendered by the `SDL2` path.
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
    /// the parent's existing children.  If the cache is empty, the `Widget` becomes the root `Widget`
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
    pub fn add_child(&mut self, parent: WidgetId, mut widget: Box<dyn Widget>) -> WidgetId {
        let id = self.cache.len() as WidgetId;
        let parent = if self.cache.is_empty() {
            id
        } else {
            assert!(
                (parent as usize) < self.cache.len(),
                "[WidgetCache::add_child] Parent widget ID {} does not exist",
                parent
            );
            parent
        };

        widget.core_mut().id = id;
        widget.core_mut().parent = parent;
        widget.set_invalidated(true);

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

        id
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
    /// The `Widget` as a trait object, or `None` if the ID does not exist.
    pub fn get(&self, id: WidgetId) -> Option<&dyn Widget> {
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
    /// The `Widget` as a mutable trait object, or `None` if the ID does not exist.
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        match self.cache.get_mut(id as usize) {
            Some(widget) => Some(widget.as_mut()),
            None => None,
        }
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

    /// Returns the origin of a `Widget` in window coordinates.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// The `Widget`'s upper left-hand corner in window coordinates: its origin plus the origins of
    /// all of its ancestors.  Returns `(0, 0)` if the ID does not exist.
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
        }

        origin
    }

    /// Returns the IDs of all visible `Widget`s in drawing order: parents before children, and
    /// siblings in the order in which they were added.  Hidden `Widget`s and their children are
    /// skipped.
    fn draw_order(&self) -> Vec<WidgetId> {
        let mut order = Vec::with_capacity(self.cache.len());
        let mut stack = Vec::new();

        if !self.cache.is_empty() {
            stack.push(ROOT_WIDGET_ID);
        }

        while let Some(id) = stack.pop() {
            if self.cache[id as usize].is_hidden() {
                continue;
            }

            order.push(id);

            for child in self.children[id as usize].iter().rev() {
                stack.push(*child);
            }
        }

        order
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
            let origin = self.absolute_origin(id);
            let size = self.cache[id as usize].get_size();

            if x >= origin.x
                && x < origin.x + size.w as i32
                && y >= origin.y
                && y < origin.y + size.h as i32
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

        ctx.into_events()
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

        ctx.into_events()
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

    /// Removes and returns the engine events queued by the cache itself, in order.
    ///
    /// # Returns
    ///
    /// The queued events; empty if there are none.
    pub fn take_pending_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns whether the next [`draw_loop`](WidgetCache::draw_loop) has anything to do.
    ///
    /// # Returns
    ///
    /// `true` if a full redraw is pending or any `Widget` is invalidated.
    pub fn needs_draw(&self) -> bool {
        self.needs_full_redraw || self.cache.iter().any(|w| w.is_invalidated())
    }

    /// Sends an event to a `Widget`.  Disabled `Widget`s, and `Widget`s whose event mask excludes
    /// the event's class, receive nothing.
    fn send(&mut self, id: WidgetId, event: &Event, ctx: &mut EventCtx) {
        let loop_mask = self.loop_mask;

        if let Some(widget) = self.get_mut(id)
            && widget.is_enabled()
            && event.kind().mask_class().is_none_or(|class| {
                loop_mask.intersects(class) && widget.event_mask().intersects(class)
            })
        {
            ctx.set_source(Some(id));
            widget.handle_event(event, ctx);
        }
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
            return ctx.into_events();
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

        ctx.into_events()
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
    /// - Other events are ignored.
    /// - Disabled `Widget`s receive none of these events, but still block the `Widget`s beneath them.
    ///
    /// # Arguments
    ///
    /// * `event` - The raw `SDL2` event.
    ///
    /// # Returns
    ///
    /// The `Event`s the `Widget`s generated while handling the event, in the order emitted.
    pub fn handle_event(&mut self, event: SdlEvent) -> Vec<Event> {
        let mut ctx = EventCtx::default();

        if self.cache.is_empty() {
            return ctx.into_events();
        }

        match event {
            SdlEvent::MouseButtonDown {
                x,
                y,
                mouse_btn,
                clicks,
                ..
            } => {
                self.current_widget_id = self.widget_at(x, y);

                let target = self.current_widget_id;

                self.captured_widget_id = Some(target);
                self.send(
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

            SdlEvent::MouseButtonUp {
                x,
                y,
                mouse_btn,
                clicks,
                ..
            } => {
                self.current_widget_id = self.widget_at(x, y);

                let target = self
                    .captured_widget_id
                    .take()
                    .unwrap_or(self.current_widget_id);

                self.send(
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

            SdlEvent::MouseMotion { x, y, .. } => {
                let previous_widget_id = self.current_widget_id;

                self.current_widget_id = self.widget_at(x, y);

                if self.current_widget_id != previous_widget_id {
                    self.send(
                        previous_widget_id,
                        &Event::PointerExited {
                            id: previous_widget_id,
                        },
                        &mut ctx,
                    );
                    self.send(
                        self.current_widget_id,
                        &Event::PointerEntered {
                            id: self.current_widget_id,
                        },
                        &mut ctx,
                    );
                }

                let target = self.captured_widget_id.unwrap_or(self.current_widget_id);

                self.send(
                    target,
                    &Event::PointerMoved {
                        id: target,
                        position: point(x, y),
                    },
                    &mut ctx,
                );
            }

            _ => {}
        }

        ctx.into_events()
    }

    /// Paints a single `Widget` into its texture.
    fn paint(&mut self, id: WidgetId, c: &mut Canvas<Window>, fc: &mut FontCache) {
        let widget = &mut self.cache[id as usize];
        let size = widget.get_size();

        if size.w == 0 || size.h == 0 {
            return;
        }

        widget.core_mut().texture.create_or_resize_texture(c, size);

        let mut texture = widget.core_mut().texture.take().unwrap();
        let painter: &dyn Widget = widget.as_ref();

        c.with_texture_canvas(&mut texture, |texture_canvas| {
            let mut sdl_painter = Sdl2Painter::new(texture_canvas, fc);

            painter.paint(&mut sdl_painter);
            paint::state_overlay(&mut sdl_painter, size, painter.state());
        })
        .unwrap();

        widget.core_mut().texture.restore(texture);
    }

    /// Repaints changed `Widget`s and redraws the window if anything changed.
    ///
    /// Every invalidated, visible `Widget` is repainted into its texture.  If any `Widget` changed (or
    /// a full redraw was requested), the window is cleared and every visible `Widget`'s texture is
    /// copied to the screen in drawing order.  Copying cached textures is cheap; repainting only
    /// happens for `Widget`s that changed.
    ///
    /// # Arguments
    ///
    /// * `c` - The window's canvas.
    /// * `fc` - The font cache, passed to each `Widget`'s `paint`.
    ///
    /// # Returns
    ///
    /// `true` if the window was redrawn and the canvas needs to be presented, `false` if nothing
    /// changed.
    ///
    /// # Panics
    ///
    /// Panics if `SDL2` fails to render into a `Widget`'s texture or copy it to the window.
    pub fn draw_loop(&mut self, c: &mut Canvas<Window>, fc: &mut FontCache) -> bool {
        let order = self.draw_order();
        let mut visible = vec![false; self.cache.len()];
        let mut redraw = self.needs_full_redraw;

        for id in &order {
            visible[*id as usize] = true;
        }

        for (id, is_visible) in visible.into_iter().enumerate() {
            if self.cache[id].is_invalidated() {
                if is_visible {
                    self.paint(id as WidgetId, c, fc);
                }

                self.cache[id].set_invalidated(false);
                redraw = true;
            }
        }

        if !redraw {
            return false;
        }

        c.set_draw_color(Color::RGBA(255, 255, 255, 255));
        c.clear();

        for id in order {
            let origin = self.absolute_origin(id);
            let widget = &mut self.cache[id as usize];
            let size = widget.get_size();

            if let Some(texture) = widget.core_mut().texture.get_optional_ref() {
                c.copy(texture, None, to_sdl_rect(make_rect(origin, size)))
                    .unwrap();
            }
        }

        self.needs_full_redraw = false;

        true
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
        fn handle_event(&mut self, _event: &Event, _ctx: &mut EventCtx) {
            self.log.borrow_mut().push(self.get_id());
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
}
