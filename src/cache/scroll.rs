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

//! Scrolled children: a `Widget` can show its children through a viewport.
//!
//! ```text
//!   child area (its own 0,0)             the Widget (viewport)
//!   +---------------------------+
//!   |        child_offset       |
//!   |          +--------+       |          +--------+
//!   |          | shown  |  ------------->  | shown  |
//!   |          +--------+       |          +--------+
//!   +---------------------------+
//! ```
//!
//! - [`WidgetCache::set_child_offset`] pans the children: each is drawn at
//!   `origin - child_offset` and cropped to the `Widget`.  Nothing is repainted; the frame
//!   copies the same textures to new places.
//! - [`WidgetCache::set_child_area`] gives the children an area larger (or smaller) than the
//!   `Widget`, which a layout manager arranges them in.
//! - [`Widget::descendant_changed`](crate::widget::Widget::descendant_changed) tells the
//!   `Widget`s above a change, so a scrolling container can follow its content.

use super::WidgetCache;
use crate::geometry::{Point, Size};
use crate::widget::WidgetId;

impl WidgetCache {
    /// Pans a `Widget`'s children: sets the point of its child area that is shown at its upper
    /// left-hand corner (see [`Widget::child_offset`](crate::widget::Widget::child_offset)).
    ///
    /// No `Widget` is repainted; the next frame copies the children's textures to their new
    /// places, cropped to the `Widget`.  The offset is not clamped: the caller decides how far
    /// the children may scroll.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` whose children scroll.
    /// * `offset` - The point of the child area to show at the upper left-hand corner.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` exists and the offset changed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::cache::WidgetCache;
    /// use pushrod::geometry::{point, size};
    /// use pushrod::root_widget::RootWidget;
    /// use pushrod::widget::Widget;
    ///
    /// let mut cache = WidgetCache::new();
    /// cache.add(Box::new(RootWidget::new(size(100, 100))));
    ///
    /// let mut child = RootWidget::new(size(20, 20));
    /// child.set_origin(point(10, 50));
    /// let child = cache.add(Box::new(child));
    ///
    /// assert!(cache.set_child_offset(0, point(0, 30)));
    /// assert_eq!(cache.absolute_origin(child), point(10, 20));
    /// ```
    pub fn set_child_offset(&mut self, id: WidgetId, offset: Point) -> bool {
        let Some(widget) = self.get_mut(id) else {
            return false;
        };

        if widget.child_offset() == offset {
            return false;
        }

        widget.core_mut().child_offset = offset;
        self.needs_full_redraw = true;

        true
    }

    /// Sets the size of the area a `Widget`'s children are placed on (see
    /// [`Widget::child_area`](crate::widget::Widget::child_area)), and marks its layout as
    /// needed, so a layout manager arranges the children in the new area.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` that holds the children.
    /// * `area` - The area's size, or `None` to place the children on the `Widget`'s own
    ///   rectangle again.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` exists and the area changed.
    pub fn set_child_area(&mut self, id: WidgetId, area: Option<Size>) -> bool {
        let Some(widget) = self.get_mut(id) else {
            return false;
        };

        if widget.child_area() == area {
            return false;
        }

        widget.core_mut().child_area = area;
        widget.request_layout();
        self.needs_full_redraw = true;

        true
    }

    /// Calls [`Widget::descendant_changed`](crate::widget::Widget::descendant_changed) on every
    /// ancestor of a `Widget`, nearest first.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` that changed; an unknown ID is ignored.
    pub(super) fn notify_ancestors(&mut self, id: WidgetId) {
        for ancestor in self.ancestors(id) {
            self.cache[ancestor as usize].descendant_changed();
        }
    }

    /// Calls [`Widget::descendant_changed`](crate::widget::Widget::descendant_changed) once on
    /// every `Widget` that has a descendant waiting to be repainted or laid out.
    /// [`WidgetCache::draw_loop`] calls it before layout.
    pub(super) fn notify_descendant_changes(&mut self) {
        let mut notified = vec![false; self.cache.len()];

        for index in 0..self.cache.len() {
            let widget = &self.cache[index];

            if !widget.is_invalidated() && !widget.core().layout_dirty() {
                continue;
            }

            let mut current = index;

            loop {
                let parent = self.cache[current].get_parent() as usize;

                // Stop at the root, and where an earlier walk already went up.
                if parent == current || parent >= notified.len() || notified[parent] {
                    break;
                }

                notified[parent] = true;
                self.cache[parent].descendant_changed();
                current = parent;
            }
        }
    }
}
