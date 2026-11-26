//! Row and column layouts through the widget cache: a dialog of fields above a right-aligned
//! button row, re-arranged as the window is resized, with minimum and maximum sizes kept.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::event::{EngineEvent, Event};
use pushrod::geometry::{Rect, size};
use pushrod::layout::{Align, Column, LayoutParams, Margin, Row, SizeHint, Spacer};
use pushrod::root_widget::RootWidget;
use pushrod::widget::WidgetId;

/// The widgets of the test dialog.
struct Dialog {
    cache: WidgetCache,
    backend: HeadlessBackend,
    field: WidgetId,
    capped: WidgetId,
    buttons: WidgetId,
    ok: WidgetId,
    cancel: WidgetId,
}

/// Adds a plain child of a size under a parent.
fn add(cache: &mut WidgetCache, parent: WidgetId, w: u32, h: u32) -> WidgetId {
    cache.add_child(parent, Box::new(RootWidget::new(size(w, h))))
}

/// Builds a window `w` pixels wide holding a column: a field that fills the width, a field that
/// fills up to 150 pixels, a stretching gap, and a row of OK and Cancel pushed to the right.
fn dialog(w: u32) -> Dialog {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(w, 200))));

    let field = add(&mut cache, 0, 100, 20);
    let capped = add(&mut cache, 0, 100, 20);

    cache.add_child(0, Box::new(Spacer::stretch()));

    let buttons = add(&mut cache, 0, 0, 0);

    cache.add_child(buttons, Box::new(Spacer::stretch()));

    let ok = add(&mut cache, buttons, 80, 24);
    let cancel = add(&mut cache, buttons, 80, 24);
    let fill = LayoutParams {
        alignment: pushrod::layout::Alignment::new(Align::Fill, Align::Start),
        ..LayoutParams::default()
    };

    cache.get_mut(field).unwrap().set_layout_params(fill);
    cache.get_mut(capped).unwrap().set_layout_params(fill);
    cache.get_mut(capped).unwrap().set_size_hint(SizeHint::new(
        size(60, 20),
        size(100, 20),
        size(150, 20),
    ));
    cache.get_mut(buttons).unwrap().set_layout_params(fill);
    cache
        .get_mut(buttons)
        .unwrap()
        .set_layout(Box::new(Row::new().spacing(8)));
    cache
        .get_mut(0)
        .unwrap()
        .set_layout(Box::new(Column::new().spacing(6).margins(Margin::all(10))));

    Dialog {
        cache,
        backend: HeadlessBackend::new(size(w, 200)),
        field,
        capped,
        buttons,
        ok,
        cancel,
    }
}

impl Dialog {
    /// Draws one frame, returning the engine events it queued.
    fn frame(&mut self) -> Vec<EngineEvent> {
        self.cache.draw_loop(&mut self.backend);
        self.cache
            .take_pending_events()
            .into_iter()
            .filter_map(|event| match event {
                Event::Engine(engine) => Some(engine),
                _ => None,
            })
            .collect()
    }

    /// Returns a widget's rectangle relative to its parent.
    fn rect(&self, id: WidgetId) -> Rect {
        let widget = self.cache.get(id).unwrap();
        let (origin, size) = (widget.get_origin(), widget.get_size());

        Rect::new(origin.x, origin.y, size.w, size.h)
    }
}

#[test]
fn a_column_of_fields_above_a_right_aligned_button_row() {
    let mut d = dialog(300);
    let events = d.frame();

    assert_eq!(d.rect(d.field), Rect::new(10, 10, 280, 20));
    assert_eq!(d.rect(d.capped), Rect::new(10, 36, 150, 20));
    // The button row sits at the bottom: 200 - 10 margin - 24 tall.
    assert_eq!(d.rect(d.buttons), Rect::new(10, 166, 280, 24));
    assert_eq!(d.rect(d.ok), Rect::new(112, 0, 80, 24));
    assert_eq!(d.rect(d.cancel), Rect::new(200, 0, 80, 24));
    assert!(events.contains(&EngineEvent::LayoutChanged { container: 0 }));
    assert!(events.contains(&EngineEvent::LayoutChanged {
        container: d.buttons
    }));
}

#[test]
fn resizing_the_window_rearranges_and_keeps_buttons_right_aligned() {
    let mut d = dialog(300);

    d.frame();
    d.cache.get_mut(0).unwrap().set_size(size(500, 260));

    let events = d.frame();

    assert_eq!(d.rect(d.field).w, 480);
    assert_eq!(d.rect(d.capped).w, 150, "the maximum is kept while growing");
    assert_eq!(d.rect(d.buttons), Rect::new(10, 226, 480, 24));
    assert_eq!(d.rect(d.cancel).x, 400);
    assert!(events.contains(&EngineEvent::LayoutChanged {
        container: d.buttons
    }));
}

#[test]
fn shrinking_the_window_never_goes_below_the_minimum() {
    let mut d = dialog(300);

    d.frame();
    d.cache.get_mut(0).unwrap().set_size(size(70, 200));
    d.frame();

    assert_eq!(
        d.rect(d.capped).w,
        60,
        "the minimum is kept across the axis"
    );
    assert_eq!(d.rect(d.ok).w, 80, "buttons keep their minimum width");

    let hint = d.cache.measured_hint(0).unwrap();

    assert_eq!(hint.min.w, 8 + 80 + 8 + 80 + 20);
}
