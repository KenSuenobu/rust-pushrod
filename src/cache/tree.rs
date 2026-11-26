// Widget tree editing
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

//! Editing the widget tree while the application runs: removing a `Widget` and its
//! descendants, moving a `Widget` to a new parent, and changing its place in its parent's
//! stacking order.
//!
//! IDs are indexes into the cache and never change.  A removed `Widget` leaves an empty slot
//! (a tombstone) behind, so every other ID still names the same `Widget`, and the removed ID is
//! never given out again.

use super::WidgetCache;
use crate::error::{Error, Result};
use crate::event::{EngineEvent, Event};
use crate::widget::{ROOT_WIDGET_ID, WidgetId};

impl WidgetCache {
    /// Removes a `Widget` and all of its descendants from the window.
    ///
    /// Each removed slot becomes a tombstone: [`get`](WidgetCache::get) returns `None` for it,
    /// its ID is never reused, and every other ID is unchanged.  Keyboard focus, hover, and mouse
    /// capture that point into the removed subtree are cleared (a focus loss emits
    /// [`EngineEvent::FocusChanged`]), and the removed `Widget`s' textures are destroyed by the
    /// backend on the next [`draw_loop`](WidgetCache::draw_loop).  Each removed `Widget` emits
    /// [`EngineEvent::WidgetRemoved`] followed by [`Event::Removed`], parents first.  If the
    /// `Widget` is a composite's part, it is also taken out of the composite's parts, as
    /// [`remove_part`](WidgetCache::remove_part) does.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget` to remove.
    ///
    /// # Returns
    ///
    /// The IDs of every removed `Widget`, `id` first, then its descendants in drawing order.
    ///
    /// # Errors
    ///
    /// [`Error::Tree`] if `id` is the root, [`Error::UnknownWidget`] if `id` does not exist or
    /// was already removed.
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
    /// let panel = cache.add(Box::new(RootWidget::new(size(100, 100))));
    /// let label = cache.add_child(panel, Box::new(RootWidget::new(size(50, 20))));
    /// let after = cache.add(Box::new(RootWidget::new(size(10, 10))));
    ///
    /// assert_eq!(cache.remove(panel).unwrap(), vec![panel, label]);
    /// assert!(cache.get(panel).is_none());
    /// assert!(cache.get(after).is_some()); // other IDs are unchanged
    /// ```
    pub fn remove(&mut self, id: WidgetId) -> Result<Vec<WidgetId>> {
        self.check_movable(id, "the root cannot be removed")?;

        let removed = self.subtree(id);

        if let Some((composite, name)) = self.part_name(id) {
            self.remove_part(composite, &name)?;
        } else {
            self.remove_subtree(id);
        }

        Ok(removed)
    }

    /// Moves a `Widget`, with all of its descendants, to a new parent.  It becomes the new
    /// parent's last (top-most) child.  Its origin is kept, and is now relative to the new
    /// parent.  IDs do not change.  Emits [`EngineEvent::WidgetReparented`]; both parents lay
    /// their children out again.
    ///
    /// If `new_parent` is already the parent, nothing changes and no event is emitted; use
    /// [`set_stacking_index`](WidgetCache::set_stacking_index) to move a `Widget` among its
    /// siblings.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget` to move.
    /// * `new_parent` - The ID of its new parent.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the `Widget` has moved (or already had that parent).
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if either ID does not exist or was removed.  [`Error::Tree`] if
    /// `id` is the root, if `new_parent` is `id` or one of its descendants (the tree would have
    /// a loop), or if `id` is a composite's part (the composite owns where its parts go).
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
    /// let left = cache.add(Box::new(RootWidget::new(size(100, 100))));
    /// let right = cache.add(Box::new(RootWidget::new(size(100, 100))));
    /// let label = cache.add_child(left, Box::new(RootWidget::new(size(50, 20))));
    ///
    /// cache.reparent(label, right).unwrap();
    ///
    /// assert_eq!(cache.children_of(right), &[label]);
    /// assert!(cache.children_of(left).is_empty());
    /// ```
    pub fn reparent(&mut self, id: WidgetId, new_parent: WidgetId) -> Result<()> {
        self.check_movable(id, "the root cannot be moved")?;

        if self.get(new_parent).is_none() {
            return Err(Error::UnknownWidget { id: new_parent });
        }

        if new_parent == id || self.ancestors(new_parent).contains(&id) {
            return Err(Error::tree(
                id,
                format!("cannot move into its own subtree (widget {new_parent})"),
            ));
        }

        if self.part_name(id).is_some() {
            return Err(Error::tree(
                id,
                "is a composite's part; remove it from the composite instead",
            ));
        }

        let old_parent = self.cache[id as usize].get_parent();

        if old_parent == new_parent {
            return Ok(());
        }

        self.notify_ancestors(id);
        self.children[old_parent as usize].retain(|&child| child != id);
        self.children[new_parent as usize].push(id);
        self.cache[id as usize].core_mut().parent = new_parent;
        self.cache[old_parent as usize].request_layout();
        self.cache[new_parent as usize].request_layout();
        self.notify_ancestors(id);
        self.needs_full_redraw = true;
        self.queue_event(Event::Engine(EngineEvent::WidgetReparented {
            id,
            old_parent,
            new_parent,
        }));

        Ok(())
    }

    /// Returns a `Widget`'s position in its parent's stacking order.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// Its index among its siblings (`0` is drawn first, at the bottom), or `None` for the root
    /// and for IDs that do not exist or were removed.
    pub fn stacking_index(&self, id: WidgetId) -> Option<usize> {
        let parent = self.get(id)?.get_parent();

        if parent == id {
            return None;
        }

        self.children[parent as usize]
            .iter()
            .position(|&child| child == id)
    }

    /// Moves a `Widget` to a new place among its siblings.  Siblings are drawn in order, so
    /// index `0` is the bottom-most and the last index is the top-most; the `Widget`'s
    /// descendants move with it.  IDs do not change.  The parent lays its children out again, so
    /// a row or column layout follows the new order.  Emits [`EngineEvent::WidgetRestacked`] if
    /// the position changed.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget` to move.
    /// * `index` - Its new position among its siblings; past the end means top-most.
    ///
    /// # Returns
    ///
    /// The `Widget`'s position afterward (`index`, clamped to the last position).
    ///
    /// # Errors
    ///
    /// [`Error::Tree`] if `id` is the root, which has no siblings; [`Error::UnknownWidget`] if
    /// `id` does not exist or was removed.
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
    /// let back = cache.add(Box::new(RootWidget::new(size(100, 100))));
    /// let front = cache.add(Box::new(RootWidget::new(size(100, 100))));
    ///
    /// // Bring `back` to the front.
    /// assert_eq!(cache.set_stacking_index(back, usize::MAX).unwrap(), 1);
    /// assert_eq!(cache.children_of(0), &[front, back]);
    /// ```
    pub fn set_stacking_index(&mut self, id: WidgetId, index: usize) -> Result<usize> {
        self.check_movable(id, "the root has no siblings")?;

        let parent = self.cache[id as usize].get_parent();
        let siblings = &mut self.children[parent as usize];
        let old_index = siblings
            .iter()
            .position(|&child| child == id)
            .ok_or(Error::UnknownWidget { id })?;
        let new_index = index.min(siblings.len() - 1);

        if new_index == old_index {
            return Ok(new_index);
        }

        siblings.remove(old_index);
        siblings.insert(new_index, id);
        self.cache[parent as usize].request_layout();
        self.notify_ancestors(id);
        self.needs_full_redraw = true;
        self.queue_event(Event::Engine(EngineEvent::WidgetRestacked {
            id,
            parent,
            old_index,
            new_index,
        }));

        Ok(new_index)
    }

    /// Checks that a `Widget` exists and is not the root.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    /// * `root_message` - The [`Error::Tree`] message to use if `id` is the root.
    ///
    /// # Errors
    ///
    /// [`Error::Tree`] for the root, [`Error::UnknownWidget`] if `id` does not exist or was
    /// removed.
    fn check_movable(&self, id: WidgetId, root_message: &str) -> Result<()> {
        if self.get(id).is_none() {
            return Err(Error::UnknownWidget { id });
        }

        if id == ROOT_WIDGET_ID {
            return Err(Error::tree(id, root_message));
        }

        Ok(())
    }

    /// Lists a `Widget` and its descendants.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// `id` first, then its descendants in drawing order (parents before children).
    pub(super) fn subtree(&self, id: WidgetId) -> Vec<WidgetId> {
        let mut stack = vec![id];
        let mut ids = Vec::new();

        while let Some(current) = stack.pop() {
            ids.push(current);
            stack.extend(self.children[current as usize].iter().rev());
        }

        ids
    }

    /// Finds the composite that owns a part, and the part's name.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of a `Widget`.
    ///
    /// # Returns
    ///
    /// The composite's ID and the part's name, or `None` if `id` is not a part.
    fn part_name(&self, id: WidgetId) -> Option<(WidgetId, String)> {
        let composite = self.owner_of(id)?;
        let name = self
            .get(composite)?
            .as_composite()?
            .part_list()
            .name_of(id)?
            .to_string();

        Some((composite, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::headless::HeadlessBackend;
    use crate::geometry::size;
    use crate::root_widget::RootWidget;

    /// A root with two panels; the first panel has one child.
    ///
    /// Returns the cache and the IDs `[panel_a, child, panel_b]`.
    fn tree() -> (WidgetCache, [WidgetId; 3]) {
        let mut cache = WidgetCache::new();

        cache.add(Box::new(RootWidget::new(size(200, 200))));

        let panel_a = cache.add(Box::new(RootWidget::new(size(100, 100))));
        let child = cache.add_child(panel_a, Box::new(RootWidget::new(size(20, 20))));
        let panel_b = cache.add(Box::new(RootWidget::new(size(100, 100))));

        cache.take_pending_events();

        (cache, [panel_a, child, panel_b])
    }

    #[test]
    fn removing_queues_textures_for_release_on_the_next_draw() {
        let (mut cache, [panel_a, child, _]) = tree();
        let mut backend = HeadlessBackend::new(size(200, 200));

        cache.draw_loop(&mut backend);
        assert!(backend.texture_sizes.contains_key(&child));

        let drawn_before = backend.draws.len();

        cache.remove(panel_a).unwrap();
        cache.draw_loop(&mut backend);

        assert_eq!(backend.released, vec![panel_a, child]);
        assert!(!backend.texture_sizes.contains_key(&child));
        assert!(
            backend.draws[drawn_before..]
                .iter()
                .all(|(id, _)| *id != panel_a && *id != child)
        );
    }

    #[test]
    fn the_root_cannot_be_removed_moved_or_restacked() {
        let (mut cache, [panel_a, ..]) = tree();

        assert!(matches!(cache.remove(0), Err(Error::Tree { id: 0, .. })));
        assert!(matches!(
            cache.reparent(0, panel_a),
            Err(Error::Tree { id: 0, .. })
        ));
        assert!(matches!(
            cache.set_stacking_index(0, 0),
            Err(Error::Tree { id: 0, .. })
        ));
        assert_eq!(cache.stacking_index(0), None);
    }
}
