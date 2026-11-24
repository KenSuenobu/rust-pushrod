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
//!   ID 0  BaseWidget (window)
//!    ├── ID 1  BaseWidget
//!    │    ├── ID 2  TextWidget      origins are relative to the parent
//!    │    └── ID 3  ButtonWidget
//!    └── ID 4  BoxWidget            later siblings are drawn on top
//! ```
//!
//! A flat list addressed by ID avoids reference-counted parent pointers entirely, and allows a
//! form to be stored and reloaded by ID.

use crate::event::{EventCtx, PushrodEvent};
use crate::font::FontCache;
use crate::geometry::{make_rect, Point};
use crate::widget::{Widget, WidgetId, ROOT_WIDGET_ID};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::any::Any;

/// Contains the tree of `Widget`s in a window.  `current_widget_id` is the `Widget` under the
/// mouse pointer, and `captured_widget_id` is the `Widget` that received a mouse button press,
/// which continues to receive mouse events until the button is released.
pub struct WidgetCache {
    cache: Vec<Box<dyn Widget>>,
    children: Vec<Vec<WidgetId>>,
    current_widget_id: WidgetId,
    captured_widget_id: Option<WidgetId>,
    needs_full_redraw: bool,
}

impl WidgetCache {
    /// Creates a new, empty `WidgetCache`.  The first `Widget` added becomes the top-level
    /// `Widget` (ID `0`).
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            children: Vec::new(),
            current_widget_id: ROOT_WIDGET_ID,
            captured_widget_id: None,
            needs_full_redraw: true,
        }
    }

    /// Adds a `Widget` as a child of the top-level `Widget`, returning its ID.  If the cache is
    /// empty, the `Widget` becomes the top-level `Widget`.
    pub fn add(&mut self, widget: Box<dyn Widget>) -> WidgetId {
        self.add_child(ROOT_WIDGET_ID, widget)
    }

    /// Adds a `Widget` as a child of `parent`, returning its ID.  The origin of the `Widget` is
    /// relative to the origin of its parent.
    ///
    /// Panics if `parent` does not exist.
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

        id
    }

    /// Returns the number of `Widget`s in the cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if the cache contains no `Widget`s.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Retrieves a `Widget` by ID, `None` if not found.
    pub fn get(&self, id: WidgetId) -> Option<&dyn Widget> {
        self.cache.get(id as usize).map(|widget| widget.as_ref())
    }

    /// Retrieves a mutable `Widget` by ID, `None` if not found.
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        match self.cache.get_mut(id as usize) {
            Some(widget) => Some(widget.as_mut()),
            None => None,
        }
    }

    /// Retrieves a `Widget` by ID as its concrete type, `None` if the ID does not exist or is a
    /// different type.
    ///
    /// ```rust,ignore
    /// let text = cache.get_as::<TextWidget>(text_id).unwrap();
    /// ```
    pub fn get_as<T: Widget>(&self, id: WidgetId) -> Option<&T> {
        let widget: &dyn Any = self.cache.get(id as usize)?.as_ref();

        widget.downcast_ref::<T>()
    }

    /// Retrieves a mutable `Widget` by ID as its concrete type, `None` if the ID does not exist or
    /// is a different type.
    pub fn get_as_mut<T: Widget>(&mut self, id: WidgetId) -> Option<&mut T> {
        let widget: &mut dyn Any = self.cache.get_mut(id as usize)?.as_mut();

        widget.downcast_mut::<T>()
    }

    /// Returns the IDs of the children of a `Widget`, in drawing order.
    pub fn children_of(&self, id: WidgetId) -> &[WidgetId] {
        self.children
            .get(id as usize)
            .map(|children| children.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the ID of the `Widget` currently under the mouse pointer.
    pub fn get_current_widget(&self) -> WidgetId {
        self.current_widget_id
    }

    /// Forces every visible `Widget` to be copied to the screen on the next frame, such as
    /// after the window has been exposed.
    pub fn invalidate_all(&mut self) {
        self.needs_full_redraw = true;
    }

    /// Returns the origin of a `Widget` in window coordinates, by adding the origins of all of its
    /// parents.
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

    /// Returns the top-most visible `Widget` ID at the given window coordinates.  Returns `0`
    /// (the top-level `Widget`) if no other `Widget` was found.
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

    /// Sends an event to a `Widget`.
    fn send(&mut self, id: WidgetId, event: &PushrodEvent, ctx: &mut EventCtx) {
        if let Some(widget) = self.get_mut(id) {
            widget.handle_event(event, ctx);
        }
    }

    /// This handles the direct events from the `Engine`.  Raw events from the `SDL2` event pump
    /// are routed to the appropriate `Widget`, and any `PushrodEvent`s that the `Widget`s
    /// generate are returned, in order, for the `Engine` to deliver to the application.
    ///
    /// - Mouse button presses go to the top-most `Widget` under the pointer, which then
    ///   captures the mouse until the button is released.
    /// - Mouse motion generates a `BoundsChange` to both the `Widget` that was left and the
    ///   `Widget` that was entered, then goes to the captured `Widget`, or the `Widget` under
    ///   the pointer.
    /// - Mouse button releases go to the captured `Widget`, then release the capture.
    pub fn handle_event(&mut self, event: Event) -> Vec<PushrodEvent> {
        let mut ctx = EventCtx::default();

        if self.cache.is_empty() {
            return ctx.into_events();
        }

        match event {
            Event::MouseButtonDown { x, y, .. } => {
                self.current_widget_id = self.widget_at(x, y);

                let target = self.current_widget_id;

                self.captured_widget_id = Some(target);
                self.send(target, &PushrodEvent::SystemEvent(target, event), &mut ctx);
            }

            Event::MouseButtonUp { x, y, .. } => {
                self.current_widget_id = self.widget_at(x, y);

                let target = self
                    .captured_widget_id
                    .take()
                    .unwrap_or(self.current_widget_id);

                self.send(target, &PushrodEvent::SystemEvent(target, event), &mut ctx);
            }

            Event::MouseMotion { x, y, .. } => {
                let previous_widget_id = self.current_widget_id;

                self.current_widget_id = self.widget_at(x, y);

                if self.current_widget_id != previous_widget_id {
                    let bounds_event =
                        PushrodEvent::BoundsChange(previous_widget_id, self.current_widget_id);

                    self.send(previous_widget_id, &bounds_event, &mut ctx);
                    self.send(self.current_widget_id, &bounds_event, &mut ctx);
                }

                let target = self.captured_widget_id.unwrap_or(self.current_widget_id);

                self.send(target, &PushrodEvent::SystemEvent(target, event), &mut ctx);
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
            painter.paint(texture_canvas, fc);
        })
        .unwrap();

        widget.core_mut().texture.restore(texture);
    }

    /// This is the main draw loop for all of the `Widget`s in the cache.  If any `Widget` is
    /// invalidated, it is repainted into its texture, and then every visible `Widget`'s texture is
    /// copied to the screen in drawing order.  Copying cached textures is cheap; repainting is
    /// only done for `Widget`s that changed.
    ///
    /// Returns `true` if the screen was redrawn and needs to be presented, `false` if nothing
    /// changed.
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
                c.copy(texture, None, make_rect(origin, size)).unwrap();
            }
        }

        self.needs_full_redraw = false;

        true
    }
}

impl Default for WidgetCache {
    fn default() -> Self {
        Self::new()
    }
}
