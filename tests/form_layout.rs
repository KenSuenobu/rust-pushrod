//! Form layout through the widget cache: aligned labels, fields that stretch as the window is
//! resized, labels that move above their fields when it gets narrow, and label clicks that focus
//! their fields.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::event::{EngineEvent, Event};
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::layout::{Align, FormLayout, Margin};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, WidgetId};

/// The widgets of the test form.
struct Settings {
    cache: WidgetCache,
    backend: HeadlessBackend,
    name_label: WidgetId,
    name: WidgetId,
    server_label: WidgetId,
    server: WidgetId,
    remember: WidgetId,
}

/// Adds a plain child of a size under the root.
fn add(cache: &mut WidgetCache, w: u32, h: u32) -> WidgetId {
    cache.add_child(0, Box::new(RootWidget::new(size(w, h))))
}

/// Adds a focusable field of a size under the root.
fn add_field(cache: &mut WidgetCache, w: u32, h: u32) -> WidgetId {
    let id = add(cache, w, h);

    cache.get_mut(id).unwrap().set_behavior(Behavior::FOCUSABLE);

    id
}

/// Builds a window `w` pixels wide holding a settings form: "Name" (40 wide) and "Server
/// address" (100 wide) labels for two 120 pixel fields, then a full-width check box.
fn settings(w: u32, align: Align) -> Settings {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(w, 200))));

    let name_label = add(&mut cache, 40, 16);
    let name = add_field(&mut cache, 120, 24);
    let server_label = add(&mut cache, 100, 16);
    let server = add_field(&mut cache, 120, 24);
    let remember = add(&mut cache, 150, 20);
    let mut form = FormLayout::new()
        .spacing(6)
        .column_spacing(8)
        .margins(Margin::all(10))
        .label_align(align);

    form.add_row(name_label, name)
        .add_row(server_label, server)
        .add_full_row(remember);
    cache.get_mut(0).unwrap().set_layout(Box::new(form));

    Settings {
        cache,
        backend: HeadlessBackend::new(size(w, 200)),
        name_label,
        name,
        server_label,
        server,
        remember,
    }
}

impl Settings {
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

    /// Clicks the middle of a widget.
    fn click(&mut self, id: WidgetId) {
        let r = self.rect(id);
        let position = point(r.x + r.w as i32 / 2, r.y + r.h as i32 / 2);

        self.cache.handle_event(InputEvent::MouseDown {
            position,
            button: MouseButton::Left,
            clicks: 1,
        });
        self.cache.handle_event(InputEvent::MouseUp {
            position,
            button: MouseButton::Left,
            clicks: 1,
        });
    }
}

#[test]
fn labels_align_to_the_widest_label() {
    let mut s = settings(400, Align::End);
    let events = s.frame();

    // The label column is 100 wide; "Name" is right-aligned in it and centered on its field.
    assert_eq!(s.rect(s.name_label), Rect::new(70, 14, 40, 16));
    assert_eq!(s.rect(s.server_label), Rect::new(10, 44, 100, 16));
    assert_eq!(s.rect(s.name).x, 118);
    assert_eq!(s.rect(s.server).x, 118);
    assert!(events.contains(&EngineEvent::LayoutChanged { container: 0 }));

    let mut left = settings(400, Align::Start);

    left.frame();
    assert_eq!(left.rect(left.name_label).x, 10);
}

#[test]
fn fields_stretch_with_the_window() {
    let mut s = settings(400, Align::End);

    s.frame();
    assert_eq!(s.rect(s.name).w, 272);

    s.cache.get_mut(0).unwrap().set_size(size(600, 200));

    let events = s.frame();

    assert_eq!(s.rect(s.name).w, 472);
    assert_eq!(s.rect(s.server).w, 472);
    assert_eq!(s.rect(s.remember), Rect::new(10, 70, 580, 20));
    assert!(events.contains(&EngineEvent::LayoutChanged { container: 0 }));
}

#[test]
fn narrow_mode_puts_labels_above_fields() {
    let mut s = settings(400, Align::End);

    s.frame();
    // Side by side needs 100 + 8 + 120 = 228 inside the margins.
    s.cache.get_mut(0).unwrap().set_size(size(247, 200));
    s.frame();

    assert_eq!(s.rect(s.name_label), Rect::new(10, 10, 40, 16));
    assert_eq!(s.rect(s.name), Rect::new(10, 26, 227, 24));
    assert_eq!(s.rect(s.server_label), Rect::new(10, 56, 100, 16));
    assert_eq!(s.rect(s.server), Rect::new(10, 72, 227, 24));

    let hint = s.cache.measured_hint(0).unwrap();

    assert_eq!(hint.min.w, 150 + 20, "the check box is the widest minimum");
    assert_eq!(hint.preferred.w, 228 + 20);

    s.cache.get_mut(0).unwrap().set_size(size(248, 200));
    s.frame();

    assert_eq!(
        s.rect(s.name).x,
        118,
        "wide enough again: back side by side"
    );
}

#[test]
fn label_click_focuses_its_field() {
    let mut s = settings(400, Align::End);

    s.frame();
    s.click(s.server_label);
    assert_eq!(s.cache.focused(), Some(s.server));

    s.click(s.name_label);
    assert_eq!(s.cache.focused(), Some(s.name));

    // A child that is not a label leaves the focus where it is.
    s.click(s.remember);
    assert_eq!(s.cache.focused(), Some(s.name));
}

#[test]
fn label_click_skips_a_field_that_cannot_take_focus() {
    let mut s = settings(400, Align::End);

    s.frame();
    s.cache.get_mut(s.server).unwrap().set_enabled(false);
    s.click(s.server_label);

    assert_eq!(s.cache.focused(), None);
}
