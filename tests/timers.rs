//! Timers, animation ticks, and idle callbacks.  Time is driven by hand through
//! `Engine::advance`, so nothing here waits.

use pushrod::engine::{Engine, TimerId};
use pushrod::event::{EngineEvent, EngineEventKind, Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::widget::{Widget, WidgetCore, WidgetId};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A `Widget` that counts the ticks it receives, and keeps animating for a set number of them.
struct Spinner {
    core: WidgetCore,
    ticks: Rc<RefCell<Vec<Duration>>>,
    frames_left: u32,
}

impl Widget for Spinner {
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

    /// Records each tick and asks for another while frames remain.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::Tick { elapsed } = event {
            self.ticks.borrow_mut().push(*elapsed);

            if self.frames_left > 0 {
                self.frames_left -= 1;
                ctx.request_animation();
            }
        }

        Propagation::Continue
    }
}

/// Adds a spinner, with or without `TIMER` in its mask.
fn spinner(
    engine: &mut Engine<u32>,
    frames: u32,
    timer: bool,
) -> (WidgetId, Rc<RefCell<Vec<Duration>>>) {
    let ticks = Rc::new(RefCell::new(Vec::new()));
    let mut core = WidgetCore::new(point(0, 0), size(10, 10));

    if timer {
        core.set_event_mask(EventMask::TIMER);
    } else {
        core.set_event_mask(EventMask::NONE);
    }

    let id = engine.add_widget(Spinner {
        core,
        ticks: ticks.clone(),
        frames_left: frames,
    });

    (id, ticks)
}

/// Records every engine event.
fn log_engine_events(engine: &mut Engine<u32>) -> Rc<RefCell<Vec<EngineEvent>>> {
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();

    engine.on_engine_event(move |_state, _ui, event| sink.borrow_mut().push(event.clone()));

    log
}

#[test]
fn a_one_shot_timer_fires_once_after_its_interval() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut count = 0;
    let id = engine.on_timer(ms(100), false, |count, _ui| *count += 1);
    let start = Instant::now();

    engine.advance(start + ms(50), &mut count);
    assert_eq!(count, 0);

    engine.advance(start + ms(150), &mut count);
    assert_eq!(count, 1);

    engine.advance(start + ms(1000), &mut count);
    assert_eq!(count, 1);

    // A finished one-shot has nothing left to cancel.
    assert!(!engine.cancel_timer(id));
}

#[test]
fn a_repeating_timer_fires_every_interval_without_bursting() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut count = 0;
    let start = Instant::now();

    engine.on_timer(ms(100), true, |count, _ui| *count += 1);

    // The loop stalls for 5 intervals: one firing, not five.
    engine.advance(start + ms(550), &mut count);
    assert_eq!(count, 1);

    // The rhythm holds: the next is at 600 ms.
    engine.advance(start + ms(590), &mut count);
    assert_eq!(count, 1);
    engine.advance(start + ms(650), &mut count);
    assert_eq!(count, 2);
}

#[test]
fn a_timer_cancels_itself_from_its_own_callback() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut count = 0;
    let own_id = Rc::new(RefCell::new(None::<TimerId>));
    let slot = own_id.clone();
    let id = engine.on_timer(ms(10), true, move |count, ui| {
        *count += 1;

        if *count == 2 {
            ui.cancel_timer(slot.borrow().unwrap());
        }
    });
    let start = Instant::now();

    *own_id.borrow_mut() = Some(id);

    for i in 1..=6 {
        engine.advance(start + ms(i * 20), &mut count);
    }

    assert_eq!(count, 2);
}

#[test]
fn a_callback_cancels_another_timer_due_in_the_same_pass() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut count = 0;
    let victim = Rc::new(RefCell::new(None::<TimerId>));
    let slot = victim.clone();

    // Due first, cancels the second even though both are due at the same `advance`.
    engine.on_timer(ms(10), false, move |_c, ui| {
        ui.cancel_timer(slot.borrow().unwrap());
    });
    *victim.borrow_mut() = Some(engine.on_timer(ms(20), false, |count, _ui| *count += 100));

    engine.advance(Instant::now() + ms(500), &mut count);

    assert_eq!(count, 0);
}

#[test]
fn timers_emit_engine_events_for_firing_and_cancelling() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let log = log_engine_events(&mut engine);
    let start = Instant::now();
    let once = engine.on_timer(ms(10), false, |_c, _ui| {});
    let repeating = engine.on_timer(ms(10), true, |_c, _ui| {});

    engine.advance(start + ms(20), &mut state);
    assert!(engine.cancel_timer(repeating));
    engine.advance(start + ms(30), &mut state);

    let events = log.borrow();

    assert!(events.contains(&EngineEvent::TimerFired {
        id: once,
        repeat: false
    }));
    assert!(events.contains(&EngineEvent::TimerFired {
        id: repeating,
        repeat: true
    }));
    assert!(events.contains(&EngineEvent::TimerCancelled { id: repeating }));
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind() == EngineEventKind::TimerFired)
            .count(),
        2
    );
}

#[test]
fn tick_flows_only_while_a_widget_requests_animation() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let (_id, ticks) = spinner(&mut engine, 0, true);
    let start = Instant::now();

    // Nobody asked: nothing ticks, however many frames pass.
    engine.advance(start + ms(16), &mut state);
    engine.advance(start + ms(32), &mut state);
    assert!(ticks.borrow().is_empty());
}

#[test]
fn an_animation_request_lasts_one_frame_and_ticks_carry_elapsed_time() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let (id, ticks) = spinner(&mut engine, 1, true);
    let log = log_engine_events(&mut engine);
    let start = Instant::now();

    engine.widgets().request_animation(id);

    engine.advance(start + ms(16), &mut state);
    engine.advance(start + ms(32), &mut state);
    engine.advance(start + ms(48), &mut state);

    // Ticks at 16 and 32 ms (the widget asked again once), none at 48 ms.
    assert_eq!(ticks.borrow().len(), 2);
    assert_eq!(ticks.borrow()[1], ms(16));

    let ticked = log
        .borrow()
        .iter()
        .filter(|e| matches!(e, EngineEvent::AnimationTicked { recipients: 1, .. }))
        .count();

    assert_eq!(ticked, 2);
}

#[test]
fn a_widget_without_timer_in_its_mask_gets_no_tick() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let (id, ticks) = spinner(&mut engine, 5, false);

    engine.widgets().request_animation(id);
    engine.advance(Instant::now() + ms(16), &mut state);

    assert!(ticks.borrow().is_empty());
}

#[test]
fn a_callback_can_start_an_animation_through_ui() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let (id, ticks) = spinner(&mut engine, 0, true);

    engine.on_timer(ms(10), false, move |_c, ui| ui.request_animation(id));

    let start = Instant::now();

    // The timer fires first, so the tick follows in the same advance.
    engine.advance(start + ms(20), &mut state);

    assert_eq!(ticks.borrow().len(), 1);
}

#[test]
fn idle_callbacks_run_each_time_the_loop_is_idle_and_emit_an_engine_event() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let log = log_engine_events(&mut engine);

    engine.on_idle(|count, _ui| *count += 1);
    engine.on_idle(|count, _ui| *count += 10);
    engine.run_idle(&mut state);
    engine.run_idle(&mut state);

    assert_eq!(state, 22);
    assert!(
        log.borrow()
            .contains(&EngineEvent::IdleRan { callbacks: 2 })
    );
}

#[test]
fn without_idle_callbacks_nothing_is_emitted() {
    let mut engine: Engine<u32> = Engine::new(size(100, 100), 60);
    let mut state = 0;
    let log = log_engine_events(&mut engine);

    engine.run_idle(&mut state);

    assert!(log.borrow().is_empty());
}
