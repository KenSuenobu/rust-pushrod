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

//! Conformance for `docs/specs/widget.md`: the core, identity, parts, and invalidation.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::composite::{Composite, PartList, PartsBuilder};
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::paint::{self, Painter};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore, WidgetId};

/// A filled square: the smallest widget that satisfies the spec.
struct Swatch {
    core: WidgetCore,
}

impl Swatch {
    /// Creates a swatch at the window's upper left-hand corner.
    fn new() -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(8, 8)),
        }
    }
}

impl Widget for Swatch {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the swatch with its color.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }
}

/// A composite with one named part, "sample", and a list of "row" parts.
struct Table {
    core: WidgetCore,
    parts: PartList,
}

impl Widget for Table {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing; the parts draw themselves.
    ///
    /// # Arguments
    ///
    /// * `_p` - The painter.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Returns this widget as a composite.
    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    /// Returns this widget as a composite, mutably.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Table {
    /// Builds the parts: one named part and two list parts.
    ///
    /// # Arguments
    ///
    /// * `parts` - The builder.
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("sample", Swatch::new());
        parts.add("row:0", Swatch::new());
        parts.add("row:1", Swatch::new());
    }

    /// Returns the part list.
    fn part_list(&self) -> &PartList {
        &self.parts
    }

    /// Returns the part list, mutably.
    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }
}

/// Widget spec, "Core": a new widget is visible, enabled, in the normal state, and not yet
/// painted (so its first frame paints it).
#[test]
fn a_new_widget_starts_visible_enabled_and_invalidated() {
    let widget = Swatch::new();

    assert!(!widget.is_hidden());
    assert!(widget.is_enabled());
    assert!(!widget.is_focusable());
    assert!(widget.is_invalidated());
    assert_eq!(widget.get_size(), size(8, 8));
    assert!(widget.behavior().is_empty());
    assert!(!widget.state().contains(VisualState::DISABLED));
}

/// Widget spec, "Identity": IDs are handed out in order, the root is ID 0, and a removed widget's
/// ID is never reused.
#[test]
fn ids_are_stable_ordered_and_never_reused() {
    let mut cache = WidgetCache::new();
    let root = cache.add(Box::new(RootWidget::new(size(50, 50))));
    let first = cache.add_child(root, Box::new(Swatch::new()));
    let second = cache.add_child(root, Box::new(Swatch::new()));

    assert_eq!((root, first, second), (0, 1, 2));
    assert_eq!(cache.get(first).unwrap().get_id(), first);

    cache.remove(first).unwrap();
    assert!(cache.get(first).is_none(), "a removed widget is gone");

    let third = cache.add_child(root, Box::new(Swatch::new()));

    assert_eq!(third, 3, "IDs are not reused");
    assert_eq!(cache.get(second).unwrap().get_id(), second);
}

/// Widget spec, "Core": the shared `WidgetCore` is reachable through `get_as`, which returns the
/// concrete type.
#[test]
fn a_widget_can_be_fetched_as_its_concrete_type() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(50, 50))));

    let id = cache.add_child(0, Box::new(Swatch::new()));

    assert!(cache.get_as::<Swatch>(id).is_some());
    assert!(cache.get_as::<RootWidget>(id).is_none());
}

/// Widget spec, "Invalidation": a widget paints when invalidated and not otherwise.
#[test]
fn a_widget_paints_only_when_invalidated() {
    let mut engine: Engine<()> = Engine::new(size(40, 30), 30);
    let mut backend = HeadlessBackend::new(size(40, 30));
    let id: WidgetId = engine.add_widget(Swatch::new());

    assert!(engine.widgets().draw_loop(&mut backend));
    let painted = backend.painted.len();

    assert!(
        !engine.widgets().draw_loop(&mut backend),
        "nothing changed, so no frame"
    );
    assert_eq!(backend.painted.len(), painted);

    engine.widgets().get_mut(id).unwrap().set_invalidated(true);
    assert!(engine.widgets().draw_loop(&mut backend));
    assert_eq!(backend.painted.len(), painted + 1);
    assert_eq!(
        backend.painted.last().map(|(painted, _)| *painted),
        Some(id)
    );
}

/// Widget spec, "Behavior": hiding a widget invalidates it and sets the `HIDDEN` flag.
#[test]
fn hiding_a_widget_sets_the_flag_and_invalidates() {
    let mut widget = Swatch::new();

    widget.set_invalidated(false);
    widget.set_hidden(true);

    assert!(widget.is_hidden());
    assert!(widget.behavior().contains(Behavior::HIDDEN));
    assert!(widget.is_invalidated());
}

/// Widget spec, "Parts": a composite's parts are ordinary children with their own IDs, in the
/// order they were built, found by name or by list.
#[test]
fn parts_are_child_widgets_found_by_name_and_list() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(50, 50))));

    let table = cache.add_child(
        0,
        Box::new(Table {
            core: WidgetCore::new(point(0, 0), size(40, 40)),
            parts: PartList::new(),
        }),
    );
    let children = cache.children_of(table).to_vec();

    assert_eq!(children.len(), 3);
    assert_eq!(
        cache.parts(table),
        vec![
            ("sample".to_string(), children[0]),
            ("row:0".to_string(), children[1]),
            ("row:1".to_string(), children[2]),
        ]
    );
    assert_eq!(
        cache
            .get_as::<Table>(table)
            .unwrap()
            .part_list()
            .list("row"),
        vec![children[1], children[2]]
    );
}
