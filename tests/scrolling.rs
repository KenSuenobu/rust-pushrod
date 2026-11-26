// Pushrod
// Scrolling tests
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

//! Tests for what the core gives a scrolling container: scrolled children, a child area,
//! wheel events, notice of changes below, and a composite that brings its parts up to date.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::composite::{Composite, PartList, PartsBuilder};
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Point, Rect, Size, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::layout::{ChildInfo, Layout, SizeHint};
use pushrod::paint::{self, Painter};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// A plain rectangle that counts the changes reported below it and logs the scrolls it sees.
struct Block {
    core: WidgetCore,
    changes: Rc<Cell<usize>>,
    scrolls: Rc<RefCell<Vec<(WidgetId, Point)>>>,
    stop_scroll: bool,
}

impl Block {
    fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        let mut core = WidgetCore::new(point(x, y), size(w, h));

        core.set_event_mask(EventMask::SCROLL);

        Self {
            core,
            changes: Rc::default(),
            scrolls: Rc::default(),
            stop_scroll: false,
        }
    }
}

impl Widget for Block {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventCtx) -> Propagation {
        if let Event::Scrolled { delta, .. } = event {
            self.scrolls.borrow_mut().push((self.get_id(), *delta));

            if self.stop_scroll {
                return Propagation::Stop;
            }
        }

        Propagation::Continue
    }

    fn descendant_changed(&mut self) {
        self.changes.set(self.changes.get() + 1);
    }
}

/// A layout that gives every child the whole area it is handed.
#[derive(Debug)]
struct Fill;

impl Layout for Fill {
    fn measure(&self, _children: &[ChildInfo], available: Size) -> SizeHint {
        SizeHint::fixed(available)
    }

    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
        children.iter().map(|_| bounds).collect()
    }
}

/// A composite with one part, which it resizes and counts from `sync_parts`.
struct Meter {
    core: WidgetCore,
    parts: PartList,
    level: u32,
    synced: Rc<Cell<usize>>,
}

impl Meter {
    fn new(synced: &Rc<Cell<usize>>) -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(100, 10)),
            parts: PartList::new(),
            level: 0,
            synced: synced.clone(),
        }
    }
}

impl Widget for Meter {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }

    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Meter {
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("bar", Block::new(0, 0, 1, 10));
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    fn sync_parts(&mut self, cache: &mut WidgetCache, ctx: &mut EventCtx) {
        self.synced.set(self.synced.get() + 1);

        // The composite is out of the cache while this runs; its slot is a hidden placeholder.
        assert!(cache.get(self.get_id()).is_some_and(|w| w.is_hidden()));

        if let Some(bar) = self.parts.get("bar").and_then(|id| cache.get_mut(id)) {
            bar.set_hidden(self.level == 0);
        }

        if self.level > 0 {
            ctx.emit(Event::ValueChanged { id: self.get_id() });
        }
    }

    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        self.parts
            .get("bar")
            .map(|id| (id, Rect::new(0, 0, self.level, self.get_size().h)))
            .into_iter()
            .collect()
    }
}

/// A cache with a 200 by 200 root.
fn cache() -> WidgetCache {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));
    cache
}

fn backend() -> HeadlessBackend {
    HeadlessBackend::new(size(200, 200))
}

#[test]
fn a_child_offset_moves_the_children_on_screen() {
    let mut cache = cache();
    let view = cache.add(Box::new(Block::new(20, 20, 100, 50)));
    let child = cache.add_child(view, Box::new(Block::new(10, 80, 30, 30)));
    let grandchild = cache.add_child(child, Box::new(Block::new(5, 5, 10, 10)));

    assert_eq!(cache.get(view).unwrap().child_offset(), point(0, 0));
    assert_eq!(cache.absolute_origin(child), point(30, 100));

    assert!(cache.set_child_offset(view, point(0, 60)));

    assert_eq!(cache.get(view).unwrap().child_offset(), point(0, 60));
    assert_eq!(cache.absolute_origin(child), point(30, 40));
    assert_eq!(cache.absolute_origin(grandchild), point(35, 45));
    // The viewport itself does not move, and the child's own origin is unchanged.
    assert_eq!(cache.absolute_origin(view), point(20, 20));
    assert_eq!(cache.get(child).unwrap().get_origin(), point(10, 80));
}

#[test]
fn scrolled_children_are_cropped_and_hit_through_the_viewport() {
    let mut cache = cache();
    let view = cache.add(Box::new(Block::new(20, 20, 100, 50)));
    let child = cache.add_child(view, Box::new(Block::new(10, 80, 30, 30)));

    // Below the viewport: not visible, not clickable.
    assert_eq!(cache.visible_rect(child), None);
    assert_eq!(cache.widget_at(35, 105), 0);

    cache.set_child_offset(view, point(0, 50));

    // Now at window y 50, with the viewport ending at 70.
    assert_eq!(cache.visible_rect(child), Some(Rect::new(30, 50, 30, 20)));
    assert_eq!(cache.widget_at(35, 55), child);
    assert_eq!(cache.widget_at(35, 75), 0);
}

#[test]
fn setting_the_same_offset_or_an_unknown_widget_changes_nothing() {
    let mut cache = cache();
    let view = cache.add(Box::new(Block::new(0, 0, 100, 50)));

    assert!(!cache.set_child_offset(view, point(0, 0)));
    assert!(!cache.set_child_offset(99, point(0, 10)));
    assert!(!cache.set_child_area(99, Some(size(1, 1))));
    assert!(cache.set_child_area(view, Some(size(100, 500))));
    assert!(!cache.set_child_area(view, Some(size(100, 500))));
}

#[test]
fn panning_redraws_without_repainting() {
    let mut cache = cache();
    let mut backend = backend();
    let view = cache.add(Box::new(Block::new(0, 0, 100, 50)));
    let child = cache.add_child(view, Box::new(Block::new(0, 0, 30, 30)));

    assert!(cache.draw_loop(&mut backend));
    assert!(!cache.needs_draw());
    backend.painted.clear();
    backend.draws.clear();

    cache.set_child_offset(view, point(0, 10));

    assert!(cache.needs_draw());
    assert!(cache.draw_loop(&mut backend));
    assert!(backend.painted.is_empty());
    assert!(backend.draws.contains(&(child, Rect::new(0, 0, 30, 20))));
    assert!(
        backend
            .draw_sources
            .contains(&(child, Rect::new(0, 10, 30, 20)))
    );
}

#[test]
fn a_layout_arranges_children_in_the_child_area() {
    let mut cache = cache();
    let mut backend = backend();
    let view = cache.add(Box::new(Block::new(0, 0, 100, 50)));
    let child = cache.add_child(view, Box::new(Block::new(0, 0, 10, 10)));

    cache.get_mut(view).unwrap().set_layout(Box::new(Fill));
    cache.draw_loop(&mut backend);
    assert_eq!(cache.get(child).unwrap().get_size(), size(100, 50));

    assert_eq!(cache.get(view).unwrap().child_area(), None);
    cache.set_child_area(view, Some(size(100, 400)));
    cache.draw_loop(&mut backend);

    assert_eq!(cache.get(view).unwrap().child_area(), Some(size(100, 400)));
    assert_eq!(cache.get(child).unwrap().get_size(), size(100, 400));
    // The viewport keeps its own size.
    assert_eq!(cache.get(view).unwrap().get_size(), size(100, 50));

    cache.set_child_area(view, None);
    cache.draw_loop(&mut backend);
    assert_eq!(cache.get(child).unwrap().get_size(), size(100, 50));
}

#[test]
fn the_wheel_scrolls_the_widget_under_the_pointer_and_bubbles() {
    let mut cache = cache();
    let outer = Block::new(0, 0, 100, 100);
    let inner = Block::new(10, 10, 50, 50);
    let log = outer.scrolls.clone();
    let inner_log = inner.scrolls.clone();
    let outer = cache.add(Box::new(outer));
    let inner = cache.add_child(outer, Box::new(inner));

    cache.handle_event(InputEvent::MouseMove {
        position: point(20, 20),
    });
    cache.handle_event(InputEvent::Wheel {
        delta: point(0, -2),
    });

    assert_eq!(*inner_log.borrow(), vec![(inner, point(0, -2))]);
    assert_eq!(*log.borrow(), vec![(outer, point(0, -2))]);
}

#[test]
fn a_widget_that_stops_a_scroll_keeps_it_from_its_parents() {
    let mut cache = cache();
    let outer = Block::new(0, 0, 100, 100);
    let mut inner = Block::new(10, 10, 50, 50);
    let log = outer.scrolls.clone();

    inner.stop_scroll = true;

    let outer = cache.add(Box::new(outer));

    cache.add_child(outer, Box::new(inner));
    cache.handle_event(InputEvent::MouseMove {
        position: point(20, 20),
    });
    cache.handle_event(InputEvent::Wheel { delta: point(0, 1) });

    assert!(log.borrow().is_empty());
}

#[test]
fn ancestors_hear_of_changes_below_once_per_frame() {
    let mut cache = cache();
    let mut backend = backend();
    let outer = Block::new(0, 0, 100, 100);
    let changes = outer.changes.clone();
    let outer = cache.add(Box::new(outer));
    let middle = cache.add_child(outer, Box::new(Block::new(0, 0, 50, 50)));
    let first = cache.add_child(middle, Box::new(Block::new(0, 0, 10, 10)));
    let second = cache.add_child(middle, Box::new(Block::new(20, 0, 10, 10)));

    cache.draw_loop(&mut backend);
    changes.set(0);

    // Nothing changed: nothing is reported.
    cache.draw_loop(&mut backend);
    assert_eq!(changes.get(), 0);

    // Two descendants change in one frame: one report.
    cache.get_mut(first).unwrap().set_origin(point(5, 5));
    cache.get_mut(second).unwrap().set_size(size(12, 12));
    cache.draw_loop(&mut backend);
    assert_eq!(changes.get(), 1);

    // A change to the `Widget` itself is not a change below it.
    cache.get_mut(outer).unwrap().set_invalidated(true);
    cache.draw_loop(&mut backend);
    assert_eq!(changes.get(), 1);
}

#[test]
fn ancestors_hear_when_a_descendant_takes_the_focus() {
    let mut cache = cache();
    let outer = Block::new(0, 0, 100, 100);
    let changes = outer.changes.clone();
    let outer = cache.add(Box::new(outer));
    let mut field = Block::new(0, 0, 10, 10);

    field.set_behavior(Behavior::FOCUSABLE);

    let field = cache.add_child(outer, Box::new(field));

    assert!(cache.set_focus(field));
    assert_eq!(changes.get(), 1);
}

#[test]
fn a_composite_syncs_its_parts_each_time_they_are_arranged() {
    let mut cache = cache();
    let mut backend = backend();
    let synced = Rc::new(Cell::new(0));
    let meter = cache.add(Box::new(Meter::new(&synced)));
    let bar = cache.part(meter, "bar").unwrap();

    // Once when built.
    assert_eq!(synced.get(), 1);
    assert!(cache.get(bar).unwrap().is_hidden());
    cache.take_pending_events();

    let widget = cache.get_as_mut::<Meter>(meter).unwrap();

    widget.level = 40;
    widget.request_arrange_parts();
    cache.draw_loop(&mut backend);

    assert_eq!(synced.get(), 2);
    assert!(!cache.get(bar).unwrap().is_hidden());
    assert_eq!(cache.get(bar).unwrap().get_size(), size(40, 10));
    // The composite is back in its slot, and the event it emitted is queued for delivery.
    assert!(cache.get_as::<Meter>(meter).is_some());
    assert!(
        cache
            .take_pending_events()
            .iter()
            .any(|event| matches!(event, Event::ValueChanged { id } if *id == meter))
    );
}

#[test]
fn resizing_a_composite_arranges_its_parts_again() {
    let mut cache = cache();
    let mut backend = backend();
    let synced = Rc::new(Cell::new(0));
    let meter = cache.add(Box::new(Meter::new(&synced)));
    let bar = cache.part(meter, "bar").unwrap();

    cache.get_as_mut::<Meter>(meter).unwrap().level = 40;
    cache.draw_loop(&mut backend);

    cache.get_mut(meter).unwrap().set_size(size(100, 24));
    cache.draw_loop(&mut backend);

    assert_eq!(cache.get(bar).unwrap().get_size(), size(40, 24));
    assert!(!cache.needs_draw());
}

#[test]
fn a_press_reaches_a_scrolled_child_in_its_own_coordinates() {
    let mut cache = cache();
    let view = cache.add(Box::new(Block::new(20, 20, 100, 50)));
    let child = cache.add_child(view, Box::new(Block::new(0, 100, 30, 30)));

    cache.set_child_offset(view, point(0, 90));

    // The child's corner is now at window (20, 30).
    cache.handle_event(InputEvent::MouseDown {
        position: point(25, 35),
        button: MouseButton::Left,
        clicks: 1,
    });

    assert_eq!(cache.get_current_widget(), child);
    assert_eq!(cache.absolute_origin(child), point(20, 30));
}
