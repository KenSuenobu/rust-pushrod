//! Layout managers: measure and arrange once per frame, invalidation that stops early, moves that
//! do not repaint, and application-defined layouts.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::event::{EngineEvent, Event};
use pushrod::geometry::{Rect, Size, point, size};
use pushrod::layout::{ChildInfo, Layout, LayoutParams, Margin, SizeHint, place_in_slot};
use pushrod::root_widget::RootWidget;
use pushrod::widget::WidgetId;
use std::cell::Cell;
use std::rc::Rc;

/// How many times a layout ran.
#[derive(Debug, Default, Clone)]
struct Calls {
    measure: Rc<Cell<usize>>,
    arrange: Rc<Cell<usize>>,
}

/// An application-defined layout: stacks the children in a column.
#[derive(Debug)]
struct Stack {
    calls: Option<Calls>,
}

impl Stack {
    /// Creates a stack that counts its calls.
    fn counting(calls: &Calls) -> Self {
        Self {
            calls: Some(calls.clone()),
        }
    }

    /// Creates a stack that counts nothing.
    fn plain() -> Self {
        Self { calls: None }
    }
}

impl Layout for Stack {
    fn measure(&self, children: &[ChildInfo], _available: Size) -> SizeHint {
        if let Some(calls) = &self.calls {
            calls.measure.set(calls.measure.get() + 1);
        }

        let w = children
            .iter()
            .map(|c| c.hint.preferred.w + c.params.margin.total().w)
            .max()
            .unwrap_or(0);
        let h = children
            .iter()
            .map(|c| c.hint.preferred.h + c.params.margin.total().h)
            .sum();

        SizeHint::fixed(size(w, h))
    }

    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
        if let Some(calls) = &self.calls {
            calls.arrange.set(calls.arrange.get() + 1);
        }

        let mut y = bounds.y;

        children
            .iter()
            .map(|c| {
                let height = c.hint.preferred.h + c.params.margin.total().h;
                let slot = Rect::new(bounds.x, y, bounds.w, height);

                y += height as i32;
                place_in_slot(slot, c)
            })
            .collect()
    }
}

/// A layout whose size never depends on its children.
#[derive(Debug)]
struct Fixed;

impl Layout for Fixed {
    fn measure(&self, _children: &[ChildInfo], _available: Size) -> SizeHint {
        SizeHint::fixed(size(50, 50))
    }

    fn arrange(&self, children: &[ChildInfo], _bounds: Rect) -> Vec<Rect> {
        children
            .iter()
            .map(|c| Rect::new(1, 1, c.hint.preferred.w, c.hint.preferred.h))
            .collect()
    }
}

/// Adds a child of a size under a parent.
fn add(cache: &mut WidgetCache, parent: WidgetId, w: u32, h: u32) -> WidgetId {
    cache.add_child(parent, Box::new(RootWidget::new(size(w, h))))
}

/// A cache with a 100x200 root that stacks three children 40x10, 40x20, and 40x30.
fn stacked(calls: &Calls) -> (WidgetCache, [WidgetId; 3]) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(100, 200))));

    let kids = [
        add(&mut cache, 0, 40, 10),
        add(&mut cache, 0, 40, 20),
        add(&mut cache, 0, 40, 30),
    ];

    cache
        .get_mut(0)
        .unwrap()
        .set_layout(Box::new(Stack::counting(calls)));

    (cache, kids)
}

/// Draws one frame, returning the engine events it queued.
fn frame(cache: &mut WidgetCache, backend: &mut HeadlessBackend) -> Vec<EngineEvent> {
    backend.painted.clear();
    cache.draw_loop(backend);
    cache
        .take_pending_events()
        .into_iter()
        .filter_map(|event| match event {
            Event::Engine(engine) => Some(engine),
            _ => None,
        })
        .collect()
}

/// The widgets repainted by the last frame.
fn repainted(backend: &HeadlessBackend) -> Vec<WidgetId> {
    backend.painted.iter().map(|(id, _)| *id).collect()
}

#[test]
fn without_a_layout_absolute_positions_are_kept() {
    let mut cache = WidgetCache::new();
    let mut backend = HeadlessBackend::new(size(100, 100));

    cache.add(Box::new(RootWidget::new(size(100, 100))));

    let child = add(&mut cache, 0, 30, 20);

    cache.get_mut(child).unwrap().set_origin(point(7, 9));
    cache.get_mut(child).unwrap().set_size(size(31, 21));

    let events = frame(&mut cache, &mut backend);

    assert_eq!(cache.get(child).unwrap().get_origin(), point(7, 9));
    assert_eq!(cache.get(child).unwrap().get_size(), size(31, 21));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, EngineEvent::LayoutChanged { .. }))
    );
}

#[test]
fn a_container_with_a_layout_places_its_children() {
    let calls = Calls::default();
    let (mut cache, kids) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(kids[0]).unwrap().get_origin(), point(0, 0));
    assert_eq!(cache.get(kids[1]).unwrap().get_origin(), point(0, 10));
    assert_eq!(cache.get(kids[2]).unwrap().get_origin(), point(0, 30));
    assert_eq!(cache.get(kids[2]).unwrap().get_size(), size(40, 30));
    assert_eq!(
        cache.measured_hint(0).map(|h| h.preferred),
        Some(size(40, 60))
    );
}

#[test]
fn layout_runs_once_per_frame_and_only_when_dirty() {
    let calls = Calls::default();
    let (mut cache, kids) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    frame(&mut cache, &mut backend);

    assert_eq!(calls.arrange.get(), 1);

    // Nothing changed: no layout.
    let before = (calls.measure.get(), calls.arrange.get());

    frame(&mut cache, &mut backend);

    assert_eq!((calls.measure.get(), calls.arrange.get()), before);

    // Three children change in the same frame: one arrange.
    for kid in kids {
        cache.get_mut(kid).unwrap().set_size(size(40, 5));
    }

    frame(&mut cache, &mut backend);

    assert_eq!(calls.arrange.get(), before.1 + 1);
    assert_eq!(cache.get(kids[2]).unwrap().get_origin(), point(0, 10));
}

#[test]
fn layout_changed_is_emitted_for_the_arranged_container() {
    let calls = Calls::default();
    let (mut cache, _) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));
    let events = frame(&mut cache, &mut backend);

    assert!(events.contains(&EngineEvent::LayoutChanged { container: 0 }));
    assert!(frame(&mut cache, &mut backend).is_empty());
}

#[test]
fn moving_without_resizing_does_not_repaint() {
    let calls = Calls::default();
    let (mut cache, kids) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    frame(&mut cache, &mut backend);

    // Hiding the first child moves the other two up; their sizes stay the same.
    cache.get_mut(kids[0]).unwrap().set_hidden(true);
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(kids[1]).unwrap().get_origin(), point(0, 0));
    assert_eq!(cache.get(kids[2]).unwrap().get_origin(), point(0, 20));
    assert!(!repainted(&backend).contains(&kids[1]));
    assert!(!repainted(&backend).contains(&kids[2]));
    // The moved widgets are still blitted, at their new places.
    assert!(
        backend
            .draws
            .iter()
            .any(|(id, dest)| *id == kids[1] && dest.y == 0)
    );
}

#[test]
fn resizing_repaints_only_the_resized_widget() {
    let calls = Calls::default();
    let (mut cache, kids) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    frame(&mut cache, &mut backend);
    cache
        .get_mut(kids[1])
        .unwrap()
        .set_size_hint(SizeHint::fixed(size(40, 50)));
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(kids[1]).unwrap().get_size(), size(40, 50));
    assert_eq!(cache.get(kids[2]).unwrap().get_origin(), point(0, 60));
    assert!(repainted(&backend).contains(&kids[1]));
    assert!(!repainted(&backend).contains(&kids[2]));
}

#[test]
fn stacked_children_follow_their_margins_and_params() {
    let calls = Calls::default();
    let (mut cache, kids) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    cache
        .get_mut(kids[0])
        .unwrap()
        .set_layout_params(LayoutParams {
            margin: Margin::all(4),
            ..LayoutParams::default()
        });
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(kids[0]).unwrap().get_origin(), point(4, 4));
    assert_eq!(cache.get(kids[1]).unwrap().get_origin(), point(0, 18));
}

/// A root with a stack, holding an inner container (given a layout) that holds a leaf.
fn nested(inner_layout: Box<dyn Layout>) -> (WidgetCache, WidgetId, WidgetId) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));

    let inner = add(&mut cache, 0, 60, 60);
    let leaf = add(&mut cache, inner, 10, 10);

    cache
        .get_mut(0)
        .unwrap()
        .set_layout(Box::new(Stack::plain()));
    cache.get_mut(inner).unwrap().set_layout(inner_layout);

    (cache, inner, leaf)
}

#[test]
fn invalidation_stops_at_a_container_whose_hint_does_not_change() {
    let (mut cache, inner, leaf) = nested(Box::new(Fixed));
    let mut backend = HeadlessBackend::new(size(200, 200));

    frame(&mut cache, &mut backend);
    cache.get_mut(leaf).unwrap().set_size(size(20, 20));

    let events = frame(&mut cache, &mut backend);

    assert!(events.contains(&EngineEvent::LayoutChanged { container: inner }));
    assert!(!events.contains(&EngineEvent::LayoutChanged { container: 0 }));
}

#[test]
fn invalidation_reaches_the_parent_when_the_hint_changes() {
    let (mut cache, inner, leaf) = nested(Box::new(Stack::plain()));
    let mut backend = HeadlessBackend::new(size(200, 200));

    frame(&mut cache, &mut backend);
    cache.get_mut(leaf).unwrap().set_size(size(20, 30));

    let events = frame(&mut cache, &mut backend);

    assert!(events.contains(&EngineEvent::LayoutChanged { container: inner }));
    assert!(events.contains(&EngineEvent::LayoutChanged { container: 0 }));
    // The stack measured the inner container at 20x30, so the parent placed it that size.
    assert_eq!(cache.get(inner).unwrap().get_size(), size(20, 30));
}

#[test]
fn a_resized_container_arranges_its_children_in_the_same_frame() {
    let (mut cache, inner, leaf) = nested(Box::new(Stack::plain()));
    let mut backend = HeadlessBackend::new(size(200, 200));

    frame(&mut cache, &mut backend);
    cache.get_mut(leaf).unwrap().set_size(size(20, 30));
    cache.get_mut(inner).unwrap().set_origin(point(5, 5));
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(leaf).unwrap().get_size(), size(20, 30));
    assert_eq!(cache.get(leaf).unwrap().get_origin(), point(0, 0));
}

#[test]
fn a_new_child_and_a_removed_layout_are_handled() {
    let calls = Calls::default();
    let (mut cache, _) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    frame(&mut cache, &mut backend);

    let added = add(&mut cache, 0, 40, 40);

    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(added).unwrap().get_origin(), point(0, 60));

    cache.get_mut(0).unwrap().remove_layout();
    cache.get_mut(added).unwrap().set_origin(point(3, 3));
    frame(&mut cache, &mut backend);

    assert!(!cache.get(0).unwrap().has_layout());
    assert_eq!(cache.get(added).unwrap().get_origin(), point(3, 3));
}

#[test]
fn invalidate_layout_asks_for_a_pass() {
    let calls = Calls::default();
    let (mut cache, kids) = stacked(&calls);
    let mut backend = HeadlessBackend::new(size(100, 200));

    frame(&mut cache, &mut backend);
    cache.invalidate_layout(kids[0]);

    assert!(cache.needs_draw());
    cache.invalidate_layout(999);
}
