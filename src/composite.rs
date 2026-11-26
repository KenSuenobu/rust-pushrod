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

//! Composite `Widget`s: `Widget`s built from parts that applications can retrieve and modify.
//!
//! A composite (an alert, a scroll bar, a button bar) is made of simpler `Widget`s.  Its parts
//! are **ordinary child `Widget`s with their own IDs**, not hidden internals, so an application
//! reaches them with the normal API: `ui.get_mut::<Text>(message)?.set_font_color(..)`.
//!
//! ```text
//!   ButtonBar (composite, id 1)
//!    ├── "title"     Text    id 2
//!    ├── "button:0"  Button  id 3
//!    └── "button:1"  Button  id 4   <- added later with WidgetCache::add_part
//! ```
//!
//! - [`Composite::build`] runs when the composite is added to a `WidgetCache`, and creates the
//!   parts through a [`PartsBuilder`], each under a stable [`PartName`] (`"title"`, `"button:0"`).
//! - [`Composite::parts`] and [`Composite::part`] list and find parts by name; a composite also
//!   implements [`TypedParts`] to return its structure as a struct of IDs.
//! - [`WidgetCache::add_part`](crate::cache::WidgetCache::add_part),
//!   [`insert_part`](crate::cache::WidgetCache::insert_part), and
//!   [`remove_part`](crate::cache::WidgetCache::remove_part) change a list-structured composite
//!   after it is built, and [`Composite::layout_parts`] lays the parts out again each time.
//! - Events a part emits reach [`Composite::handle_part_event`] first, so the composite can
//!   translate them (an arrow's `Clicked` into a `ValueChanged`).  The part's event is still
//!   delivered, so callbacks registered on the part's ID run too, followed by the composite's
//!   translated events.
//!
//! # Examples
//!
//! ```rust
//! use pushrod::cache::WidgetCache;
//! use pushrod::composite::{Composite, PartList, PartsBuilder};
//! use pushrod::geometry::{Rect, size};
//! use pushrod::paint::{self, Painter};
//! use pushrod::root_widget::RootWidget;
//! use pushrod::widget::{Widget, WidgetCore, WidgetId};
//!
//! /// Two colored squares, side by side.
//! struct Pair {
//!     core: WidgetCore,
//!     parts: PartList,
//! }
//!
//! impl Widget for Pair {
//!     fn core(&self) -> &WidgetCore { &self.core }
//!     fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }
//!     fn paint(&self, p: &mut dyn Painter) { paint::fill(p, self.get_color()); }
//!     fn as_composite(&self) -> Option<&dyn Composite> { Some(self) }
//!     fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> { Some(self) }
//! }
//!
//! impl Composite for Pair {
//!     fn build(&mut self, parts: &mut PartsBuilder) {
//!         parts.add("left", RootWidget::new(size(10, 10)));
//!         parts.add("right", RootWidget::new(size(10, 10)));
//!     }
//!
//!     fn part_list(&self) -> &PartList { &self.parts }
//!     fn part_list_mut(&mut self) -> &mut PartList { &mut self.parts }
//!
//!     fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
//!         self.parts
//!             .ids()
//!             .into_iter()
//!             .enumerate()
//!             .map(|(i, id)| (id, Rect::new(i as i32 * 10, 0, 10, 10)))
//!             .collect()
//!     }
//! }
//!
//! let mut cache = WidgetCache::new();
//! cache.add(Box::new(RootWidget::new(size(100, 100))));
//!
//! let pair = cache.add(Box::new(Pair {
//!     core: WidgetCore::new(Default::default(), size(20, 10)),
//!     parts: PartList::new(),
//! }));
//! let right = cache.part(pair, "right").unwrap();
//!
//! assert_eq!(cache.get(right).unwrap().get_parent(), pair);
//! assert_eq!(cache.get(right).unwrap().get_origin().x, 10);
//! ```

use crate::cache::WidgetCache;
use crate::event::{Event, EventCtx};
use crate::geometry::Rect;
use crate::widget::{Widget, WidgetId};

/// The stable name of a composite's part, such as `"message"`, `"thumb"`, or `"button:0"`.
///
/// Parts that form a list use `"<kind>:<index>"`; [`PartList::renumber`] keeps the indexes in
/// order after a part is inserted or removed.
pub type PartName = String;

/// A composite's parts, by name, in order.
///
/// A composite stores one of these and returns it from [`Composite::part_list`].  The
/// `WidgetCache` fills it while the composite is built and updates it when parts are added,
/// inserted, or removed; a composite normally only reads it.
///
/// # Examples
///
/// ```rust
/// use pushrod::composite::PartList;
///
/// let parts = PartList::new();
///
/// assert!(parts.is_empty());
/// assert_eq!(parts.get("title"), None);
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PartList {
    parts: Vec<(PartName, WidgetId)>,
}

impl PartList {
    /// Creates an empty list of parts.
    ///
    /// # Returns
    ///
    /// A list with no parts.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns every part, in order.
    ///
    /// # Returns
    ///
    /// Each part's name and `Widget` ID.
    pub fn as_slice(&self) -> &[(PartName, WidgetId)] {
        &self.parts
    }

    /// Returns the ID of every part, in order.
    ///
    /// # Returns
    ///
    /// The part IDs.
    pub fn ids(&self) -> Vec<WidgetId> {
        self.parts.iter().map(|(_, id)| *id).collect()
    }

    /// Finds a part by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The part's name, such as `"title"`.
    ///
    /// # Returns
    ///
    /// The part's `Widget` ID, or `None` if no part has that name.
    pub fn get(&self, name: &str) -> Option<WidgetId> {
        self.parts
            .iter()
            .find(|(part, _)| part == name)
            .map(|(_, id)| *id)
    }

    /// Finds the name of a part by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The part's `Widget` ID.
    ///
    /// # Returns
    ///
    /// The part's name, or `None` if the ID is not one of these parts.
    pub fn name_of(&self, id: WidgetId) -> Option<&str> {
        self.parts
            .iter()
            .find(|(_, part)| *part == id)
            .map(|(name, _)| name.as_str())
    }

    /// Returns whether an ID is one of these parts.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` ID.
    ///
    /// # Returns
    ///
    /// `true` if `id` is a part.
    pub fn contains(&self, id: WidgetId) -> bool {
        self.name_of(id).is_some()
    }

    /// Returns the parts of one list, such as every `"button:<n>"`, in order.
    ///
    /// # Arguments
    ///
    /// * `kind` - The list's name before the colon, such as `"button"`.
    ///
    /// # Returns
    ///
    /// The IDs of the parts named `"<kind>:<anything>"`, in list order.
    pub fn list(&self, kind: &str) -> Vec<WidgetId> {
        self.parts
            .iter()
            .filter(|(name, _)| in_list(name, kind))
            .map(|(_, id)| *id)
            .collect()
    }

    /// Returns where a list item belongs in the whole list of parts.
    ///
    /// # Arguments
    ///
    /// * `kind` - The list's name before the colon, such as `"button"`.
    /// * `index` - The item's position within that list; past the end means after its last item.
    ///
    /// # Returns
    ///
    /// The index in the whole list of parts to pass to
    /// [`WidgetCache::insert_part`](crate::cache::WidgetCache::insert_part).
    pub fn list_position(&self, kind: &str, index: usize) -> usize {
        let positions: Vec<usize> = self
            .parts
            .iter()
            .enumerate()
            .filter(|(_, (name, _))| in_list(name, kind))
            .map(|(position, _)| position)
            .collect();

        match positions.get(index) {
            Some(position) => *position,
            None => positions.last().map_or(self.parts.len(), |last| last + 1),
        }
    }

    /// Renames the parts of one list to `"<kind>:0"`, `"<kind>:1"`, and so on, in order.  Call
    /// it after inserting or removing a list item so the names stay in step with the positions.
    ///
    /// # Arguments
    ///
    /// * `kind` - The list's name before the colon, such as `"button"`.
    pub fn renumber(&mut self, kind: &str) {
        let mut index = 0;

        for (name, _) in self.parts.iter_mut() {
            if in_list(name, kind) {
                *name = format!("{kind}:{index}");
                index += 1;
            }
        }
    }

    /// Returns the number of parts.
    ///
    /// # Returns
    ///
    /// How many parts there are.
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// Returns whether there are no parts.
    ///
    /// # Returns
    ///
    /// `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Inserts a part, clamping `index` to the end of the list.
    ///
    /// # Arguments
    ///
    /// * `index` - Where the part goes.
    /// * `name` - The part's name.
    /// * `id` - The part's `Widget` ID.
    pub(crate) fn insert(&mut self, index: usize, name: PartName, id: WidgetId) {
        let index = index.min(self.parts.len());

        self.parts.insert(index, (name, id));
    }

    /// Removes a part by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The part's name.
    ///
    /// # Returns
    ///
    /// The removed part's ID, or `None` if no part has that name.
    pub(crate) fn remove(&mut self, name: &str) -> Option<WidgetId> {
        let position = self.parts.iter().position(|(part, _)| part == name)?;

        Some(self.parts.remove(position).1)
    }
}

/// Returns whether a part name belongs to the list `kind` (`"<kind>:<anything>"`).
fn in_list(name: &str, kind: &str) -> bool {
    name.strip_prefix(kind)
        .is_some_and(|rest| rest.starts_with(':'))
}

/// Creates a composite's parts while it is being built; passed to [`Composite::build`].
///
/// Each part is added to the `WidgetCache` at once, as a child of the composite, so its ID is
/// known immediately and the composite can keep it for its typed accessors.  A part that is
/// itself a composite is built in turn.
#[derive(Debug)]
pub struct PartsBuilder<'a> {
    cache: &'a mut WidgetCache,
    owner: WidgetId,
    parts: PartList,
}

impl<'a> PartsBuilder<'a> {
    /// Creates a builder for a composite already in the cache.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the composite.
    /// * `owner` - The composite's ID.
    ///
    /// # Returns
    ///
    /// A builder with no parts yet.
    pub(crate) fn new(cache: &'a mut WidgetCache, owner: WidgetId) -> Self {
        Self {
            cache,
            owner,
            parts: PartList::new(),
        }
    }

    /// Returns the ID of the composite being built.
    ///
    /// # Returns
    ///
    /// The composite's `Widget` ID.
    pub fn owner(&self) -> WidgetId {
        self.owner
    }

    /// Adds a part, as the composite's next child, under a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The part's stable name, such as `"title"` or `"button:0"`.
    /// * `widget` - The part.  Its origin is relative to the composite; if the composite
    ///   implements [`Composite::layout_parts`], the layout places it after the build.
    ///
    /// # Returns
    ///
    /// The part's `Widget` ID.
    ///
    /// # Panics
    ///
    /// Panics if the composite already has a part with this name.
    pub fn add<W: Widget>(&mut self, name: impl Into<PartName>, widget: W) -> WidgetId {
        let name = name.into();

        if self.parts.get(&name).is_some() {
            panic!(
                "[PartsBuilder::add] composite {} already has a part named \"{name}\"",
                self.owner
            );
        }

        let id = self.cache.add_child(self.owner, Box::new(widget));

        self.parts.insert(self.parts.len(), name, id);
        id
    }

    /// Ends the build and returns the parts created.
    ///
    /// # Returns
    ///
    /// The parts, in the order added.
    pub(crate) fn finish(self) -> PartList {
        self.parts
    }
}

/// A `Widget` made of parts that are ordinary child `Widget`s.
///
/// Implement it next to [`Widget`], and override [`Widget::as_composite`] and
/// [`Widget::as_composite_mut`] to return `Some(self)` so the `WidgetCache` can find it.  See
/// the [module documentation](self) for a complete example.
pub trait Composite: Widget {
    /// Creates the composite's parts.  The `WidgetCache` calls this once, when the composite is
    /// added; the composite already has its ID, and each part gets the next free ID.
    ///
    /// # Arguments
    ///
    /// * `parts` - Adds the parts; see [`PartsBuilder::add`].
    fn build(&mut self, parts: &mut PartsBuilder);

    /// Returns the composite's parts.
    ///
    /// # Returns
    ///
    /// The [`PartList`] the composite stores.
    fn part_list(&self) -> &PartList;

    /// Returns the composite's parts for modification.  The `WidgetCache` updates them when
    /// parts are built, added, inserted, or removed.
    ///
    /// # Returns
    ///
    /// The [`PartList`] the composite stores.
    fn part_list_mut(&mut self) -> &mut PartList;

    /// Lists every part by its stable name.
    ///
    /// # Returns
    ///
    /// Each part's name and `Widget` ID, in order.
    fn parts(&self) -> Vec<(PartName, WidgetId)> {
        self.part_list().as_slice().to_vec()
    }

    /// Finds a part by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The part's name, such as `"message"` or `"button:0"`.
    ///
    /// # Returns
    ///
    /// The part's `Widget` ID, or `None` if there is no such part.
    fn part(&self, name: &str) -> Option<WidgetId> {
        self.part_list().get(name)
    }

    /// Says where each part goes.  The `WidgetCache` calls this after the build, after each
    /// part is added, inserted, or removed, and on
    /// [`WidgetCache::arrange_parts`](crate::cache::WidgetCache::arrange_parts), then moves and
    /// resizes the parts that changed.
    ///
    /// # Returns
    ///
    /// Each part to place and its rectangle relative to the composite.  Parts left out keep
    /// their position.  The default places nothing.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        Vec::new()
    }

    /// Brings the parts up to date with the composite's own state, with access to the whole
    /// `WidgetCache`: set a part's value, hide it, or read what the parts hold.  The
    /// `WidgetCache` calls this just before [`Composite::layout_parts`], every time the parts
    /// are arranged.
    ///
    /// While it runs the composite is out of the cache (its slot holds a hidden placeholder), so
    /// reach the composite through `self`, not through `cache`.  Events emitted through `ctx`
    /// are delivered to the application like any other.  The default does nothing.
    ///
    /// # Arguments
    ///
    /// * `_cache` - The cache holding the parts.
    /// * `_ctx` - Queues the events the composite generates.
    fn sync_parts(&mut self, _cache: &mut WidgetCache, _ctx: &mut EventCtx) {}

    /// Receives an event one of the parts emitted, before it reaches the application.  Emit
    /// translated events through `ctx`, such as the scroll bar's own `ValueChanged` for an
    /// arrow's `Clicked`.  The part's event is still delivered (callbacks on the part's ID run),
    /// and the translated events are delivered right after it.  The default ignores part events.
    ///
    /// # Arguments
    ///
    /// * `_part` - The ID of the part that emitted the event.
    /// * `_event` - The event.
    /// * `_ctx` - Queues the events the composite generates.
    fn handle_part_event(&mut self, _part: WidgetId, _event: &Event, _ctx: &mut EventCtx) {}
}

/// A composite's structure as a struct of part IDs, such as
/// `AlertParts { icon, title, message, buttons }`.
///
/// It is separate from [`Composite`] so `Composite` can still be used as `dyn Composite`.
pub trait TypedParts: Composite {
    /// The struct of part IDs.
    type Parts;

    /// Returns the composite's parts as a struct.
    ///
    /// # Returns
    ///
    /// The part IDs, by role.
    fn typed_parts(&self) -> Self::Parts;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> PartList {
        let mut parts = PartList::new();

        parts.insert(0, "title".into(), 2);
        parts.insert(1, "button:0".into(), 3);
        parts.insert(2, "button:1".into(), 4);
        parts.insert(3, "footer".into(), 5);
        parts
    }

    #[test]
    fn parts_are_found_by_name_and_id() {
        let parts = list();

        assert_eq!(parts.get("button:1"), Some(4));
        assert_eq!(parts.get("missing"), None);
        assert_eq!(parts.name_of(2), Some("title"));
        assert!(parts.contains(5));
        assert!(!parts.contains(9));
        assert_eq!(parts.ids(), vec![2, 3, 4, 5]);
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn a_list_is_the_parts_named_kind_colon_anything() {
        let mut parts = list();

        parts.insert(1, "buttons".into(), 6);

        assert_eq!(parts.list("button"), vec![3, 4]);
        assert_eq!(parts.list_position("button", 0), 2);
        assert_eq!(parts.list_position("button", 1), 3);
        assert_eq!(parts.list_position("button", 9), 4);
        assert_eq!(parts.list_position("tab", 0), parts.len());
    }

    #[test]
    fn renumbering_follows_list_order() {
        let mut parts = list();

        parts.insert(1, "button:new".into(), 7);
        parts.renumber("button");

        assert_eq!(parts.get("button:0"), Some(7));
        assert_eq!(parts.get("button:1"), Some(3));
        assert_eq!(parts.get("button:2"), Some(4));
        assert_eq!(parts.remove("button:1"), Some(3));
        assert_eq!(parts.remove("button:1"), None);
        assert_eq!(parts.remove("button:2"), Some(4));
        assert_eq!(parts.remove("nothing"), None);
    }
}
