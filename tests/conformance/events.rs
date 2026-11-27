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

//! Conformance for `docs/specs/events.md`: phases, propagation, masks, engine events, ordering.

use crate::common::{PhaseLog, cache, probe, probe_with_mask};
use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, EngineEventKind, Event, EventKind, EventMask, EventPhase};
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers};
use pushrod::root_widget::RootWidget;
use std::cell::RefCell;
use std::rc::Rc;

/// A key press addressed to `id`.
fn key_press(id: u32) -> Event {
    Event::KeyPressed {
        id,
        key: Key::Return,
        modifiers: Modifiers::empty(),
        repeat: false,
    }
}

/// Events spec, "Phases": capture runs root-down through widgets that opt in, then the target,
/// then bubble runs target-parent-up through every ancestor.
#[test]
fn delivery_is_capture_then_target_then_bubble() {
    use EventPhase::{Bubble, Capture, Target};

    let log: PhaseLog = Rc::default();
    let mut cache = cache();
    let dialog = probe(&mut cache, 0, "dialog", true, None, &log);
    let group = probe(&mut cache, dialog, "group", false, None, &log);
    let field = probe(&mut cache, group, "field", true, None, &log);

    cache.propagate(field, &key_press(field));

    // `group` does not opt in to capture; `field` is the target, so it is not captured either.
    assert_eq!(
        *log.borrow(),
        vec![
            ("dialog", Capture),
            ("field", Target),
            ("group", Bubble),
            ("dialog", Bubble),
        ]
    );
}

/// Events spec, "Handled and ignored": `Stop` in any phase ends delivery at once.
#[test]
fn stop_in_each_phase_ends_delivery() {
    use EventPhase::{Bubble, Capture, Target};

    let cases = [
        (Capture, vec![("outer", Capture)]),
        (Target, vec![("outer", Capture), ("field", Target)]),
        (
            Bubble,
            vec![("outer", Capture), ("field", Target), ("inner", Bubble)],
        ),
    ];

    for (phase, expected) in cases {
        let log: PhaseLog = Rc::default();
        let mut cache = cache();
        let outer = probe(
            &mut cache,
            0,
            "outer",
            true,
            (phase == Capture).then_some(Capture),
            &log,
        );
        let inner = probe(
            &mut cache,
            outer,
            "inner",
            false,
            (phase == Bubble).then_some(Bubble),
            &log,
        );
        let field = probe(
            &mut cache,
            inner,
            "field",
            false,
            (phase == Target).then_some(Target),
            &log,
        );

        cache.propagate(field, &key_press(field));
        assert_eq!(*log.borrow(), expected, "stopping in {phase:?}");
    }
}

/// Events spec, "Which events propagate": hover and semantic events go to the target only.
#[test]
fn events_that_do_not_propagate_reach_only_the_target() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();
    let outer = probe(&mut cache, 0, "outer", true, None, &log);
    let field = probe(&mut cache, outer, "field", true, None, &log);

    cache.propagate(field, &Event::PointerEntered { id: field });

    assert_eq!(*log.borrow(), vec![("field", EventPhase::Target)]);
    assert!(!EventKind::PointerEntered.propagates());
    assert!(!EventKind::Clicked.propagates());
    assert!(EventKind::KeyPressed.propagates());
    assert!(EventKind::PointerPressed.propagates());
}

/// Events spec, "Masks": a widget whose mask excludes the event's class is skipped, but its
/// ancestors still see the bubble.
#[test]
fn a_widget_mask_filters_its_own_deliveries() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();
    let outer = probe(&mut cache, 0, "outer", false, None, &log);
    let deaf = probe_with_mask(
        &mut cache,
        outer,
        "deaf",
        false,
        None,
        &log,
        EventMask::POINTER_BUTTONS,
    );

    cache.propagate(deaf, &key_press(deaf));

    assert_eq!(*log.borrow(), vec![("outer", EventPhase::Bubble)]);
}

/// Events spec, "Masks": broadcasts reach only widgets whose mask includes the class.
#[test]
fn a_broadcast_reaches_only_widgets_whose_mask_includes_the_class() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();

    probe_with_mask(
        &mut cache,
        0,
        "window",
        false,
        None,
        &log,
        EventMask::WINDOW,
    );
    probe_with_mask(&mut cache, 0, "timer", false, None, &log, EventMask::TIMER);
    probe_with_mask(&mut cache, 0, "none", false, None, &log, EventMask::NONE);
    cache.broadcast(&Event::WindowFocusChanged { focused: true });

    let seen: Vec<_> = log.borrow().iter().map(|(name, _)| *name).collect();

    assert_eq!(seen, vec!["window"]);
}

/// Events spec, "Masks": the run-loop mask is applied before any widget sees an input event.
#[test]
fn the_run_loop_mask_drops_input_before_routing() {
    let mut engine: Engine<()> = Engine::new(size(100, 100), 30);
    let log: PhaseLog = Rc::default();
    probe(engine.widgets(), 0, "field", false, None, &log);

    engine.set_event_mask(EventMask::KEYBOARD);
    engine.dispatch(
        InputEvent::MouseMove {
            position: point(10, 10),
        },
        &mut (),
    );
    assert!(log.borrow().is_empty(), "motion is outside the loop mask");

    engine.set_event_mask(EventMask::ALL);
    engine.dispatch(
        InputEvent::MouseMove {
            position: point(10, 10),
        },
        &mut (),
    );
    assert!(
        !log.borrow().is_empty(),
        "motion arrives once the mask allows it"
    );
}

/// Events spec, "Ordering guarantees": a broadcast visits parents before children and siblings in
/// the order they were added.
#[test]
fn broadcast_order_is_parents_first_then_siblings_in_add_order() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();
    let first = probe(&mut cache, 0, "first", false, None, &log);

    probe(&mut cache, first, "first-child", false, None, &log);
    probe(&mut cache, 0, "second", false, None, &log);
    cache.broadcast(&Event::WindowFocusChanged { focused: false });

    let seen: Vec<_> = log.borrow().iter().map(|(name, _)| *name).collect();

    assert_eq!(seen, vec!["first", "first-child", "second"]);
}

/// Events spec, "Ordering guarantees": a broadcast queues one `EventBroadcast` engine event that
/// reports how many widgets received it.
#[test]
fn a_broadcast_reports_its_recipient_count() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();

    probe(&mut cache, 0, "one", false, None, &log);
    probe(&mut cache, 0, "two", false, None, &log);
    cache.broadcast(&Event::WindowFocusChanged { focused: true });

    let pending = cache.take_pending_events();
    let reports: Vec<_> = pending
        .iter()
        .filter_map(|event| match event {
            Event::Engine(EngineEvent::EventBroadcast { kind, recipients }) => {
                Some((*kind, *recipients))
            }
            _ => None,
        })
        .collect();

    assert_eq!(reports, vec![(EventKind::WindowFocusChanged, 2)]);
}

/// Events spec, "Engine events": adding and removing a widget each emit an engine event, in
/// order, to the engine callbacks.
#[test]
fn adding_and_removing_widgets_emit_engine_events_in_order() {
    let mut engine: Engine<Vec<EngineEventKind>> = Engine::new(size(100, 100), 30);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();

    engine.on_engine_event(move |_, _, event| sink.borrow_mut().push(event.clone()));

    let id = engine.add_widget(RootWidget::new(size(10, 10)));
    let mut state = Vec::new();

    engine.dispatch(InputEvent::Wake, &mut state);
    engine.widgets().remove(id).unwrap();
    engine.dispatch(InputEvent::Wake, &mut state);

    let kinds: Vec<_> = seen.borrow().iter().map(EngineEvent::kind).collect();

    assert_eq!(
        kinds,
        vec![EngineEventKind::WidgetAdded, EngineEventKind::WidgetRemoved]
    );
    assert!(matches!(
        seen.borrow()[0],
        EngineEvent::WidgetAdded { id: added, parent: 0 } if added == id
    ));
}

/// Events spec, "Engine events": a widget receives them only when its mask includes `ENGINE`, and
/// a kind filter narrows them further.
#[test]
fn engine_events_reach_only_widgets_that_ask_for_them() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();

    probe_with_mask(&mut cache, 0, "all", false, None, &log, EventMask::ALL);
    probe_with_mask(
        &mut cache,
        0,
        "quiet",
        false,
        None,
        &log,
        EventMask::KEYBOARD,
    );
    let filtered = probe_with_mask(&mut cache, 0, "filtered", false, None, &log, EventMask::ALL);

    cache.set_engine_filter(filtered, Some(vec![EngineEventKind::FocusChanged]));
    log.borrow_mut().clear();
    cache.broadcast_engine_event(&EngineEvent::WidgetRemoved { id: 99, parent: 0 });

    let seen: Vec<_> = log.borrow().iter().map(|(name, _)| *name).collect();

    assert_eq!(seen, vec!["all"]);
}

/// Events spec, "Engine events": a response to an engine event is never broadcast again, so
/// engine events cannot loop.
#[test]
fn engine_events_do_not_loop() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();

    probe(&mut cache, 0, "once", false, None, &log);
    log.borrow_mut().clear();
    cache.broadcast_engine_event(&EngineEvent::WidgetRemoved { id: 99, parent: 0 });

    assert_eq!(log.borrow().len(), 1);
}

/// Events spec, "Ordering guarantees": callbacks and the event loop see the same events in the
/// same order.
#[test]
fn callbacks_and_the_event_loop_see_the_same_sequence() {
    /// Builds an engine with two widgets and a scripted backend, shared by both styles.
    fn scripted() -> (Engine<()>, HeadlessBackend) {
        let mut engine: Engine<()> = Engine::new(size(100, 100), 30);
        let mut backend = HeadlessBackend::new(size(100, 100));

        engine.add_widget(RootWidget::new(size(10, 10)));
        backend.push_event(InputEvent::FocusChanged(true));
        backend.push_event(InputEvent::Resized(size(120, 100)));
        backend.push_event(InputEvent::Quit);
        (engine, backend)
    }

    let by_callback = Rc::new(RefCell::new(Vec::new()));
    let (mut engine, mut backend) = scripted();
    let sink = by_callback.clone();

    engine.on_event(move |_, _, event| sink.borrow_mut().push(format!("{event:?}")));
    engine.run(&mut backend, ());

    let (mut engine, backend) = scripted();
    let mut events = engine.event_loop(Box::new(backend));
    let mut by_loop = Vec::new();

    while let Some(event) = engine.next_event(&mut events) {
        by_loop.push(format!("{event:?}"));
    }

    assert!(!by_loop.is_empty());
    assert_eq!(*by_callback.borrow(), by_loop);
}

/// Events spec, "Composition": a composition emits `CompositionStarted`, one `CompositionUpdated`
/// per change, and `CompositionEnded` (committed) when the final text arrives; the widget sees the
/// composition as `TextComposing` and the commit as `TextEntered`.
#[test]
fn a_composition_emits_started_updated_and_ended() {
    let log: PhaseLog = Rc::default();
    let mut cache = cache();
    let field = probe(&mut cache, 0, "field", false, None, &log);

    cache.set_focus(field);
    cache.take_pending_events();
    cache.handle_event(InputEvent::Composition {
        text: "に".to_string(),
        start: 1,
        length: 0,
    });
    cache.handle_event(InputEvent::Text("日".to_string()));

    let kinds: Vec<_> = cache
        .take_pending_events()
        .iter()
        .filter_map(|event| match event {
            Event::Engine(engine) => Some(engine.kind()),
            _ => None,
        })
        .collect();

    assert_eq!(
        kinds,
        vec![
            EngineEventKind::CompositionStarted,
            EngineEventKind::CompositionUpdated,
            EngineEventKind::CompositionEnded,
        ]
    );
}
