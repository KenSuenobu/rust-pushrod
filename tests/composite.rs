// Composite widgets: parts are ordinary child widgets that applications can find, change, add,
// and remove, and whose events reach the composite first.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::event::{EngineEvent, Event, EventCtx, Propagation};
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::paint::{self, Painter};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};

/// A clickable square used as a part.
struct Pad {
    core: WidgetCore,
}

impl Pad {
    fn new() -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(5, 5));

        core.add_controller(ClickController::new());
        Self { core }
    }
}

impl Widget for Pad {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }

    fn on_controller_event(&mut self, event: &ControllerEvent, ctx: &mut EventCtx) -> Propagation {
        if let ControllerEvent::Clicked { button, count } = event {
            ctx.emit(Event::Clicked {
                id: self.get_id(),
                button: *button,
                count: *count,
            });
        }

        Propagation::Continue
    }
}

/// A label above a row of items, 20 pixels wide each.  A click on an item becomes the bar's
/// own `ValueChanged`.
struct Bar {
    core: WidgetCore,
    parts: PartList,
    items: usize,
    label: WidgetId,
    /// How far right the label is placed.
    shift: i32,
}

/// The bar's structure.
#[derive(Debug, PartialEq)]
struct BarParts {
    label: WidgetId,
    items: Vec<WidgetId>,
}

impl Bar {
    fn new(items: usize) -> Self {
        Self {
            core: WidgetCore::new(point(10, 10), size(100, 20)),
            parts: PartList::new(),
            items,
            label: 0,
            shift: 0,
        }
    }
}

impl Widget for Bar {
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

impl Composite for Bar {
    fn build(&mut self, parts: &mut PartsBuilder) {
        self.label = parts.add("label", Pad::new());

        for index in 0..self.items {
            parts.add(format!("item:{index}"), Pad::new());
        }
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let mut placed = vec![(self.label, Rect::new(self.shift, 0, 100, 10))];

        for (index, id) in self.parts.list("item").into_iter().enumerate() {
            placed.push((id, Rect::new(index as i32 * 20, 10, 20, 10)));
        }

        placed
    }

    fn handle_part_event(&mut self, part: WidgetId, event: &Event, ctx: &mut EventCtx) {
        if matches!(event, Event::Clicked { .. }) && self.parts.list("item").contains(&part) {
            ctx.emit(Event::ValueChanged { id: self.get_id() });
        }
    }
}

impl TypedParts for Bar {
    type Parts = BarParts;

    fn typed_parts(&self) -> BarParts {
        BarParts {
            label: self.label,
            items: self.parts.list("item"),
        }
    }
}

/// A composite whose only part is a `Bar`; the bar's `ValueChanged` becomes its `Activated`.
struct Outer {
    core: WidgetCore,
    parts: PartList,
}

impl Widget for Outer {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Outer {
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("bar", Bar::new(1));
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    fn handle_part_event(&mut self, _part: WidgetId, event: &Event, ctx: &mut EventCtx) {
        if matches!(event, Event::ValueChanged { .. }) {
            ctx.emit(Event::Activated { id: self.get_id() });
        }
    }
}

/// A composite that answers its part's click with another click from the same part, which
/// must be delivered once and not handed back to it.
struct Echo {
    core: WidgetCore,
    parts: PartList,
}

impl Widget for Echo {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Echo {
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("pad", Pad::new());
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    fn handle_part_event(&mut self, part: WidgetId, event: &Event, ctx: &mut EventCtx) {
        if let Event::Clicked { button, count, .. } = event {
            ctx.emit(Event::Clicked {
                id: part,
                button: *button,
                count: *count,
            });
        }
    }
}

fn cache() -> WidgetCache {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 100))));
    cache
}

fn engine_events(cache: &mut WidgetCache) -> Vec<EngineEvent> {
    cache
        .take_pending_events()
        .into_iter()
        .filter_map(|event| match event {
            Event::Engine(engine) => Some(engine),
            _ => None,
        })
        .collect()
}

fn click_at(x: i32, y: i32) -> [InputEvent; 3] {
    [
        InputEvent::MouseMove {
            position: point(x, y),
        },
        InputEvent::MouseDown {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
        InputEvent::MouseUp {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
    ]
}

#[test]
fn parts_are_ordinary_child_widgets_with_their_own_ids() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(2)));

    assert_eq!(bar, 1);
    assert_eq!(cache.children_of(bar), &[2, 3, 4]);
    assert_eq!(
        cache.parts(bar),
        vec![
            ("label".to_string(), 2),
            ("item:0".to_string(), 3),
            ("item:1".to_string(), 4)
        ]
    );
    assert_eq!(cache.part(bar, "item:1"), Some(4));
    assert_eq!(cache.part(bar, "nothing"), None);
    assert_eq!(cache.get(4).unwrap().get_parent(), bar);
    assert_eq!(
        cache.get_as::<Bar>(bar).unwrap().typed_parts(),
        BarParts {
            label: 2,
            items: vec![3, 4]
        }
    );

    let added: Vec<_> = engine_events(&mut cache)
        .into_iter()
        .filter_map(|event| match event {
            EngineEvent::WidgetAdded { id, parent } => Some((id, parent)),
            _ => None,
        })
        .collect();

    assert_eq!(added, vec![(1, 0), (2, 1), (3, 1), (4, 1)]);
}

#[test]
fn the_layout_places_parts_after_the_build() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(2)));

    assert_eq!(cache.get(2).unwrap().get_size(), size(100, 10));
    assert_eq!(cache.get(4).unwrap().get_origin(), point(20, 10));
    assert!(engine_events(&mut cache).contains(&EngineEvent::PartsArranged { id: bar, placed: 3 }));
}

#[test]
fn list_parts_are_added_inserted_and_removed_with_relayout() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(1)));

    cache.take_pending_events();

    let appended = cache
        .insert_list_part(bar, "item", usize::MAX, Box::new(Pad::new()))
        .unwrap();
    let first = cache
        .insert_list_part(bar, "item", 0, Box::new(Pad::new()))
        .unwrap();

    assert_eq!(cache.part(bar, "item:0"), Some(first));
    assert_eq!(cache.part(bar, "item:1"), Some(3));
    assert_eq!(cache.part(bar, "item:2"), Some(appended));
    assert_eq!(cache.get(first).unwrap().get_origin(), point(0, 10));
    assert_eq!(cache.get(3).unwrap().get_origin(), point(20, 10));

    let removed = cache.remove_list_part(bar, "item", 0).unwrap();

    assert_eq!(removed, first);
    assert!(cache.get(first).is_none());
    assert!(!cache.children_of(bar).contains(&first));
    assert_eq!(cache.part(bar, "item:0"), Some(3));
    assert_eq!(cache.get(3).unwrap().get_origin(), point(0, 10));

    let events = engine_events(&mut cache);

    assert!(events.contains(&EngineEvent::WidgetAdded {
        id: first,
        parent: bar
    }));
    assert!(events.contains(&EngineEvent::WidgetRemoved {
        id: first,
        parent: bar
    }));
}

#[test]
fn named_parts_are_added_and_removed() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(0)));
    let extra = cache.add_part(bar, "extra", Box::new(Pad::new())).unwrap();

    assert_eq!(cache.part(bar, "extra"), Some(extra));
    assert_eq!(cache.remove_part(bar, "extra"), Ok(extra));
    assert_eq!(cache.part(bar, "extra"), None);

    let mut backend = HeadlessBackend::new(size(200, 100));

    assert!(cache.draw_loop(&mut backend));
    assert!(backend.draws.iter().all(|(id, _)| *id != extra));
    assert!(backend.draws.iter().any(|(id, _)| *id == bar));
}

#[test]
fn a_removed_part_takes_its_children_focus_and_hit_testing_with_it() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(1)));
    let item = cache.part(bar, "item:0").unwrap();
    let inner = cache.add_child(item, Box::new(Pad::new()));

    cache
        .get_mut(item)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE);
    assert!(cache.set_focus(item));
    assert_eq!(cache.widget_at(11, 21), inner);
    cache.take_pending_events();

    cache.remove_part(bar, "item:0").unwrap();

    assert!(cache.get(inner).is_none());
    assert!(cache.get_as::<Pad>(item).is_none());
    assert_eq!(cache.focused(), None);
    assert_eq!(cache.widget_at(11, 21), bar);

    let events = engine_events(&mut cache);

    assert!(events.contains(&EngineEvent::FocusChanged {
        old: Some(item),
        new: None
    }));
    assert!(events.contains(&EngineEvent::WidgetRemoved {
        id: inner,
        parent: item
    }));
    assert!(cache.try_add_child(item, Box::new(Pad::new())).is_err());
}

#[test]
fn part_changes_report_clear_errors() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(1)));

    assert_eq!(
        cache.add_part(bar, "label", Box::new(Pad::new())),
        Err(Error::composite(bar, "already has a part named \"label\""))
    );
    assert_eq!(
        cache.remove_part(bar, "missing"),
        Err(Error::composite(bar, "has no part named \"missing\""))
    );
    assert_eq!(
        cache.add_part(0, "x", Box::new(Pad::new())),
        Err(Error::composite(0, "is not a composite widget"))
    );
    assert_eq!(
        cache.arrange_parts(99),
        Err(Error::UnknownWidget { id: 99 })
    );
    assert!(cache.remove_list_part(bar, "item", 5).is_err());
    assert!(cache.parts(0).is_empty());
}

#[test]
fn nested_composites_are_built_in_turn() {
    let mut cache = cache();
    let outer = cache.add(Box::new(Outer {
        core: WidgetCore::new(point(0, 0), size(200, 100)),
        parts: PartList::new(),
    }));
    let bar = cache.part(outer, "bar").unwrap();

    assert_eq!(bar, outer + 1);
    assert_eq!(cache.parts(bar).len(), 2);
    assert_eq!(cache.get(bar + 1).unwrap().get_parent(), bar);
}

/// Which callbacks ran, in order.
type Log = Vec<&'static str>;

#[test]
fn part_events_reach_the_composite_and_callbacks_on_the_part_still_run() {
    let mut engine: Engine<Log> = Engine::new(size(200, 100), 30);
    let bar = engine.add_widget(Bar::new(1));
    let item = engine.widgets().part(bar, "item:0").unwrap();
    let label = engine.widgets().part(bar, "label").unwrap();
    let mut log = Log::new();

    engine.on_click(item, |log, _ui| log.push("item clicked"));
    engine.on_value_change(bar, |log, _ui| log.push("bar value changed"));
    engine.on_click(label, |log, _ui| log.push("label clicked"));

    // The item is at (10, 20) in the window.
    for event in click_at(12, 22) {
        engine.dispatch(event, &mut log);
    }

    assert_eq!(log, vec!["item clicked", "bar value changed"]);

    // The label's click is not translated.
    log.clear();

    for event in click_at(12, 12) {
        engine.dispatch(event, &mut log);
    }

    assert_eq!(log, vec!["label clicked"]);
}

#[test]
fn translated_events_climb_through_nested_composites() {
    let mut engine: Engine<Log> = Engine::new(size(200, 100), 30);
    let outer = engine.add_widget(Outer {
        core: WidgetCore::new(point(0, 0), size(200, 100)),
        parts: PartList::new(),
    });
    let bar = engine.widgets().part(outer, "bar").unwrap();
    let mut log = Log::new();

    engine.on_value_change(bar, |log, _ui| log.push("bar value changed"));
    engine.on_activate(outer, |log, _ui| log.push("outer activated"));

    for event in click_at(12, 22) {
        engine.dispatch(event, &mut log);
    }

    assert_eq!(log, vec!["bar value changed", "outer activated"]);
}

#[test]
fn a_composite_is_not_handed_its_own_translations_again() {
    let mut engine: Engine<u32> = Engine::new(size(200, 100), 30);
    let echo = engine.add_widget(Echo {
        core: WidgetCore::new(point(0, 0), size(50, 50)),
        parts: PartList::new(),
    });
    let pad = engine.widgets().part(echo, "pad").unwrap();
    let mut clicks = 0;

    engine.on_click(pad, |clicks, _ui| *clicks += 1);

    for event in click_at(2, 2) {
        engine.dispatch(event, &mut clicks);
    }

    // The pad's click, and the one echo emitted for it; not an endless loop.
    assert_eq!(clicks, 2);
}

#[test]
fn engine_callbacks_see_parts_removed() {
    let mut engine: Engine<Vec<WidgetId>> = Engine::new(size(200, 100), 30);
    let bar = engine.add_widget(Bar::new(2));
    let mut removed = Vec::new();

    engine.on_engine_event(|removed, _ui, event| {
        if let EngineEvent::WidgetRemoved { id, .. } = event {
            removed.push(*id);
        }
    });
    engine.widgets().remove_list_part(bar, "item", 1).unwrap();
    engine.run_idle(&mut removed);

    assert_eq!(removed, vec![4]);
}

#[test]
fn a_requested_arrange_runs_once_before_the_next_frame() {
    let mut cache = cache();
    let bar = cache.add(Box::new(Bar::new(1)));
    let mut backend = HeadlessBackend::new(size(200, 100));

    cache.draw_loop(&mut backend);
    let _ = engine_events(&mut cache);
    assert!(!cache.needs_draw());

    // The composite changes its own state and asks for its parts to follow.
    let widget = cache.get_as_mut::<Bar>(bar).unwrap();
    widget.shift = 30;
    widget.request_arrange_parts();
    widget.request_arrange_parts();

    assert!(cache.needs_draw());
    assert_eq!(cache.get(2).unwrap().get_origin(), point(0, 0));

    cache.draw_loop(&mut backend);

    assert_eq!(cache.get(2).unwrap().get_origin(), point(30, 0));
    assert_eq!(
        engine_events(&mut cache)
            .into_iter()
            .filter(|event| matches!(event, EngineEvent::PartsArranged { .. }))
            .count(),
        1
    );
    assert!(!cache.needs_draw());
}

#[test]
fn an_arrange_request_on_a_plain_widget_is_dropped() {
    let mut cache = cache();
    let pad = cache.add(Box::new(Pad::new()));
    let mut backend = HeadlessBackend::new(size(200, 100));

    cache.draw_loop(&mut backend);
    cache.get_mut(pad).unwrap().request_arrange_parts();
    assert!(cache.needs_draw());

    cache.draw_loop(&mut backend);

    assert!(!cache.needs_draw());
    assert!(
        !engine_events(&mut cache)
            .iter()
            .any(|event| matches!(event, EngineEvent::PartsArranged { .. }))
    );
}
