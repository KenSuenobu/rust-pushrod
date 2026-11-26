//! High-DPI rendering and window resize (issue #25), on the headless backend: the root follows
//! the window and its layout re-arranges, the root layout's minimum size becomes the window's,
//! and a scale change repaints every `Widget` at the new physical resolution.

use pushrod::backend::Backend;
use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::{size, to_physical_size};
use pushrod::input::InputEvent;
use pushrod::layout::{Align, Alignment, Column, LayoutParams, Margin, SizeHint};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{ROOT_WIDGET_ID, WidgetId};

/// Builds a 200x100 window whose root has a column layout with 10-pixel margins, holding one
/// field that fills the width and is at least 60x20.
///
/// # Returns
///
/// The engine, recording its engine events, and the field's ID.
fn form() -> (Engine<Vec<EngineEvent>>, WidgetId) {
    let mut engine: Engine<Vec<EngineEvent>> = Engine::new(size(200, 100), 30);
    let field = engine.add_widget(RootWidget::new(size(100, 20)));
    let widgets = engine.widgets();

    widgets
        .get_mut(field)
        .unwrap()
        .set_layout_params(LayoutParams {
            alignment: Alignment::new(Align::Fill, Align::Start),
            ..LayoutParams::default()
        });
    widgets.get_mut(field).unwrap().set_size_hint(SizeHint::new(
        size(60, 20),
        size(100, 20),
        size(10_000, 20),
    ));
    widgets
        .get_mut(ROOT_WIDGET_ID)
        .unwrap()
        .set_layout(Box::new(Column::new().margins(Margin::all(10))));
    engine.on_engine_event(|log, _ui, event| log.push(event.clone()));

    (engine, field)
}

#[test]
fn resizing_the_window_resizes_the_root_and_re_runs_its_layout() {
    let (mut engine, field) = form();
    let mut backend = HeadlessBackend::new(size(200, 100));
    let mut log = Vec::new();

    engine.widgets().draw_loop(&mut backend);
    assert_eq!(engine.widgets().get(field).unwrap().get_size().w, 180);

    engine.dispatch(InputEvent::Resized(size(300, 150)), &mut log);
    engine.widgets().draw_loop(&mut backend);

    assert_eq!(engine.get_size(), size(300, 150));
    assert_eq!(
        engine.widgets().get(ROOT_WIDGET_ID).unwrap().get_size(),
        size(300, 150)
    );
    assert_eq!(engine.widgets().get(field).unwrap().get_size().w, 280);
}

#[test]
fn the_root_layouts_minimum_size_becomes_the_windows() {
    let (mut engine, _) = form();
    let mut backend = HeadlessBackend::new(size(200, 100));

    backend.push_event(InputEvent::Resized(size(320, 240)));

    let log = engine.run(&mut backend, Vec::new());

    // The field's 60x20 minimum plus 10-pixel margins on every side.
    assert_eq!(engine.minimum_window_size(), Some(size(80, 40)));
    assert_eq!(backend.minimum_size, Some(size(80, 40)));
    assert!(log.contains(&EngineEvent::MinimumWindowSizeChanged { size: size(80, 40) }));
    assert!(log.contains(&EngineEvent::LayoutChanged {
        container: ROOT_WIDGET_ID
    }));
}

#[test]
fn a_window_opened_on_a_high_dpi_display_paints_at_physical_size() {
    let mut engine: Engine<Vec<EngineEvent>> = Engine::new(size(200, 100), 30);
    let mut backend = HeadlessBackend::with_scale_factor(size(200, 100), 2.0);

    engine.on_engine_event(|log, _ui, event| log.push(event.clone()));

    let log = engine.run(&mut backend, Vec::new());

    assert_eq!(engine.scale_factor(), 2.0);
    assert!(log.contains(&EngineEvent::ScaleFactorChanged {
        old_percent: 100,
        new_percent: 200
    }));

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);

    assert_eq!(backend.texture_sizes[&ROOT_WIDGET_ID], size(400, 200));
}

#[test]
fn moving_to_another_display_repaints_at_the_new_scale() {
    let (mut engine, field) = form();
    let mut backend = HeadlessBackend::new(size(200, 100));
    let mut log = Vec::new();

    engine.widgets().draw_loop(&mut backend);
    assert_eq!(backend.texture_sizes[&field], size(180, 20));

    for scale in [1.5, 2.0, 1.0] {
        backend.set_scale_factor(scale);

        for event in backend.poll_events(None) {
            engine.dispatch(event, &mut log);
        }

        let painted = backend.painted.len();

        assert!(engine.widgets().draw_loop(&mut backend), "scale {scale}");
        assert!(backend.painted.len() > painted, "scale {scale}");
        assert_eq!(
            backend.texture_sizes[&field],
            to_physical_size(size(180, 20), scale)
        );
    }

    assert_eq!(
        log.iter()
            .filter_map(|e| match e {
                EngineEvent::ScaleFactorChanged {
                    old_percent,
                    new_percent,
                } => Some((*old_percent, *new_percent)),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec![(100, 150), (150, 200), (200, 100)]
    );
}

#[test]
fn an_unchanged_scale_repaints_nothing() {
    let mut engine: Engine<Vec<EngineEvent>> = Engine::new(size(50, 50), 30);
    let mut backend = HeadlessBackend::new(size(50, 50));
    let mut log = Vec::new();

    engine.on_engine_event(|log, _ui, event| log.push(event.clone()));
    engine.widgets().draw_loop(&mut backend);
    engine.dispatch(InputEvent::ScaleChanged(1.0), &mut log);

    assert!(!engine.widgets().draw_loop(&mut backend));
    assert!(log.is_empty());

    // An unusable scale reads as 1x, so it changes nothing either.
    engine.dispatch(InputEvent::ScaleChanged(f32::NAN), &mut log);
    assert!(!engine.widgets().draw_loop(&mut backend));
    assert_eq!(engine.scale_factor(), 1.0);
}
