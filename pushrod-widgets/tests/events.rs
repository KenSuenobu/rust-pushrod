// Event dispatch tests.  These exercise the WidgetCache and Engine without opening a window.

use pushrod::cache::WidgetCache;
use pushrod::color::{Color, FontStyle};
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, EngineEventKind, Event, EventKind, EventMask, MaskScope};
use pushrod::geometry::{point, size};
use pushrod::widget::{Behavior, VisualState, Widget};
use pushrod_widgets::base_widget::BaseWidget;
use pushrod_widgets::button_widget::ButtonWidget;
use pushrod_widgets::text_widget::{TextJustify, TextWidget};
use sdl2::event::Event as SdlEvent;
use sdl2::mouse::{MouseButton, MouseState};

const FONT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/OpenSans-Regular.ttf"
);

fn button(x: i32, y: i32) -> ButtonWidget {
    ButtonWidget::new(
        point(x, y),
        size(100, 20),
        String::from(FONT),
        FontStyle::NORMAL,
        14,
        Color::BLACK,
        TextJustify::Center,
        String::from("Button"),
        2,
    )
}

fn down(x: i32, y: i32) -> SdlEvent {
    SdlEvent::MouseButtonDown {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x,
        y,
    }
}

fn up(x: i32, y: i32) -> SdlEvent {
    SdlEvent::MouseButtonUp {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x,
        y,
    }
}

fn motion(x: i32, y: i32) -> SdlEvent {
    SdlEvent::MouseMotion {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mousestate: MouseState::from_sdl_state(0),
        x,
        y,
        xrel: 0,
        yrel: 0,
    }
}

fn clicks(events: &[Event]) -> Vec<u32> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Clicked { id, .. } => Some(*id),
            _ => None,
        })
        .collect()
}

/// Root window with a panel at (20, 20) containing a button at (10, 10), so the button covers
/// window coordinates (30..130, 30..50).
fn cache_with_button() -> (WidgetCache, u32, u32) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(BaseWidget::new(point(0, 0), size(400, 300))));

    let panel = cache.add(Box::new(BaseWidget::new(point(20, 20), size(300, 200))));
    let button = cache.add_child(panel, Box::new(button(10, 10)));

    (cache, panel, button)
}

#[test]
fn ids_are_assigned_in_order_and_children_are_relative() {
    let (cache, panel, button) = cache_with_button();

    assert_eq!((panel, button), (1, 2));
    assert_eq!(cache.get(button).unwrap().get_id(), button);
    assert_eq!(cache.get(button).unwrap().get_parent(), panel);
    assert_eq!(cache.children_of(panel), &[button]);

    let origin = cache.absolute_origin(button);
    assert_eq!((origin.x, origin.y), (30, 30));
}

#[test]
fn hit_testing_finds_the_top_most_visible_widget() {
    let (mut cache, panel, button) = cache_with_button();

    assert_eq!(cache.widget_at(35, 35), button);
    assert_eq!(cache.widget_at(129, 49), button);
    assert_eq!(
        cache.widget_at(130, 50),
        panel,
        "right/bottom edges are exclusive"
    );
    assert_eq!(cache.widget_at(5, 5), 0);

    cache.get_mut(button).unwrap().set_hidden(true);
    assert_eq!(cache.widget_at(35, 35), panel, "hidden widgets are not hit");

    cache.get_mut(button).unwrap().set_hidden(false);
    cache.get_mut(panel).unwrap().set_hidden(true);
    assert_eq!(
        cache.widget_at(35, 35),
        0,
        "hiding a parent hides its children"
    );
}

#[test]
fn press_and_release_inside_emits_clicked() {
    let (mut cache, _, button) = cache_with_button();

    assert!(clicks(&cache.handle_event(motion(40, 40))).is_empty());
    assert!(clicks(&cache.handle_event(down(40, 40))).is_empty());
    assert!(cache.get_as::<ButtonWidget>(button).unwrap().is_selected());
    assert_eq!(clicks(&cache.handle_event(up(40, 40))), vec![button]);
    assert!(!cache.get_as::<ButtonWidget>(button).unwrap().is_selected());
}

#[test]
fn dragging_out_before_release_cancels_the_click() {
    let (mut cache, _, button) = cache_with_button();

    cache.handle_event(motion(40, 40));
    cache.handle_event(down(40, 40));
    cache.handle_event(motion(200, 150));
    assert!(!cache.get_as::<ButtonWidget>(button).unwrap().is_selected());
    assert!(clicks(&cache.handle_event(up(200, 150))).is_empty());
}

#[test]
fn dragging_back_in_before_release_still_clicks() {
    let (mut cache, _, button) = cache_with_button();

    cache.handle_event(motion(40, 40));
    cache.handle_event(down(40, 40));
    cache.handle_event(motion(200, 150));
    cache.handle_event(motion(45, 45));
    assert!(cache.get_as::<ButtonWidget>(button).unwrap().is_selected());
    assert_eq!(clicks(&cache.handle_event(up(45, 45))), vec![button]);
}

#[test]
fn releasing_over_another_button_clicks_neither() {
    let (mut cache, panel, first) = cache_with_button();
    let second = cache.add_child(panel, Box::new(button(10, 50)));

    cache.handle_event(motion(40, 40));
    cache.handle_event(down(40, 40));
    cache.handle_event(motion(40, 80));
    assert!(clicks(&cache.handle_event(up(40, 80))).is_empty());
    assert!(!cache.get_as::<ButtonWidget>(first).unwrap().is_selected());
    assert!(!cache.get_as::<ButtonWidget>(second).unwrap().is_selected());
}

#[test]
fn get_as_returns_none_for_the_wrong_type() {
    let (cache, panel, button) = cache_with_button();

    assert!(cache.get_as::<ButtonWidget>(panel).is_none());
    assert!(cache.get_as::<ButtonWidget>(button).is_some());
    assert!(cache.get_as::<ButtonWidget>(99).is_none());
}

#[derive(Default)]
struct State {
    clicks: u32,
    events: u32,
}

#[test]
fn engine_runs_click_callbacks_with_state_and_widget_access() {
    let mut engine: Engine<State> = Engine::new(size(400, 300), 30);
    let button_id = engine.add_widget(button(10, 10));
    let label_id = engine.add_widget(TextWidget::new(
        point(10, 50),
        size(200, 20),
        String::from(FONT),
        FontStyle::NORMAL,
        14,
        Color::BLACK,
        TextJustify::Left,
        String::new(),
    ));

    engine.on_click(button_id, move |state, ui| {
        state.clicks += 1;
        ui.get_mut::<TextWidget>(label_id)
            .unwrap()
            .set_text(format!("{}", state.clicks));
    });
    engine.on_event(|state, _ui, event| {
        if matches!(event, Event::Clicked { .. }) {
            state.events += 1;
        }
    });

    let mut state = State::default();

    for _ in 0..2 {
        engine.dispatch(motion(20, 20), &mut state);
        engine.dispatch(down(20, 20), &mut state);
        engine.dispatch(up(20, 20), &mut state);
    }

    assert_eq!(state.clicks, 2);
    assert_eq!(state.events, 2);
    assert_eq!(
        engine
            .widgets()
            .get_as::<TextWidget>(label_id)
            .unwrap()
            .get_text(),
        "2"
    );
}

#[test]
fn callbacks_can_shut_down_the_engine() {
    let mut engine: Engine<()> = Engine::new(size(400, 300), 30);
    let button_id = engine.add_widget(button(10, 10));

    engine.on_click(button_id, |_, ui| ui.shutdown());

    assert!(engine.is_running());
    engine.dispatch(down(20, 20), &mut ());
    engine.dispatch(up(20, 20), &mut ());
    assert!(!engine.is_running());
}

/// A widget that echoes every event it receives, so tests can see what the cache routed to it.
struct Recorder {
    core: pushrod::widget::WidgetCore,
}

impl pushrod::widget::Widget for Recorder {
    fn core(&self) -> &pushrod::widget::WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut pushrod::widget::WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn pushrod::paint::Painter) {}

    fn handle_event(&mut self, event: &Event, ctx: &mut pushrod::event::EventCtx) {
        ctx.emit(event.clone());
    }
}

fn recorder(x: i32, y: i32) -> Recorder {
    let mut core = pushrod::widget::WidgetCore::new(point(x, y), size(100, 20));

    core.set_event_mask(EventMask::ALL);

    Recorder { core }
}

#[test]
fn pointer_events_use_the_new_names_and_named_fields() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(BaseWidget::new(point(0, 0), size(400, 300))));

    let first = cache.add(Box::new(recorder(10, 10)));
    let second = cache.add(Box::new(recorder(10, 50)));

    // Pointer enters the first widget from the root (which echoes nothing): Entered, then Moved.
    let events = cache.handle_event(motion(20, 15));
    let kinds: Vec<_> = events.iter().map(Event::kind).collect();

    assert_eq!(
        kinds,
        vec![EventKind::PointerEntered, EventKind::PointerMoved]
    );
    assert_eq!(events[0].widget(), Some(first));

    // Pointer goes from the first widget to the second: Exited, Entered, then Moved.
    let events = cache.handle_event(motion(20, 55));
    let kinds: Vec<_> = events.iter().map(Event::kind).collect();

    assert_eq!(
        kinds,
        vec![
            EventKind::PointerExited,
            EventKind::PointerEntered,
            EventKind::PointerMoved
        ]
    );
    assert_eq!(events[0].widget(), Some(first));
    assert_eq!(events[1].widget(), Some(second));

    // Press and release carry the button, position, and click count.
    let pressed = cache.handle_event(down(20, 55));

    assert!(matches!(
        pressed.as_slice(),
        [Event::PointerPressed { id, button: MouseButton::Left, position, count: 1 }]
            if *id == second && position.x == 20 && position.y == 55
    ));

    let released = cache.handle_event(up(20, 55));

    assert!(matches!(
        released.as_slice(),
        [Event::PointerReleased { id, button: MouseButton::Left, count: 1, .. }] if *id == second
    ));
}

#[test]
fn clicked_carries_button_and_count() {
    let (mut cache, _, button) = cache_with_button();

    cache.handle_event(motion(40, 40));
    cache.handle_event(down(40, 40));

    let events = cache.handle_event(up(40, 40));

    assert!(events.iter().any(|event| matches!(
        event,
        Event::Clicked { id, button: MouseButton::Left, count: 1 } if *id == button
    )));
}

#[test]
fn new_widgets_have_no_behavior_flags_and_a_normal_state() {
    let (cache, _, button) = cache_with_button();
    let widget = cache.get(button).unwrap();

    assert_eq!(widget.behavior(), Behavior::empty());
    assert_eq!(widget.state(), VisualState::empty());
    assert!(widget.is_enabled());
    assert!(!widget.is_hidden());
}

#[test]
fn set_hidden_is_backed_by_the_hidden_behavior_flag() {
    let (mut cache, _, button) = cache_with_button();
    let widget = cache.get_mut(button).unwrap();

    widget.set_behavior(Behavior::FOCUSABLE);
    widget.set_hidden(true);
    assert_eq!(widget.behavior(), Behavior::FOCUSABLE | Behavior::HIDDEN);
    assert!(widget.is_hidden());

    widget.set_behavior(Behavior::FOCUSABLE);
    assert!(!widget.is_hidden(), "clearing the flag shows the widget");

    widget.set_behavior(Behavior::HIDDEN);
    assert!(widget.is_hidden(), "setting the flag hides the widget");
}

#[test]
fn set_state_invalidates_only_when_the_state_changes() {
    let (mut cache, _, button) = cache_with_button();
    let widget = cache.get_mut(button).unwrap();

    widget.set_invalidated(false);
    widget.set_state(VisualState::empty());
    assert!(!widget.is_invalidated());

    widget.set_state(VisualState::CHECKED | VisualState::HOVERED);
    assert!(widget.is_invalidated());
    assert_eq!(widget.state(), VisualState::CHECKED | VisualState::HOVERED);
}

#[test]
fn set_enabled_toggles_the_disabled_state_and_keeps_other_flags() {
    let (mut cache, _, button) = cache_with_button();
    let widget = cache.get_mut(button).unwrap();

    widget.set_state(VisualState::CHECKED);
    widget.set_enabled(false);
    assert!(!widget.is_enabled());
    assert_eq!(widget.state(), VisualState::CHECKED | VisualState::DISABLED);

    widget.set_enabled(true);
    assert!(widget.is_enabled());
    assert_eq!(widget.state(), VisualState::CHECKED);
}

#[test]
fn only_enabled_visible_focusable_widgets_can_take_focus() {
    let (mut cache, _, button) = cache_with_button();
    let widget = cache.get_mut(button).unwrap();

    assert!(!widget.is_focusable(), "not FOCUSABLE yet");

    widget.set_behavior(Behavior::FOCUSABLE);
    assert!(widget.is_focusable());

    widget.set_enabled(false);
    assert!(
        !widget.is_focusable(),
        "disabled widgets are skipped by focus"
    );

    widget.set_enabled(true);
    widget.set_hidden(true);
    assert!(
        !widget.is_focusable(),
        "hidden widgets are skipped by focus"
    );
}

#[test]
fn disabled_widgets_receive_no_input_and_block_widgets_beneath_them() {
    let (mut cache, panel, button) = cache_with_button();

    cache.get_mut(button).unwrap().set_enabled(false);

    assert_eq!(cache.widget_at(40, 40), button, "still hit-tested");
    assert!(clicks(&cache.handle_event(down(40, 40))).is_empty());
    assert!(clicks(&cache.handle_event(up(40, 40))).is_empty());
    assert!(!cache.get_as::<ButtonWidget>(button).unwrap().is_selected());
    assert!(cache.get(panel).is_some());

    cache.get_mut(button).unwrap().set_enabled(true);
    cache.handle_event(down(40, 40));
    assert_eq!(clicks(&cache.handle_event(up(40, 40))), vec![button]);
}

#[test]
fn widgets_with_an_empty_mask_receive_nothing_while_a_button_still_clicks() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(BaseWidget::new(point(0, 0), size(400, 300))));

    let mut quiet = recorder(10, 10);

    quiet.core_mut().set_event_mask(EventMask::NONE);

    let quiet_id = cache.add(Box::new(quiet));
    let button_id = cache.add(Box::new(button(10, 50)));

    // The quiet widget is hit but filtered: nothing is generated for it.
    assert!(cache.handle_event(down(20, 15)).is_empty());
    assert!(cache.handle_event(up(20, 15)).is_empty());
    assert_eq!(cache.get(quiet_id).unwrap().event_mask(), EventMask::NONE);

    // The button still clicks.
    cache.handle_event(down(20, 55));

    let kinds: Vec<_> = cache
        .handle_event(up(20, 55))
        .iter()
        .map(Event::kind)
        .collect();

    assert_eq!(kinds, vec![EventKind::Clicked, EventKind::PointerReleased]);
    assert!(cache.get(button_id).is_some());
}

#[test]
fn a_widget_only_receives_the_classes_in_its_mask() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(BaseWidget::new(point(0, 0), size(400, 300))));

    let mut hover_only = recorder(10, 10);

    hover_only.core_mut().set_event_mask(EventMask::HOVER);
    cache.add(Box::new(hover_only));

    let kinds: Vec<_> = cache
        .handle_event(motion(20, 15))
        .iter()
        .map(Event::kind)
        .collect();

    assert_eq!(kinds, vec![EventKind::PointerEntered]);
}

#[test]
fn the_run_loop_mask_filters_before_widgets_and_close_is_always_delivered() {
    let mut engine: Engine<Vec<EventKind>> = Engine::new(size(400, 300), 30);

    engine.add_widget(recorder(10, 10));
    engine.on_event(|seen: &mut Vec<EventKind>, _ui, event| seen.push(event.kind()));
    engine.set_event_mask(EventMask::HOVER);

    let mut seen = Vec::new();

    engine.dispatch(down(20, 15), &mut seen);
    engine.dispatch(motion(20, 15), &mut seen);

    // The widget-added and mask-change events, then hover only: PointerMoved and presses are filtered.
    assert_eq!(
        seen,
        vec![
            EventKind::Engine,
            EventKind::Engine,
            EventKind::PointerEntered
        ]
    );

    engine.set_event_mask(EventMask::NONE);
    engine.dispatch(SdlEvent::Quit { timestamp: 0 }, &mut seen);

    assert!(!engine.is_running());
}

#[test]
fn changing_masks_emits_event_mask_changed_once_per_change() {
    let mut engine: Engine<Vec<Event>> = Engine::new(size(400, 300), 30);
    let id = engine.add_widget(recorder(10, 10));

    engine.on_event(|seen: &mut Vec<Event>, _ui, event| seen.push(event.clone()));
    engine.set_event_mask(EventMask::KEYBOARD);
    engine.set_event_mask(EventMask::KEYBOARD);
    engine.widgets().set_event_mask(id, EventMask::TEXT);

    let mut seen = Vec::new();

    // Pending events are delivered with the next dispatched event.
    engine.dispatch(motion(500, 500), &mut seen);

    let changes: Vec<_> = seen
        .iter()
        .filter_map(|event| match event {
            Event::Engine(EngineEvent::EventMaskChanged { scope, old, new }) => {
                Some((*scope, *old, *new))
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        changes,
        vec![
            (MaskScope::RunLoop, EventMask::ALL, EventMask::KEYBOARD),
            (MaskScope::Widget(id), EventMask::ALL, EventMask::TEXT),
        ]
    );
}

#[test]
fn callbacks_can_change_the_run_loop_mask() {
    let mut engine: Engine<usize> = Engine::new(size(400, 300), 30);
    let id = engine.add_widget(button(10, 10));

    engine.on_click(id, |_state: &mut usize, ui| {
        ui.set_event_mask(EventMask::NONE)
    });
    engine.on_event(|count: &mut usize, _ui, event| {
        if matches!(event, Event::Engine(_)) {
            *count += 1;
        }
    });

    let mut count = 0;

    engine.dispatch(down(20, 15), &mut count);
    engine.dispatch(up(20, 15), &mut count);

    assert_eq!(engine.event_mask(), EventMask::NONE);
    // `WidgetAdded` and the mask change.
    assert_eq!(count, 2);
}

/// Records every engine event, filtered by kind, and the engine events it received as a widget.
fn engine_listener(mask: EventMask) -> Recorder {
    let mut core = pushrod::widget::WidgetCore::new(point(10, 10), size(20, 20));

    core.set_event_mask(mask);

    Recorder { core }
}

#[test]
fn adding_a_widget_emits_widget_added_but_the_root_does_not() {
    let mut engine: Engine<Vec<EngineEvent>> = Engine::new(size(400, 300), 30);
    let id = engine.add_widget(button(10, 10));

    engine.on_engine_event(|seen, _ui, event| seen.push(event.clone()));

    let mut seen = Vec::new();

    engine.dispatch(motion(500, 500), &mut seen);

    assert_eq!(
        seen,
        vec![EngineEvent::WidgetAdded {
            id,
            parent: pushrod::widget::ROOT_WIDGET_ID
        }]
    );
}

#[test]
fn engine_callbacks_can_filter_by_kind() {
    let mut engine: Engine<Vec<EngineEventKind>> = Engine::new(size(400, 300), 30);

    engine.add_widget(button(10, 10));
    engine.set_event_mask(EventMask::ALL & !EventMask::HOVER);
    engine.on_engine_event_filtered(&[EngineEventKind::EventMaskChanged], |seen, _ui, event| {
        seen.push(event.kind())
    });

    let mut seen = Vec::new();

    engine.dispatch(motion(500, 500), &mut seen);

    assert_eq!(seen, vec![EngineEventKind::EventMaskChanged]);
}

#[test]
fn widgets_receive_engine_events_only_with_the_engine_mask_and_filter() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(pushrod::root_widget::RootWidget::new(size(
        100, 100,
    ))));

    let all = cache.add(Box::new(engine_listener(EventMask::ENGINE)));
    let none = cache.add(Box::new(engine_listener(EventMask::KEYBOARD)));
    let filtered = cache.add(Box::new(engine_listener(EventMask::ENGINE)));

    cache.set_engine_filter(filtered, Some(vec![EngineEventKind::EventMaskChanged]));
    cache.take_pending_events();

    let echoed = cache.broadcast_engine_event(&EngineEvent::WidgetAdded { id: 9, parent: 0 });

    // One echo only: the `ENGINE` widget without a filter.
    assert_eq!(echoed.len(), 1);
    assert_ne!(all, none);

    cache.set_engine_filter(filtered, None);

    let echoed = cache.broadcast_engine_event(&EngineEvent::WidgetAdded { id: 9, parent: 0 });

    assert_eq!(echoed.len(), 2);
}

#[test]
fn the_run_loop_mask_can_silence_widget_engine_events() {
    let mut engine: Engine<usize> = Engine::new(size(400, 300), 30);

    engine.add_widget(engine_listener(EventMask::ENGINE));
    engine.set_event_mask(EventMask::ALL & !EventMask::ENGINE);
    engine.on_event(|count: &mut usize, _ui, event| {
        if matches!(event, Event::Engine(_)) {
            *count += 1;
        }
    });

    let mut count = 0;

    engine.dispatch(motion(500, 500), &mut count);

    // `EventMaskChanged` and `WidgetAdded` reach the callback once each; the widget echoes nothing.
    assert_eq!(count, 2);
}

#[test]
fn engine_events_do_not_loop_when_widgets_respond_to_them() {
    let mut engine: Engine<usize> = Engine::new(size(400, 300), 30);

    engine.add_widget(engine_listener(EventMask::ENGINE));
    engine.on_engine_event(|count: &mut usize, _ui, _event| *count += 1);

    let mut count = 0;

    engine.dispatch(motion(500, 500), &mut count);
    engine.dispatch(motion(500, 500), &mut count);

    // `WidgetAdded`, plus the copy this test widget emits in response.  The copy is not broadcast
    // again, and the second dispatch adds nothing, so there is no loop.
    assert_eq!(count, 2);
}

#[test]
fn registry_callbacks_fire_from_real_pointer_input_on_a_button() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut engine: Engine<Vec<String>> = Engine::new(size(200, 100), 30);
    let id = engine.add_widget(button(10, 10));
    let other = engine.add_widget(button(10, 50));
    let order = Rc::new(RefCell::new(Vec::new()));

    let log = order.clone();
    engine.on_hover(id, move |_, _, inside| {
        log.borrow_mut().push(format!("hover {inside}"))
    });
    let log = order.clone();
    engine.on_press(id, move |_, _| log.borrow_mut().push("press".into()));
    let log = order.clone();
    engine.on_release(id, move |_, _| log.borrow_mut().push("release".into()));
    let log = order.clone();
    engine.on_click(id, move |_, _| log.borrow_mut().push("click".into()));
    // A callback on the other button must stay silent.
    let log = order.clone();
    engine.on_press(other, move |_, _| log.borrow_mut().push("other".into()));

    let mut state = Vec::new();
    for event in [motion(20, 20), down(20, 20), up(20, 20), motion(150, 90)] {
        engine.dispatch(event, &mut state);
    }

    assert_eq!(
        *order.borrow(),
        vec!["hover true", "press", "click", "release", "hover false"]
    );
}
