// Event dispatch tests.  These exercise the WidgetCache and Engine without opening a window.

use pushrod::cache::WidgetCache;
use pushrod::engine::Engine;
use pushrod::event::{Event, EventKind};
use pushrod::geometry::{point, size};
use pushrod_widgets::base_widget::BaseWidget;
use pushrod_widgets::button_widget::ButtonWidget;
use pushrod_widgets::text_widget::{TextJustify, TextWidget};
use sdl2::event::Event as SdlEvent;
use sdl2::mouse::{MouseButton, MouseState};
use sdl2::pixels::Color;
use sdl2::ttf::FontStyle;

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
    engine.on_event(|state, _ui, _event| state.events += 1);

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

    fn paint(
        &self,
        _c: &mut sdl2::render::Canvas<sdl2::video::Window>,
        _fc: &mut pushrod::font::FontCache,
    ) {
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut pushrod::event::EventCtx) {
        ctx.emit(event.clone());
    }
}

fn recorder(x: i32, y: i32) -> Recorder {
    Recorder {
        core: pushrod::widget::WidgetCore::new(point(x, y), size(100, 20)),
    }
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

    assert!(matches!(
        events.as_slice(),
        [Event::Clicked { id, button: MouseButton::Left, count: 1 }] if *id == button
    ));
}
