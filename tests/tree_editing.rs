// Editing the widget tree at run time: removing a subtree, moving a widget to a new parent,
// and changing the stacking order among siblings, without changing any other widget's ID.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::composite::{Composite, PartList, PartsBuilder};
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::event::{EngineEvent, Event, EventCtx, Propagation};
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::paint::{self, Painter};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};

/// A clickable square.
struct Pad {
    core: WidgetCore,
}

impl Pad {
    /// A `w` x `h` pad at `(x, y)`, focusable and clickable.
    fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        let mut core = WidgetCore::new(point(x, y), size(w, h));

        core.add_controller(ClickController::new());

        let mut pad = Self { core };

        pad.set_behavior(Behavior::FOCUSABLE);
        pad
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

/// A composite with a single part named `"label"`.
struct Labelled {
    core: WidgetCore,
    parts: PartList,
}

impl Widget for Labelled {
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

impl Composite for Labelled {
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("label", Pad::new(0, 0, 10, 10));
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }
}

/// The tree used by most tests:
///
/// ```text
/// 0 root (200x200)
/// ├── 1 panel  (0,0 100x100)
/// │    ├── 2 inner (10,10 50x50)
/// │    │    └── 3 leaf (5,5 10x10)
/// │    └── 4 side  (70,10 20x20)
/// └── 5 other  (100,0 100x100)
/// ```
struct Tree {
    cache: WidgetCache,
    panel: WidgetId,
    inner: WidgetId,
    leaf: WidgetId,
    side: WidgetId,
    other: WidgetId,
}

impl Tree {
    fn new() -> Self {
        let mut cache = WidgetCache::new();

        cache.add(Box::new(RootWidget::new(size(200, 200))));

        let panel = cache.add(Box::new(Pad::new(0, 0, 100, 100)));
        let inner = cache.add_child(panel, Box::new(Pad::new(10, 10, 50, 50)));
        let leaf = cache.add_child(inner, Box::new(Pad::new(5, 5, 10, 10)));
        let side = cache.add_child(panel, Box::new(Pad::new(70, 10, 20, 20)));
        let other = cache.add(Box::new(Pad::new(100, 0, 100, 100)));

        cache.take_pending_events();

        Self {
            cache,
            panel,
            inner,
            leaf,
            side,
            other,
        }
    }
}

/// Events do not implement `PartialEq`, so tests compare their `Debug` text.
fn text(events: impl IntoIterator<Item = Event>) -> Vec<String> {
    events.into_iter().map(|e| format!("{e:?}")).collect()
}

fn mouse_down(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn mouse_up(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseUp {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

// ---- remove ----

#[test]
fn removing_a_subtree_keeps_every_other_id() {
    let mut t = Tree::new();
    let len = t.cache.len();

    let removed = t.cache.remove(t.inner).unwrap();

    assert_eq!(removed, vec![t.inner, t.leaf]);
    assert!(t.cache.get(t.inner).is_none());
    assert!(t.cache.get(t.leaf).is_none());
    assert_eq!(t.cache.len(), len, "the removed slots stay as tombstones");
    assert_eq!(t.cache.children_of(t.panel), &[t.side]);
    assert!(t.cache.children_of(t.inner).is_empty());

    for id in [0, t.panel, t.side, t.other] {
        assert_eq!(t.cache.get(id).unwrap().get_id(), id);
    }

    // A new widget gets the next ID; removed IDs are never reused.
    let added = t.cache.add(Box::new(Pad::new(0, 0, 5, 5)));

    assert_eq!(added as usize, len);
}

#[test]
fn removing_emits_widget_removed_and_removed_parents_first() {
    let mut t = Tree::new();

    t.cache.remove(t.inner).unwrap();

    assert_eq!(
        text(t.cache.take_pending_events()),
        text([
            Event::Engine(EngineEvent::WidgetRemoved {
                id: t.inner,
                parent: t.panel,
            }),
            Event::Removed { id: t.inner },
            Event::Engine(EngineEvent::WidgetRemoved {
                id: t.leaf,
                parent: t.inner,
            }),
            Event::Removed { id: t.leaf },
        ])
    );
}

#[test]
fn removing_twice_or_removing_the_root_is_an_error() {
    let mut t = Tree::new();

    t.cache.remove(t.side).unwrap();

    assert_eq!(
        t.cache.remove(t.side),
        Err(Error::UnknownWidget { id: t.side })
    );
    assert_eq!(t.cache.remove(99), Err(Error::UnknownWidget { id: 99 }));
    assert!(matches!(t.cache.remove(0), Err(Error::Tree { id: 0, .. })));
    assert_eq!(
        t.cache
            .try_add_child(t.side, Box::new(Pad::new(0, 0, 1, 1)))
            .err(),
        Some(Error::UnknownWidget { id: t.side })
    );
}

#[test]
fn removing_clears_focus_hover_and_capture_in_the_subtree() {
    let mut t = Tree::new();

    // Press on the leaf (window 15..25): it is hovered, captures the mouse, and takes focus.
    t.cache.handle_event(mouse_down(20, 20));
    assert_eq!(t.cache.get_current_widget(), t.leaf);
    assert_eq!(t.cache.focused(), Some(t.leaf));
    t.cache.take_pending_events();

    t.cache.remove(t.inner).unwrap();

    assert_eq!(t.cache.get_current_widget(), 0);
    assert_eq!(t.cache.focused(), None);
    assert!(text(t.cache.take_pending_events()).contains(&format!(
        "{:?}",
        Event::Engine(EngineEvent::FocusChanged {
            old: Some(t.leaf),
            new: None,
        })
    )));

    // The release no longer goes to the removed leaf: the capture was cleared.
    let released = t.cache.handle_event(mouse_up(20, 20));

    assert!(
        released
            .iter()
            .all(|event| event.widget_id() != Some(t.leaf))
    );
}

#[test]
fn removing_outside_the_focused_widget_keeps_the_focus() {
    let mut t = Tree::new();

    assert!(t.cache.set_focus(t.other));
    t.cache.remove(t.inner).unwrap();

    assert_eq!(t.cache.focused(), Some(t.other));
}

#[test]
fn removed_textures_are_destroyed_and_never_drawn_again() {
    let mut t = Tree::new();
    let mut backend = HeadlessBackend::new(size(200, 200));

    t.cache.draw_loop(&mut backend);
    assert!(backend.texture_sizes.contains_key(&t.leaf));

    t.cache.remove(t.inner).unwrap();

    let drawn_before = backend.draws.len();

    assert!(t.cache.draw_loop(&mut backend));
    assert_eq!(backend.released, vec![t.inner, t.leaf]);
    assert!(!backend.texture_sizes.contains_key(&t.inner));
    assert!(!backend.texture_sizes.contains_key(&t.leaf));
    assert!(
        backend.draws[drawn_before..]
            .iter()
            .all(|(id, _)| *id != t.inner && *id != t.leaf)
    );

    // Releasing happens once.
    t.cache.draw_loop(&mut backend);
    assert_eq!(backend.released.len(), 2);
}

#[test]
fn removing_a_composite_part_takes_it_out_of_the_composite() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));

    let labelled = cache.add(Box::new(Labelled {
        core: WidgetCore::new(point(0, 0), size(50, 20)),
        parts: PartList::new(),
    }));
    let label = cache.part(labelled, "label").unwrap();

    assert_eq!(cache.remove(label).unwrap(), vec![label]);
    assert_eq!(cache.part(labelled, "label"), None);
    assert!(cache.get(label).is_none());
}

// ---- reparent ----

#[test]
fn reparenting_moves_the_subtree_and_keeps_ids() {
    let mut t = Tree::new();

    t.cache.reparent(t.inner, t.other).unwrap();

    assert_eq!(t.cache.children_of(t.panel), &[t.side]);
    assert_eq!(t.cache.children_of(t.other), &[t.inner]);
    assert_eq!(t.cache.children_of(t.inner), &[t.leaf]);
    assert_eq!(t.cache.get(t.inner).unwrap().get_parent(), t.other);
    assert_eq!(t.cache.get(t.leaf).unwrap().get_parent(), t.inner);

    // The origin is kept and is now relative to the new parent at x = 100.
    assert_eq!(t.cache.absolute_origin(t.inner), point(110, 10));
    assert_eq!(t.cache.absolute_origin(t.leaf), point(115, 15));
    assert_eq!(t.cache.widget_at(120, 20), t.leaf);

    assert_eq!(
        text(t.cache.take_pending_events()),
        text([Event::Engine(EngineEvent::WidgetReparented {
            id: t.inner,
            old_parent: t.panel,
            new_parent: t.other,
        })])
    );
}

#[test]
fn a_reparented_widget_becomes_the_top_most_child() {
    let mut t = Tree::new();

    t.cache.reparent(t.leaf, t.panel).unwrap();

    assert_eq!(t.cache.children_of(t.panel), &[t.inner, t.side, t.leaf]);
    assert_eq!(t.cache.stacking_index(t.leaf), Some(2));
}

#[test]
fn reparenting_to_the_same_parent_changes_nothing() {
    let mut t = Tree::new();

    t.cache.reparent(t.inner, t.panel).unwrap();

    assert_eq!(t.cache.children_of(t.panel), &[t.inner, t.side]);
    assert!(t.cache.take_pending_events().is_empty());
}

#[test]
fn reparenting_into_its_own_subtree_or_moving_the_root_is_an_error() {
    let mut t = Tree::new();

    assert!(matches!(
        t.cache.reparent(t.panel, t.leaf),
        Err(Error::Tree { .. })
    ));
    assert!(matches!(
        t.cache.reparent(t.panel, t.panel),
        Err(Error::Tree { .. })
    ));
    assert!(matches!(
        t.cache.reparent(0, t.panel),
        Err(Error::Tree { id: 0, .. })
    ));
    assert_eq!(
        t.cache.reparent(t.inner, 99),
        Err(Error::UnknownWidget { id: 99 })
    );
    assert_eq!(
        t.cache.reparent(99, t.panel),
        Err(Error::UnknownWidget { id: 99 })
    );

    t.cache.remove(t.side).unwrap();
    assert_eq!(
        t.cache.reparent(t.inner, t.side),
        Err(Error::UnknownWidget { id: t.side })
    );

    // Nothing moved.
    assert_eq!(t.cache.children_of(t.panel), &[t.inner]);
    assert_eq!(t.cache.children_of(t.inner), &[t.leaf]);
}

#[test]
fn a_composite_part_cannot_be_reparented() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));

    let labelled = cache.add(Box::new(Labelled {
        core: WidgetCore::new(point(0, 0), size(50, 20)),
        parts: PartList::new(),
    }));
    let label = cache.part(labelled, "label").unwrap();

    assert!(matches!(
        cache.reparent(label, 0),
        Err(Error::Tree { id, .. }) if id == label
    ));
    assert_eq!(cache.get(label).unwrap().get_parent(), labelled);
}

// ---- reorder ----

#[test]
fn restacking_changes_drawing_order_and_hit_testing() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));

    let back = cache.add(Box::new(Pad::new(0, 0, 50, 50)));
    let front = cache.add(Box::new(Pad::new(0, 0, 50, 50)));

    cache.take_pending_events();
    assert_eq!(cache.widget_at(10, 10), front);

    assert_eq!(cache.set_stacking_index(back, 1), Ok(1));

    assert_eq!(cache.children_of(0), &[front, back]);
    assert_eq!(cache.widget_at(10, 10), back);
    assert_eq!(cache.stacking_index(back), Some(1));
    assert_eq!(cache.stacking_index(front), Some(0));
    assert_eq!(
        text(cache.take_pending_events()),
        text([Event::Engine(EngineEvent::WidgetRestacked {
            id: back,
            parent: 0,
            old_index: 0,
            new_index: 1,
        })])
    );

    let mut backend = HeadlessBackend::new(size(200, 200));

    cache.draw_loop(&mut backend);

    let order: Vec<WidgetId> = backend.draws.iter().map(|(id, _)| *id).collect();

    assert_eq!(order, vec![0, front, back]);
}

#[test]
fn restacking_clamps_the_index_and_is_quiet_when_nothing_moves() {
    let mut t = Tree::new();

    assert_eq!(t.cache.set_stacking_index(t.inner, 50), Ok(1));
    assert_eq!(t.cache.children_of(t.panel), &[t.side, t.inner]);
    t.cache.take_pending_events();

    assert_eq!(t.cache.set_stacking_index(t.inner, 1), Ok(1));
    assert_eq!(t.cache.set_stacking_index(t.leaf, 0), Ok(0));
    assert!(t.cache.take_pending_events().is_empty());
}

#[test]
fn restacking_the_root_or_a_removed_widget_is_an_error() {
    let mut t = Tree::new();

    assert!(matches!(
        t.cache.set_stacking_index(0, 0),
        Err(Error::Tree { id: 0, .. })
    ));

    t.cache.remove(t.side).unwrap();
    assert_eq!(
        t.cache.set_stacking_index(t.side, 0),
        Err(Error::UnknownWidget { id: t.side })
    );
    assert_eq!(t.cache.stacking_index(t.side), None);
}

// ---- engine ----

#[test]
fn a_click_callback_adds_and_removes_a_widget() {
    // The application state holds the ID of the widget the button toggles.
    let mut engine: Engine<(Option<WidgetId>, Vec<String>)> = Engine::new(size(200, 200), 30);
    let button = engine.add_widget(Pad::new(0, 0, 40, 20));

    engine.on_click(button, |(shown, _), ui| {
        *shown = match shown.take() {
            Some(id) => {
                ui.widgets().remove(id).unwrap();
                None
            }
            None => Some(ui.widgets().add(Box::new(Pad::new(50, 50, 30, 30)))),
        };
    });
    engine.on_engine_event(move |(_, log), _ui, event| match event {
        EngineEvent::WidgetAdded { id, .. } if *id != button => log.push(format!("added {id}")),
        EngineEvent::WidgetRemoved { id, .. } => log.push(format!("removed {id}")),
        _ => {}
    });

    let mut state = (None, Vec::new());

    for _ in 0..3 {
        engine.dispatch(mouse_down(5, 5), &mut state);
        engine.dispatch(mouse_up(5, 5), &mut state);
    }

    let first = button + 1;
    let second = button + 2;

    assert_eq!(state.0, Some(second), "a new widget never reuses an ID");
    assert_eq!(
        state.1,
        vec![
            format!("added {first}"),
            format!("removed {first}"),
            format!("added {second}"),
        ]
    );
    assert!(engine.widgets().get(first).is_none());
    assert!(engine.widgets().get(second).is_some());
}

#[test]
fn callbacks_on_a_removed_widget_are_dropped() {
    let mut engine: Engine<Vec<String>> = Engine::new(size(200, 200), 30);
    let pad = engine.add_widget(Pad::new(0, 0, 40, 20));

    engine.on_click(pad, |log, _ui| log.push("clicked".into()));
    engine.on_event(|log, _ui, event| {
        if let Event::Removed { id } = event {
            log.push(format!("removed {id}"));
        }
    });

    let mut log = Vec::new();

    engine.dispatch(mouse_down(5, 5), &mut log);
    engine.dispatch(mouse_up(5, 5), &mut log);
    assert_eq!(log, vec!["clicked".to_string()]);

    engine.widgets().remove(pad).unwrap();
    // Deliver the queued removal events.
    engine.dispatch(InputEvent::Wake, &mut log);

    engine.dispatch(mouse_down(5, 5), &mut log);
    engine.dispatch(mouse_up(5, 5), &mut log);

    assert_eq!(log, vec!["clicked".to_string(), format!("removed {pad}")]);
}
