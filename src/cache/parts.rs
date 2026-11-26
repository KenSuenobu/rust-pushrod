// Composite parts
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

//! The `WidgetCache` side of composite `Widget`s: building a composite's parts when it is
//! added, adding, inserting, and removing parts later, laying them out, and handing the events
//! parts emit to their composite.  See the [`composite`](crate::composite) module.

use super::WidgetCache;
use crate::composite::{Composite, PartName, PartsBuilder};
use crate::error::{Error, Result};
use crate::event::{EngineEvent, Event, EventCtx};
use crate::geometry::{Point, Size, point};
use crate::paint::Painter;
use crate::widget::{ROOT_WIDGET_ID, Widget, WidgetCore, WidgetId};
use std::collections::VecDeque;

/// Fills a slot in the cache that holds no `Widget`: a composite's slot while it is being built,
/// and the slot of a removed `Widget` (IDs are indexes and are never reused).  It is hidden,
/// handles no events, and draws nothing.
#[derive(Debug)]
pub(super) struct Vacant {
    core: WidgetCore,
}

impl Vacant {
    /// Creates an empty slot.
    ///
    /// # Arguments
    ///
    /// * `id` - The slot's ID.
    /// * `parent` - The parent recorded for the slot.
    ///
    /// # Returns
    ///
    /// A hidden, zero-sized placeholder.
    pub(super) fn new(id: WidgetId, parent: WidgetId) -> Self {
        let mut core = WidgetCore::new(Point::new(0, 0), Size::new(0, 0));

        core.id = id;
        core.parent = parent;

        let mut vacant = Self { core };

        vacant.set_hidden(true);
        vacant.set_invalidated(false);
        vacant
    }
}

impl Widget for Vacant {
    /// Returns the placeholder's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the placeholder's shared state for modification.
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
    fn paint(&self, _p: &mut dyn Painter) {}
}

impl WidgetCache {
    /// Builds the parts of a `Widget` just added, if it is a composite, then lays them out.
    ///
    /// The composite is moved out of its slot while [`Composite::build`] runs, so the builder can
    /// add its parts to the cache; each part gets the next free ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget` just added.
    pub(super) fn build_composite(&mut self, id: WidgetId) {
        if self.get(id).and_then(|w| w.as_composite()).is_none() {
            return;
        }

        let parent = self.cache[id as usize].get_parent();
        let mut widget = std::mem::replace(
            &mut self.cache[id as usize],
            Box::new(Vacant::new(id, parent)),
        );
        let mut builder = PartsBuilder::new(self, id);

        if let Some(composite) = widget.as_composite_mut() {
            composite.build(&mut builder);
        }

        let parts = builder.finish();

        if let Some(composite) = widget.as_composite_mut() {
            *composite.part_list_mut() = parts;
        }

        self.cache[id as usize] = widget;

        // The composite exists, so arranging cannot fail.
        let _ = self.arrange_parts(id);
    }

    /// Returns a `Widget` as a composite.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the composite.
    ///
    /// # Returns
    ///
    /// The composite.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `id` does not exist, [`Error::Composite`] if it is not a
    /// composite.
    fn composite(&self, id: WidgetId) -> Result<&dyn Composite> {
        self.get(id)
            .ok_or(Error::UnknownWidget { id })?
            .as_composite()
            .ok_or_else(|| Error::composite(id, "is not a composite widget"))
    }

    /// Returns a `Widget` as a mutable composite.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the composite.
    ///
    /// # Returns
    ///
    /// The composite.
    ///
    /// # Errors
    ///
    /// The same as [`WidgetCache::composite`].
    fn composite_mut(&mut self, id: WidgetId) -> Result<&mut dyn Composite> {
        self.get_mut(id)
            .ok_or(Error::UnknownWidget { id })?
            .as_composite_mut()
            .ok_or_else(|| Error::composite(id, "is not a composite widget"))
    }

    /// Finds a composite's part by name.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    /// * `name` - The part's name, such as `"title"` or `"button:0"`.
    ///
    /// # Returns
    ///
    /// The part's ID, or `None` if `composite` is not a composite or has no such part.
    pub fn part(&self, composite: WidgetId, name: &str) -> Option<WidgetId> {
        self.composite(composite).ok()?.part(name)
    }

    /// Lists a composite's parts by name.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    ///
    /// # Returns
    ///
    /// Each part's name and ID, in order; empty if `composite` is not a composite.
    pub fn parts(&self, composite: WidgetId) -> Vec<(PartName, WidgetId)> {
        self.composite(composite)
            .map(|c| c.parts())
            .unwrap_or_default()
    }

    /// Adds a part after a composite's existing parts, then lays the parts out again.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    /// * `name` - The new part's name.
    /// * `widget` - The new part.
    ///
    /// # Returns
    ///
    /// The new part's ID.
    ///
    /// # Errors
    ///
    /// The same as [`WidgetCache::insert_part`].
    pub fn add_part(
        &mut self,
        composite: WidgetId,
        name: impl Into<PartName>,
        widget: Box<dyn Widget>,
    ) -> Result<WidgetId> {
        let end = self.composite(composite)?.part_list().len();

        self.insert_part(composite, end, name, widget)
    }

    /// Inserts a part into a composite at a position in its list of parts, then lays the parts
    /// out again.  The part is added to the window as the composite's last child, so it gets
    /// the next free ID and emits `WidgetAdded`.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    /// * `index` - The position in the composite's list of parts; past the end means last.
    /// * `name` - The new part's name.
    /// * `widget` - The new part.
    ///
    /// # Returns
    ///
    /// The new part's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `composite` does not exist, [`Error::Composite`] if it is not
    /// a composite or already has a part named `name`.
    pub fn insert_part(
        &mut self,
        composite: WidgetId,
        index: usize,
        name: impl Into<PartName>,
        widget: Box<dyn Widget>,
    ) -> Result<WidgetId> {
        let name = name.into();

        if self.composite(composite)?.part(&name).is_some() {
            return Err(Error::composite(
                composite,
                format!("already has a part named \"{name}\""),
            ));
        }

        let id = self.try_add_child(composite, widget)?;

        self.composite_mut(composite)?
            .part_list_mut()
            .insert(index, name, id);
        self.arrange_parts(composite)?;

        Ok(id)
    }

    /// Removes a part, and every `Widget` inside it, from a composite and the window, then lays
    /// the remaining parts out again.  Each removed `Widget` emits `WidgetRemoved`, and its ID is
    /// never reused: `get` returns `None` for it, and callbacks registered on it never run again.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    /// * `name` - The part's name.
    ///
    /// # Returns
    ///
    /// The removed part's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `composite` does not exist, [`Error::Composite`] if it is not
    /// a composite or has no part named `name`.
    pub fn remove_part(&mut self, composite: WidgetId, name: &str) -> Result<WidgetId> {
        let id = self
            .composite_mut(composite)?
            .part_list_mut()
            .remove(name)
            .ok_or_else(|| Error::composite(composite, format!("has no part named \"{name}\"")))?;

        self.remove_subtree(id);
        self.arrange_parts(composite)?;

        Ok(id)
    }

    /// Inserts an item into one of a composite's lists of parts (its buttons, menu items, or
    /// tabs, named `"<kind>:<n>"`), renumbers the list, and lays the parts out again.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    /// * `kind` - The list's name before the colon, such as `"button"`.
    /// * `index` - The item's position in the list; past the end appends it.
    /// * `widget` - The new item.
    ///
    /// # Returns
    ///
    /// The new item's ID; it is named `"<kind>:<index>"` (clamped to the list's length).
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `composite` does not exist, [`Error::Composite`] if it is not
    /// a composite.
    pub fn insert_list_part(
        &mut self,
        composite: WidgetId,
        kind: &str,
        index: usize,
        widget: Box<dyn Widget>,
    ) -> Result<WidgetId> {
        let position = self
            .composite(composite)?
            .part_list()
            .list_position(kind, index);

        // "<kind>:" belongs to the list, and renumbering gives it its index.
        let id = self.insert_part(composite, position, format!("{kind}:"), widget)?;

        self.composite_mut(composite)?
            .part_list_mut()
            .renumber(kind);

        Ok(id)
    }

    /// Removes an item from one of a composite's lists of parts, renumbers the list, and lays
    /// the parts out again.  See [`WidgetCache::remove_part`].
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    /// * `kind` - The list's name before the colon, such as `"button"`.
    /// * `index` - The item's position in the list.
    ///
    /// # Returns
    ///
    /// The removed item's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `composite` does not exist, [`Error::Composite`] if it is not
    /// a composite or the list has no item at `index`.
    pub fn remove_list_part(
        &mut self,
        composite: WidgetId,
        kind: &str,
        index: usize,
    ) -> Result<WidgetId> {
        let id = self.remove_part(composite, &format!("{kind}:{index}"))?;

        self.composite_mut(composite)?
            .part_list_mut()
            .renumber(kind);

        Ok(id)
    }

    /// Lays a composite's parts out again with [`Composite::layout_parts`], moving and resizing
    /// the parts whose rectangle changed, and emits `PartsArranged`.  The cache calls it after
    /// building and after each part change; call it after resizing a composite.
    ///
    /// # Arguments
    ///
    /// * `composite` - The ID of the composite.
    ///
    /// # Returns
    ///
    /// How many parts the layout placed.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `composite` does not exist, [`Error::Composite`] if it is not
    /// a composite.
    pub fn arrange_parts(&mut self, composite: WidgetId) -> Result<usize> {
        let owner = self.composite(composite)?;
        let placements: Vec<_> = owner
            .layout_parts()
            .into_iter()
            .filter(|(id, _)| owner.part_list().contains(*id))
            .collect();
        let placed = placements.len();

        for (id, rect) in placements {
            if let Some(part) = self.get_mut(id) {
                let origin = point(rect.x, rect.y);
                let size = Size::new(rect.w, rect.h);

                if part.get_origin() != origin {
                    part.set_origin(origin);
                }

                if part.get_size() != size {
                    part.set_size(size);
                }
            }
        }

        self.needs_full_redraw = true;
        self.queue_event(Event::Engine(EngineEvent::PartsArranged {
            id: composite,
            placed,
        }));

        Ok(placed)
    }

    /// Returns the composite that owns a part.
    ///
    /// # Arguments
    ///
    /// * `part` - The ID of a `Widget`.
    ///
    /// # Returns
    ///
    /// The ID of its parent, if the parent is a composite that lists it as a part.
    fn owner_of(&self, part: WidgetId) -> Option<WidgetId> {
        let parent = self.get(part)?.get_parent();

        (parent != part
            && self
                .composite(parent)
                .is_ok_and(|c| c.part_list().contains(part)))
        .then_some(parent)
    }

    /// Hands an event a part emitted to the composite that owns it, and the events the composite
    /// emits in response to its own owner, and so on up the tree.
    ///
    /// A composite's translated events are offered only to composites above it, so translation
    /// ends after at most one step per level of the tree: an event a composite emits for one of
    /// its own parts is delivered but not handed back to it.
    ///
    /// # Arguments
    ///
    /// * `event` - An event a `Widget` generated.
    ///
    /// # Returns
    ///
    /// The events the composites generated, in order; empty if `event` is not from a part.
    pub(crate) fn translate_part_event(&mut self, event: &Event) -> Vec<Event> {
        let mut out = Vec::new();
        let mut queue: VecDeque<(Event, Option<WidgetId>)> = VecDeque::new();

        queue.push_back((event.clone(), None));

        while let Some((event, translated_by)) = queue.pop_front() {
            let Some(owner) = event.widget_id().and_then(|id| self.owner_of(id)) else {
                continue;
            };

            if translated_by.is_some_and(|below| !self.ancestors(below).contains(&owner)) {
                continue;
            }

            let mut ctx = EventCtx::default();
            let part = event.widget_id().unwrap_or(ROOT_WIDGET_ID);

            ctx.set_source(Some(owner));
            ctx.set_origin(self.absolute_origin(owner));

            if let Ok(composite) = self.composite_mut(owner) {
                composite.handle_part_event(part, &event, &mut ctx);
            }

            for generated in self.finish(ctx) {
                out.push(generated.clone());
                queue.push_back((generated, Some(owner)));
            }
        }

        out
    }

    /// Removes a `Widget` and its descendants from the tree, leaving empty slots behind.
    ///
    /// Focus, mouse capture, hover, animation requests, and engine filters that name a removed
    /// `Widget` are cleared; a focus loss emits `FocusChanged`.  Each removed `Widget` emits
    /// `WidgetRemoved`, parents first.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget` to remove; never the root.
    fn remove_subtree(&mut self, id: WidgetId) {
        if id == ROOT_WIDGET_ID || self.get(id).is_none() {
            return;
        }

        let parent = self.cache[id as usize].get_parent();

        self.children[parent as usize].retain(|&child| child != id);

        let mut stack = vec![id];
        let mut removed = Vec::new();

        while let Some(current) = stack.pop() {
            removed.push(current);
            stack.extend(self.children[current as usize].drain(..).rev());
        }

        for current in removed {
            let parent = self.cache[current as usize].get_parent();

            self.cache[current as usize] = Box::new(Vacant::new(current, parent));
            self.removed.insert(current);
            self.animating.retain(|&animating| animating != current);
            self.engine_filters.remove(&current);

            if self.captured_widget_id == Some(current) {
                self.captured_widget_id = None;
            }

            if self.current_widget_id == current {
                self.current_widget_id = ROOT_WIDGET_ID;
            }

            if self.focused_widget_id == Some(current) {
                self.focused_widget_id = None;
                self.queue_event(Event::Engine(EngineEvent::FocusChanged {
                    old: Some(current),
                    new: None,
                }));
            }

            self.queue_event(Event::Engine(EngineEvent::WidgetRemoved {
                id: current,
                parent,
            }));
        }

        self.needs_full_redraw = true;
    }
}
