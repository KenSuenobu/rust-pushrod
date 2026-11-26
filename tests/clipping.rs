//! Children are clipped to their parent's bounds: cropped blits and cropped hit testing.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::geometry::{Rect, point, size};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetId};

/// Adds a `RootWidget` child at a position with a size.
fn add(cache: &mut WidgetCache, parent: WidgetId, x: i32, y: i32, w: u32, h: u32) -> WidgetId {
    let mut widget = RootWidget::new(size(w, h));

    widget.set_origin(point(x, y));
    cache.add_child(parent, Box::new(widget))
}

/// A cache with a 100x100 root.
fn cache() -> WidgetCache {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(100, 100))));
    cache
}

#[test]
fn a_child_inside_its_parent_is_not_cropped() {
    let mut cache = cache();
    let child = add(&mut cache, 0, 10, 10, 20, 20);

    assert_eq!(cache.visible_rect(child), Some(Rect::new(10, 10, 20, 20)));
}

#[test]
fn a_child_partly_outside_is_cropped_to_the_parent() {
    let mut cache = cache();
    let child = add(&mut cache, 0, 90, 80, 30, 30);

    assert_eq!(cache.visible_rect(child), Some(Rect::new(90, 80, 10, 20)));
}

#[test]
fn clipping_applies_through_every_ancestor() {
    let mut cache = cache();
    let panel = add(&mut cache, 0, 20, 20, 30, 30);
    let child = add(&mut cache, panel, 20, -5, 40, 40);

    // The panel covers 20..50; the child starts at 40 and is cut at 50, and at the panel's top.
    assert_eq!(cache.visible_rect(child), Some(Rect::new(40, 20, 10, 30)));
}

#[test]
fn a_child_wholly_outside_has_no_visible_rect() {
    let mut cache = cache();
    let child = add(&mut cache, 0, 150, 0, 10, 10);

    assert_eq!(cache.visible_rect(child), None);
    assert_eq!(cache.visible_rect(99), None);
}

#[test]
fn hit_testing_ignores_the_cropped_part() {
    let mut cache = cache();
    let panel = add(&mut cache, 0, 0, 0, 50, 50);
    let sibling = add(&mut cache, 0, 50, 0, 50, 50);
    let child = add(&mut cache, panel, 40, 10, 40, 20);

    assert_eq!(cache.widget_at(45, 15), child);
    // Over the sibling, where the child sticks out of the panel: the sibling wins.
    assert_eq!(cache.widget_at(60, 15), sibling);
}

#[test]
fn the_cropped_part_of_a_child_is_not_clickable_even_with_no_sibling() {
    let mut cache = cache();
    let panel = add(&mut cache, 0, 0, 0, 50, 50);
    let child = add(&mut cache, panel, 40, 10, 40, 20);

    assert_eq!(cache.widget_at(45, 15), child);
    assert_eq!(cache.widget_at(60, 15), 0);
}

#[test]
fn an_overlay_is_not_clipped_to_its_parent() {
    let mut cache = cache();
    let panel = add(&mut cache, 0, 0, 0, 50, 50);
    let popup = add(&mut cache, panel, 40, 10, 40, 20);

    cache
        .get_mut(popup)
        .unwrap()
        .set_behavior(Behavior::OVERLAY);

    assert_eq!(cache.visible_rect(popup), Some(Rect::new(40, 10, 40, 20)));
    assert_eq!(cache.widget_at(60, 15), popup);
}

#[test]
fn a_child_of_an_overlay_is_clipped_to_the_overlay_only() {
    let mut cache = cache();
    let panel = add(&mut cache, 0, 0, 0, 20, 20);
    let popup = add(&mut cache, panel, 10, 10, 60, 60);
    let item = add(&mut cache, popup, 0, 50, 30, 30);

    cache
        .get_mut(popup)
        .unwrap()
        .set_behavior(Behavior::OVERLAY);

    assert_eq!(cache.visible_rect(item), Some(Rect::new(10, 60, 30, 10)));
}

#[test]
fn blits_use_cropped_source_and_destination_rectangles() {
    let mut cache = cache();
    let child = add(&mut cache, 0, 90, 80, 30, 30);
    let mut backend = HeadlessBackend::new(size(100, 100));

    assert!(cache.draw_loop(&mut backend));

    let dest = backend.draws.iter().find(|(id, _)| *id == child).unwrap().1;
    let source = backend
        .draw_sources
        .iter()
        .find(|(id, _)| *id == child)
        .unwrap()
        .1;

    assert_eq!(dest, Rect::new(90, 80, 10, 20));
    assert_eq!(source, Rect::new(0, 0, 10, 20));
}

#[test]
fn a_child_scrolled_left_of_its_parent_crops_the_left_of_its_texture() {
    let mut cache = cache();
    let panel = add(&mut cache, 0, 20, 20, 40, 40);
    let child = add(&mut cache, panel, -10, 5, 30, 10);
    let mut backend = HeadlessBackend::new(size(100, 100));

    cache.draw_loop(&mut backend);

    let dest = backend.draws.iter().find(|(id, _)| *id == child).unwrap().1;
    let source = backend
        .draw_sources
        .iter()
        .find(|(id, _)| *id == child)
        .unwrap()
        .1;

    assert_eq!(dest, Rect::new(20, 25, 20, 10));
    assert_eq!(source, Rect::new(10, 0, 20, 10));
}

#[test]
fn a_fully_clipped_child_is_not_blitted() {
    let mut cache = cache();
    let child = add(&mut cache, 0, 150, 0, 10, 10);
    let mut backend = HeadlessBackend::new(size(100, 100));

    cache.draw_loop(&mut backend);

    assert!(backend.draws.iter().all(|(id, _)| *id != child));
}
