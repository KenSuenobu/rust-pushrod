//! Custom application events: posting, targets, payload typing, ordering, the per-frame cap, and
//! cross-thread posting.  Everything runs headless through `Engine::dispatch`.

use pushrod::engine::{Engine, MAX_CUSTOM_EVENTS_PER_FRAME};
use pushrod::event::{EngineEvent, Event, EventCtx, EventMask, EventTarget, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::widget::{Widget, WidgetCore, WidgetId};
use sdl2::event::Event as SdlEvent;
use std::cell::RefCell;
use std::rc::Rc;

/// A payload type the tests post.
struct Progress(u8);

/// A payload type no callback listens for.
struct Unrelated;

type Log = Rc<RefCell<Vec<String>>>;

/// A `Widget` that logs the `Progress` payloads it receives and can repost one on receipt.
struct Probe {
    core: WidgetCore,
    log: Log,
    repost: Option<EventTarget>,
}

impl Widget for Probe {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Logs the payload; reposts a `Progress` if asked.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Some(Progress(percent)) = event.payload::<Progress>() {
            self.log
                .borrow_mut()
                .push(format!("widget {} got {}", self.get_id(), percent));

            if let Some(target) = self.repost {
                ctx.post(target, Progress(percent + 1));
            }
        }

        Propagation::Continue
    }
}

/// Adds a probe listening for custom events (or not) under `parent`.
fn probe(
    engine: &mut Engine<Log>,
    parent: WidgetId,
    log: &Log,
    listens: bool,
    repost: Option<EventTarget>,
) -> WidgetId {
    let mut core = WidgetCore::new(point(0, 0), size(10, 10));

    if listens {
        core.set_event_mask(EventMask::CUSTOM);
    }

    engine.add_child_widget(
        parent,
        Probe {
            core,
            log: log.clone(),
            repost,
        },
    )
}

fn engine() -> (Engine<Log>, Log) {
    (
        Engine::new(size(100, 100), 30),
        Rc::new(RefCell::new(Vec::new())),
    )
}

/// Delivers any queued events by dispatching an event nothing handles.
fn pump(engine: &mut Engine<Log>, log: &mut Log) {
    engine.dispatch(
        SdlEvent::Unknown {
            timestamp: 0,
            type_: 0,
        },
        log,
    );
}

#[test]
fn ui_post_reaches_on_custom_with_the_payload_intact() {
    let (mut engine, mut log) = engine();

    engine.on_event(|_log, ui, event| {
        if matches!(event, Event::Engine(_)) {
            ui.post(EventTarget::ApplicationOnly, Progress(40));
        }
    });
    engine.on_custom::<Progress, _>(|log, _ui, source, p| {
        log.borrow_mut().push(format!("app {:?} {}", source, p.0));
    });
    let a = probe(&mut engine, 0, &log, true, None);

    assert!(a > 0);
    pump(&mut engine, &mut log);

    assert_eq!(*log.borrow(), vec!["app None 40"]);
}

#[test]
fn widget_target_reaches_only_that_widget() {
    let (mut engine, mut log) = engine();
    let a = probe(&mut engine, 0, &log, true, None);
    let _b = probe(&mut engine, 0, &log, true, None);

    engine.sender().post(EventTarget::Widget(a), Progress(1));
    pump(&mut engine, &mut log);

    assert_eq!(*log.borrow(), vec![format!("widget {a} got 1")]);
}

#[test]
fn subtree_target_reaches_the_widget_and_its_descendants_in_order() {
    let (mut engine, mut log) = engine();
    let a = probe(&mut engine, 0, &log, true, None);
    let child = probe(&mut engine, a, &log, true, None);
    let _other = probe(&mut engine, 0, &log, true, None);

    engine.sender().post(EventTarget::Subtree(a), Progress(2));
    pump(&mut engine, &mut log);

    assert_eq!(
        *log.borrow(),
        vec![format!("widget {a} got 2"), format!("widget {child} got 2")]
    );
}

#[test]
fn broadcast_reaches_every_listening_widget_and_skips_the_others() {
    let (mut engine, mut log) = engine();
    let a = probe(&mut engine, 0, &log, true, None);
    let _deaf = probe(&mut engine, 0, &log, false, None);
    let c = probe(&mut engine, 0, &log, true, None);

    engine.sender().post(EventTarget::Broadcast, Progress(3));
    pump(&mut engine, &mut log);

    assert_eq!(
        *log.borrow(),
        vec![format!("widget {a} got 3"), format!("widget {c} got 3")]
    );
}

#[test]
fn application_only_reaches_no_widget_but_still_reaches_callbacks() {
    let (mut engine, mut log) = engine();
    let _a = probe(&mut engine, 0, &log, true, None);

    engine.on_event(|log, _ui, event| {
        if matches!(event, Event::Custom(_)) {
            log.borrow_mut().push("on_event".into());
        }
    });
    engine
        .sender()
        .post(EventTarget::ApplicationOnly, Progress(4));
    pump(&mut engine, &mut log);

    assert_eq!(*log.borrow(), vec!["on_event"]);
}

#[test]
fn on_custom_fires_only_for_its_payload_type() {
    let (mut engine, mut log) = engine();

    engine.on_custom::<Progress, _>(|log, _ui, _s, p| log.borrow_mut().push(format!("p{}", p.0)));
    engine
        .sender()
        .post(EventTarget::ApplicationOnly, Unrelated);
    engine
        .sender()
        .post(EventTarget::ApplicationOnly, Progress(5));
    pump(&mut engine, &mut log);

    assert_eq!(*log.borrow(), vec!["p5"]);
}

#[test]
fn delivery_order_is_widgets_then_on_custom_then_on_event() {
    let (mut engine, mut log) = engine();
    let a = probe(&mut engine, 0, &log, true, None);

    engine.on_custom::<Progress, _>(|log, _ui, _s, _p| log.borrow_mut().push("on_custom".into()));
    engine.on_event(|log, _ui, event| {
        if matches!(event, Event::Custom(_)) {
            log.borrow_mut().push("on_event".into());
        }
    });
    engine.sender().post(EventTarget::Widget(a), Progress(6));
    pump(&mut engine, &mut log);

    assert_eq!(
        *log.borrow(),
        vec![
            format!("widget {a} got 6"),
            "on_custom".into(),
            "on_event".into()
        ]
    );
}

#[test]
fn a_widget_ctx_post_names_the_widget_as_source() {
    let (mut engine, mut log) = engine();
    let a = probe(
        &mut engine,
        0,
        &log,
        true,
        Some(EventTarget::ApplicationOnly),
    );

    engine.on_custom::<Progress, _>(move |log, _ui, source, p| {
        log.borrow_mut()
            .push(format!("source {:?} {}", source, p.0));
    });
    engine.sender().post(EventTarget::Widget(a), Progress(7));
    pump(&mut engine, &mut log);

    assert_eq!(
        *log.borrow(),
        vec![
            format!("widget {a} got 7"),
            "source None 7".to_string(),
            format!("source Some({a}) 8"),
        ]
    );
}

#[test]
fn custom_events_have_no_core_side_effects() {
    let (mut engine, mut log) = engine();
    let a = probe(&mut engine, 0, &log, true, None);

    let invalidated_before = engine.widgets().get(a).unwrap().is_invalidated();

    engine.sender().post(EventTarget::Broadcast, Progress(1));
    pump(&mut engine, &mut log);

    assert_eq!(
        engine.widgets().get(a).unwrap().is_invalidated(),
        invalidated_before
    );
    assert_eq!(engine.widgets().get_current_widget(), 0);
}

#[test]
fn runaway_reposting_is_capped_and_reported() {
    let (mut engine, mut log) = engine();
    let drops = Rc::new(RefCell::new(0));
    let seen = drops.clone();

    engine.on_custom::<Progress, _>(|_log, ui, _s, p| {
        ui.post(EventTarget::ApplicationOnly, Progress(p.0));
    });
    engine.on_engine_event(move |_log, _ui, event| {
        if matches!(event, EngineEvent::CustomEventDropped { .. }) {
            *seen.borrow_mut() += 1;
        }
    });
    engine
        .sender()
        .post(EventTarget::ApplicationOnly, Progress(0));

    let delivered = Rc::new(RefCell::new(0));
    let counter = delivered.clone();

    engine.on_event(move |_log, _ui, event| {
        if matches!(event, Event::Custom(_)) {
            *counter.borrow_mut() += 1;
        }
    });
    pump(&mut engine, &mut log);

    assert_eq!(*delivered.borrow(), MAX_CUSTOM_EVENTS_PER_FRAME);
    assert_eq!(*drops.borrow(), 1);
}

#[test]
fn another_thread_can_post_and_the_ui_thread_receives_it() {
    let (mut engine, mut log) = engine();
    let a = probe(&mut engine, 0, &log, true, None);
    let sender = engine.sender();

    std::thread::spawn(move || {
        sender.post(EventTarget::Widget(a), Progress(40));
        sender.post(EventTarget::Widget(a), Progress(80));
    })
    .join()
    .unwrap();
    pump(&mut engine, &mut log);

    assert_eq!(
        *log.borrow(),
        vec![format!("widget {a} got 40"), format!("widget {a} got 80")]
    );
}
