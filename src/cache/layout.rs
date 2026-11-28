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

//! The layout pass: measures and arranges the containers that have a layout manager.
//!
//! [`WidgetCache::draw_loop`] runs it once per frame, before painting.  Only `Widget`s that
//! flagged themselves (a changed size, visibility, size hint, text, or font, a new or removed
//! child, or a new layout) start it.
//!
//! Before it, [`WidgetCache::run_anchors`] moves and resizes the children of resized containers
//! that have no layout, according to each child's [`Anchors`](crate::layout::Anchors).

use super::WidgetCache;
use crate::backend::Backend;
use crate::event::{EngineEvent, Event};
use crate::geometry::{Rect, Size};
use crate::layout::{ChildInfo, SizeHint};
use crate::paint::Painter;
use crate::widget::{ROOT_WIDGET_ID, WidgetId};
use std::collections::BTreeSet;

impl WidgetCache {
    /// Marks a `Widget` as needing layout.  Equivalent to calling
    /// [`Widget::request_layout`](crate::widget::Widget::request_layout).
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` whose size hint or place in its container changed; an unknown ID is
    ///   ignored.
    pub fn invalidate_layout(&mut self, id: WidgetId) {
        if let Some(widget) = self.get_mut(id) {
            widget.request_layout();
        }
    }

    /// Returns the size hint a container's layout last measured.
    ///
    /// # Arguments
    ///
    /// * `id` - The container.
    ///
    /// # Returns
    ///
    /// The hint, or `None` if the container has no layout or has not been measured yet.  The
    /// window can use the root's minimum size as its minimum size.
    pub fn measured_hint(&self, id: WidgetId) -> Option<SizeHint> {
        self.layout_hints.get(&id).copied()
    }

    /// Applies [`Anchors`](crate::layout::Anchors): for each container without a layout whose
    /// size changed since its children were last anchored, moves and resizes the children that
    /// have anchors by the amount the container grew or shrank.  Containers are handled parents
    /// first, so a resized child's own anchored children follow in the same pass.  A moved child
    /// keeps its cached texture; a resized one is repainted.  Emits
    /// [`EngineEvent::AnchorsApplied`] once for each container that changed a child.
    ///
    /// The first time a container is seen with anchored children, its current size is remembered
    /// and nothing moves.
    pub(super) fn run_anchors(&mut self) {
        let mut containers = BTreeSet::new();

        for id in 1..self.cache.len() as WidgetId {
            if !self.cache[id as usize].anchors().is_empty() {
                let parent = self.cache[id as usize].get_parent();

                if !self.cache[parent as usize].has_layout() {
                    containers.insert((self.depth(parent), parent));
                }
            }
        }

        // A container that lost its anchored children starts over if it gets new ones.
        for id in 0..self.cache.len() as WidgetId {
            if self.cache[id as usize].core().anchor_basis().is_some()
                && !containers.iter().any(|(_, c)| *c == id)
            {
                self.cache[id as usize].core_mut().set_anchor_basis(None);
            }
        }

        for (_, container) in containers {
            self.anchor_children(container);
        }
    }

    /// Applies the anchors of one container's children.
    ///
    /// # Arguments
    ///
    /// * `container` - A container without a layout that has anchored children.
    fn anchor_children(&mut self, container: WidgetId) {
        let now = self.cache[container as usize].get_size();
        let basis = self.cache[container as usize].core().anchor_basis();

        self.cache[container as usize]
            .core_mut()
            .set_anchor_basis(Some(now));

        let Some(before) = basis else {
            return;
        };

        if before == now {
            return;
        }

        let dw = i64::from(now.w) - i64::from(before.w);
        let dh = i64::from(now.h) - i64::from(before.h);
        let mut changed = 0;

        for child in self.children[container as usize].clone() {
            let widget = &mut self.cache[child as usize];
            let anchors = widget.anchors();

            if anchors.is_empty() {
                continue;
            }

            let origin = widget.get_origin();
            let size = widget.get_size();
            let rect = anchors.apply(Rect::new(origin.x, origin.y, size.w, size.h), dw, dh);
            let (moved, resized) = widget.core_mut().place(rect);

            if resized && widget.has_layout() {
                widget.request_layout();
            }

            if moved || resized {
                changed += 1;
                self.needs_full_redraw = true;
            }
        }

        if changed > 0 {
            self.queue_event(Event::Engine(EngineEvent::AnchorsApplied {
                container,
                children: changed,
            }));
        }
    }

    /// Runs the layout pass for everything that needs it, then clears the flags.
    ///
    /// Nothing happens if no `Widget` asked for layout.
    ///
    /// # Arguments
    ///
    /// * `backend` - Supplies the painter that measures text.
    pub(super) fn run_layout(&mut self, backend: &mut dyn Backend) {
        let seeds: Vec<WidgetId> = (0..self.cache.len() as WidgetId)
            .filter(|id| self.cache[*id as usize].core().layout_dirty())
            .collect();

        if seeds.is_empty() {
            return;
        }

        backend.measure(&mut |p| self.layout_pass(&seeds, p));

        for widget in &mut self.cache {
            widget.core_mut().set_layout_dirty(false);
        }
    }

    /// Measures bottom-up from the flagged `Widget`s, stopping at containers whose size hint did
    /// not change, then arranges the containers top-down.
    ///
    /// # Arguments
    ///
    /// * `seeds` - The `Widget`s that flagged themselves.
    /// * `p` - The measuring painter.
    fn layout_pass(&mut self, seeds: &[WidgetId], p: &mut dyn Painter) {
        let mut arrange = BTreeSet::new();

        for &seed in seeds {
            if self.cache[seed as usize].has_layout() {
                self.measure_upward(seed, p, &mut arrange);
            }

            let parent = self.cache[seed as usize].get_parent();

            if seed != ROOT_WIDGET_ID && self.cache[parent as usize].has_layout() {
                self.measure_upward(parent, p, &mut arrange);
            }
        }

        while let Some((_, container)) = arrange.pop_first() {
            self.arrange_container(container, p, &mut arrange);
        }
    }

    /// Queues a container for arranging, then does the same for its parent while the container's
    /// size hint keeps changing and the parent has a layout.
    ///
    /// # Arguments
    ///
    /// * `start` - The container with a layout.
    /// * `p` - The measuring painter.
    /// * `arrange` - The containers to arrange, ordered by depth and ID.
    fn measure_upward(
        &mut self,
        start: WidgetId,
        p: &mut dyn Painter,
        arrange: &mut BTreeSet<(usize, WidgetId)>,
    ) {
        let mut container = start;

        loop {
            arrange.insert((self.depth(container), container));

            let before = self.layout_hints.get(&container).copied();
            let after = self.measure_container(container, p);
            let parent = self.cache[container as usize].get_parent();

            if before == Some(after)
                || container == ROOT_WIDGET_ID
                || !self.cache[parent as usize].has_layout()
            {
                return;
            }

            container = parent;
        }
    }

    /// Measures a container with a layout, and remembers the hint.
    ///
    /// # Arguments
    ///
    /// * `container` - A container with a layout.
    /// * `p` - The measuring painter.
    ///
    /// # Returns
    ///
    /// The container's size hint; the default hint if it has no layout.
    fn measure_container(&mut self, container: WidgetId, p: &mut dyn Painter) -> SizeHint {
        let infos = self.child_infos(container, p);
        let size = self.layout_area(container);
        let Some(layout) = self.cache[container as usize].core().layout_ref() else {
            return SizeHint::default();
        };
        let hint = layout.measure(&infos, size);

        self.layout_hints.insert(container, hint);

        hint
    }

    /// Arranges one container's children and applies the rectangles.  A child that only moved is
    /// not invalidated; the frame copies its cached texture to the new place.  A child that was
    /// resized is repainted, and if it is a container, arranged in turn.
    ///
    /// # Arguments
    ///
    /// * `container` - A container with a layout.
    /// * `p` - The measuring painter.
    /// * `arrange` - The containers still to arrange; resized containers are added.
    fn arrange_container(
        &mut self,
        container: WidgetId,
        p: &mut dyn Painter,
        arrange: &mut BTreeSet<(usize, WidgetId)>,
    ) {
        let infos = self.child_infos(container, p);
        let Size { w, h } = self.layout_area(container);
        let Some(layout) = self.cache[container as usize].core().layout_ref() else {
            return;
        };
        let rects = layout.arrange(&infos, Rect::new(0, 0, w, h));

        for (info, rect) in infos.iter().zip(rects) {
            let (moved, resized) = self.cache[info.id as usize].core_mut().place(rect);

            if moved || resized {
                self.needs_full_redraw = true;
            }

            if resized && self.cache[info.id as usize].has_layout() {
                arrange.insert((self.depth(info.id), info.id));
            }
        }

        self.queue_event(Event::Engine(EngineEvent::LayoutChanged { container }));
    }

    /// Returns the size of the area a container's layout places its children on: its child
    /// area if it has one (a scrolling viewport's content), otherwise its own size.
    ///
    /// # Arguments
    ///
    /// * `container` - The container.
    ///
    /// # Returns
    ///
    /// The size of the area.
    fn layout_area(&self, container: WidgetId) -> Size {
        let widget = &self.cache[container as usize];

        widget.child_area().unwrap_or(widget.get_size())
    }

    /// Collects what a container's layout needs to know about its visible children.  A child
    /// that is itself a container with a layout reports what its layout measures.
    ///
    /// # Arguments
    ///
    /// * `container` - The container.
    /// * `p` - The measuring painter.
    ///
    /// # Returns
    ///
    /// One [`ChildInfo`] per visible child, in order.
    fn child_infos(&mut self, container: WidgetId, p: &mut dyn Painter) -> Vec<ChildInfo> {
        let children = self.children[container as usize].clone();
        let mut infos = Vec::with_capacity(children.len());

        for id in children {
            if self.cache[id as usize].is_hidden() {
                continue;
            }

            let hint = if self.cache[id as usize].has_layout() {
                self.measure_container(id, p)
            } else {
                self.cache[id as usize].size_hint(p)
            };

            infos.push(ChildInfo {
                id,
                hint,
                params: self.cache[id as usize].layout_params(),
            });
        }

        infos
    }

    /// Counts the ancestors of a `Widget`, so parents are arranged before their children.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`.
    ///
    /// # Returns
    ///
    /// The distance from the root; `0` for the root.
    fn depth(&self, id: WidgetId) -> usize {
        let mut depth = 0;
        let mut current = id;

        while current != ROOT_WIDGET_ID {
            current = self.cache[current as usize].get_parent();
            depth += 1;
        }

        depth
    }
}
