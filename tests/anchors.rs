//! Anchors (autoresizing): children keep their margins or stretch when a container without a
//! layout is resized, nested containers follow in one frame, and layouts take precedence.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::event::{EngineEvent, Event};
use pushrod::geometry::{point, size};
use pushrod::layout::{Anchors, Column};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetId};

/// Adds a child with a place, a size, and anchors.
fn add(
    cache: &mut WidgetCache,
    parent: WidgetId,
    at: (i32, i32),
    wh: (u32, u32),
    anchors: Anchors,
) -> WidgetId {
    let mut widget = RootWidget::new(size(wh.0, wh.1));

    widget.set_origin(point(at.0, at.1));
    widget.set_anchors(anchors);

    cache.add_child(parent, Box::new(widget))
}

/// Draws one frame, returning the anchor events it queued.
fn frame(cache: &mut WidgetCache, backend: &mut HeadlessBackend) -> Vec<EngineEvent> {
    cache.draw_loop(backend);

    cache
        .take_pending_events()
        .into_iter()
        .filter_map(|e| match e {
            Event::Engine(e @ EngineEvent::AnchorsApplied { .. }) => Some(e),
            _ => None,
        })
        .collect()
}

/// A 200x100 root with a headless backend.
fn setup() -> (WidgetCache, HeadlessBackend) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 100))));

    (cache, HeadlessBackend::new(size(200, 100)))
}

#[test]
fn children_follow_their_anchors_on_resize() {
    let (mut cache, mut backend) = setup();
    let fixed = add(&mut cache, 0, (10, 10), (50, 20), Anchors::empty());
    let right = add(
        &mut cache,
        0,
        (140, 10),
        (50, 20),
        Anchors::RIGHT | Anchors::TOP,
    );
    let bar = add(
        &mut cache,
        0,
        (0, 80),
        (200, 20),
        Anchors::LEFT | Anchors::RIGHT | Anchors::BOTTOM,
    );
    let tall = add(&mut cache, 0, (60, 10), (30, 70), Anchors::STRETCH_HEIGHT);

    assert!(frame(&mut cache, &mut backend).is_empty());

    cache.get_mut(0).unwrap().set_size(size(300, 150));

    let events = frame(&mut cache, &mut backend);

    assert_eq!(
        events,
        vec![EngineEvent::AnchorsApplied {
            container: 0,
            children: 3
        }]
    );
    assert_eq!(cache.get(fixed).unwrap().get_origin(), point(10, 10));
    assert_eq!(cache.get(fixed).unwrap().get_size(), size(50, 20));
    assert_eq!(cache.get(right).unwrap().get_origin(), point(240, 10));
    assert_eq!(cache.get(bar).unwrap().get_origin(), point(0, 130));
    assert_eq!(cache.get(bar).unwrap().get_size(), size(300, 20));
    assert_eq!(cache.get(tall).unwrap().get_size(), size(30, 120));
}

#[test]
fn shrinking_and_growing_back_keeps_the_margins() {
    let (mut cache, mut backend) = setup();
    let bar = add(
        &mut cache,
        0,
        (10, 0),
        (180, 20),
        Anchors::LEFT | Anchors::RIGHT,
    );

    frame(&mut cache, &mut backend);
    cache.get_mut(0).unwrap().set_size(size(120, 100));
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(bar).unwrap().get_size(), size(100, 20));

    cache.get_mut(0).unwrap().set_size(size(200, 100));
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(bar).unwrap().get_size(), size(180, 20));
}

#[test]
fn a_frame_without_a_resize_changes_nothing() {
    let (mut cache, mut backend) = setup();
    let child = add(&mut cache, 0, (140, 10), (50, 20), Anchors::RIGHT);

    frame(&mut cache, &mut backend);

    assert!(frame(&mut cache, &mut backend).is_empty());
    assert_eq!(cache.get(child).unwrap().get_origin(), point(140, 10));
}

#[test]
fn nested_containers_follow_in_one_frame() {
    let (mut cache, mut backend) = setup();
    let pane = add(
        &mut cache,
        0,
        (0, 0),
        (200, 100),
        Anchors::LEFT | Anchors::RIGHT | Anchors::TOP | Anchors::BOTTOM,
    );
    let corner = add(
        &mut cache,
        pane,
        (150, 70),
        (50, 30),
        Anchors::RIGHT | Anchors::BOTTOM,
    );

    frame(&mut cache, &mut backend);
    cache.get_mut(0).unwrap().set_size(size(260, 140));
    frame(&mut cache, &mut backend);

    assert_eq!(cache.get(pane).unwrap().get_size(), size(260, 140));
    assert_eq!(cache.get(corner).unwrap().get_origin(), point(210, 110));
}

#[test]
fn a_container_with_a_layout_ignores_anchors() {
    let (mut cache, mut backend) = setup();
    let child = add(
        &mut cache,
        0,
        (0, 0),
        (50, 20),
        Anchors::RIGHT | Anchors::BOTTOM,
    );

    cache
        .get_mut(0)
        .unwrap()
        .set_layout(Box::new(Column::new()));
    frame(&mut cache, &mut backend);
    cache.get_mut(0).unwrap().set_size(size(300, 300));

    assert!(frame(&mut cache, &mut backend).is_empty());
    assert_eq!(cache.get(child).unwrap().get_origin(), point(0, 0));
}

#[test]
fn a_moved_child_is_not_repainted_but_a_resized_one_is() {
    let (mut cache, mut backend) = setup();
    let mover = add(&mut cache, 0, (140, 10), (50, 20), Anchors::RIGHT);
    let stretcher = add(&mut cache, 0, (0, 40), (50, 20), Anchors::STRETCH_WIDTH);

    frame(&mut cache, &mut backend);
    cache.get_mut(0).unwrap().set_size(size(300, 100));
    backend.painted.clear();
    frame(&mut cache, &mut backend);

    let painted: Vec<WidgetId> = backend.painted.iter().map(|(id, _)| *id).collect();

    assert!(painted.contains(&stretcher));
    assert!(!painted.contains(&mover));
}
