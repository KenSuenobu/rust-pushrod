// CustomPaint tests.  The widget paints into a RecordingPainter and receives events through a
// WidgetCache and Engine, so no window or display is needed.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::{Event, EventMask, Propagation};
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::paint;
use pushrod::widget::Widget;
use pushrod_widgets::custom_paint::CustomPaint;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::prelude::CustomPaint as PreludeCustomPaint;
use std::cell::Cell;
use std::rc::Rc;

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

fn down(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn up(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseUp {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

#[test]
fn closure_paints_with_the_widget_size() {
    let widget = CustomPaint::new(point(5, 6), size(40, 20), |p, size, _| {
        paint::fill(p, Color::WHITE);
        paint::border(p, size, Color::BLACK, 1);
    });

    assert_eq!(widget.get_origin(), point(5, 6));
    assert_eq!(widget.get_size(), size(40, 20));
    assert_eq!(
        painted(&widget),
        vec![
            PaintCall::Fill(Color::WHITE),
            PaintCall::Rect(Rect::new(0, 0, 40, 20), Color::BLACK),
        ]
    );
}

#[test]
fn closure_reads_the_data_and_repaints_after_it_changes() {
    let mut widget =
        CustomPaint::with_data(point(0, 0), size(100, 50), vec![10u32, 20], |p, _, bars| {
            for (i, bar) in bars.iter().enumerate() {
                p.fill_rect(Rect::new(i as i32 * 10, 0, 5, *bar), Color::BLUE);
            }
        });

    assert_eq!(painted(&widget).len(), 2);

    widget.set_invalidated(false);
    widget.data_mut().push(30);

    assert!(widget.is_invalidated());
    assert_eq!(painted(&widget).len(), 3);

    widget.set_invalidated(false);
    widget.set_data(vec![5]);

    assert!(widget.is_invalidated());
    assert_eq!(widget.data(), &vec![5]);
    assert_eq!(painted(&widget).len(), 1);
}

#[test]
fn paint_runs_only_when_painted() {
    let runs = Rc::new(Cell::new(0));
    let counter = Rc::clone(&runs);
    let widget = CustomPaint::new(point(0, 0), size(10, 10), move |_, _, _| {
        counter.set(counter.get() + 1);
    });

    assert_eq!(runs.get(), 0);
    painted(&widget);
    assert_eq!(runs.get(), 1);
}

#[test]
fn without_an_event_closure_it_receives_nothing() {
    let widget = CustomPaint::new(point(0, 0), size(10, 10), |_, _, _| {});

    assert_eq!(widget.event_mask(), EventMask::NONE);
}

#[test]
fn event_closure_receives_events_and_changes_the_data() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().size(200, 200)));

    let id = cache.add(Box::new(
        CustomPaint::with_data(point(10, 10), size(100, 100), 0u32, |p, _, presses| {
            p.fill_rect(Rect::new(0, 0, *presses, 1), Color::RED);
        })
        .on_event(|presses, event, _ctx| {
            if matches!(event, Event::PointerPressed { .. }) {
                *presses += 1;
                return Propagation::Stop;
            }

            Propagation::Continue
        }),
    ));

    cache.handle_event(down(20, 20));
    cache.handle_event(up(20, 20));
    cache.get_mut(id).unwrap().set_invalidated(false);
    cache.handle_event(down(30, 30));

    let widget = cache.get_as::<CustomPaint<u32>>(id).unwrap();

    assert_eq!(*widget.data(), 2);
    assert!(widget.is_invalidated(), "an event repaints the widget");
}

#[test]
fn event_closure_can_emit_events() {
    let mut engine: Engine<u32> = Engine::new(size(200, 200), 60);
    let id = engine.add_widget(
        CustomPaint::new(point(10, 10), size(100, 100), |_, _, _| {}).on_event(|_, event, ctx| {
            if let Event::PointerReleased { id, .. } = event {
                ctx.emit(Event::Clicked {
                    id: *id,
                    button: MouseButton::Left,
                    count: 1,
                });
            }

            Propagation::Continue
        }),
    );

    engine.on_click(id, |clicks, _ui| *clicks += 1);

    let mut clicks = 0;

    engine.dispatch(down(20, 20), &mut clicks);
    engine.dispatch(up(20, 20), &mut clicks);

    assert_eq!(clicks, 1);
}

#[test]
fn event_mask_overrides_the_default() {
    let widget = CustomPaint::new(point(0, 0), size(10, 10), |_, _, _| {})
        .on_event(|_, _, _| Propagation::Continue)
        .with_event_mask(EventMask::SCROLL);

    assert_eq!(widget.event_mask(), EventMask::SCROLL);
}

#[test]
fn prelude_exports_custom_paint_and_debug_works() {
    let widget = PreludeCustomPaint::new(point(0, 0), size(10, 10), |_, _, _| {});

    assert!(format!("{widget:?}").contains("CustomPaint"));
}
