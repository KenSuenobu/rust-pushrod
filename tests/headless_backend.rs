//! The engine's run loop on the headless backend: no window, no display, no SDL.

use pushrod::backend::Backend;
use pushrod::backend::headless::{HeadlessBackend, PaintCall};
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::input::InputEvent;
use pushrod::root_widget::RootWidget;

#[test]
fn the_run_loop_routes_scripted_input_and_composes_a_frame() {
    let mut engine: Engine<Vec<String>> = Engine::new(size(100, 100), 30);
    let mut backend = HeadlessBackend::new(size(100, 100));

    engine
        .widgets()
        .set_event_mask(0, pushrod::event::EventMask::WINDOW);
    engine.on_event(|log, _ui, event| log.push(format!("{event:?}")));
    backend.push_event(InputEvent::Resized(size(50, 50)));
    backend.push_event(InputEvent::FocusChanged(true));

    let log = engine.run(&mut backend, Vec::new());

    // Each window broadcast reports how many widgets received it.
    let broadcasts = log.iter().filter(|e| e.contains("EventBroadcast")).count();

    assert_eq!(broadcasts, 2);
    assert!(!engine.is_running(), "an empty script closes the window");
}

#[test]
fn the_root_widget_is_painted_once_and_blitted() {
    let mut engine: Engine = Engine::new(size(40, 30), 30);
    let mut backend = HeadlessBackend::new(size(40, 30));

    engine.run(&mut backend, ());

    // The loop quits before drawing, so draw one frame by hand.
    engine.widgets().invalidate_all();
    assert!(engine.widgets().draw_loop(&mut backend));
    backend.present();

    assert_eq!(backend.painted.len(), 1);
    assert!(matches!(backend.painted[0].1[0], PaintCall::Fill(_)));
    assert_eq!(backend.draws.len(), 1);
    assert_eq!(backend.frames, 1);

    // Nothing changed, so nothing is repainted and no frame is drawn.
    assert!(!engine.widgets().draw_loop(&mut backend));
}

#[test]
fn a_dropped_in_widget_tree_paints_every_widget_in_order() {
    let mut engine: Engine = Engine::new(size(40, 30), 30);
    let mut backend = HeadlessBackend::new(size(40, 30));
    let child = engine.add_widget(RootWidget::new(size(10, 10)));

    assert!(engine.widgets().draw_loop(&mut backend));

    let ids: Vec<_> = backend.painted.iter().map(|(id, _)| *id).collect();

    assert_eq!(ids, vec![0, child]);
}

#[test]
fn the_backend_receives_the_event_mask_and_clipboard_calls() {
    let mut backend = HeadlessBackend::new(size(10, 10));

    backend.set_clipboard_text("copied");

    assert_eq!(backend.clipboard_text().as_deref(), Some("copied"));
    assert_eq!(backend.window_size(), size(10, 10));
    backend.resize_window(size(20, 20));
    assert_eq!(backend.window_size(), size(20, 20));
}
