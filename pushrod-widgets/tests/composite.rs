// Composite widgets built from standard widgets: an application finds a part by name and
// changes its font, color, or text through `ui.get_mut`, and the part is repainted.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::color::{Color, FontStyle};
use pushrod::composite::{Composite, PartList, PartsBuilder};
use pushrod::engine::Engine;
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore, WidgetId};
use pushrod_widgets::button::Button;
use pushrod_widgets::text::Text;

/// A message above an OK button.
struct Notice {
    core: WidgetCore,
    parts: PartList,
}

impl Widget for Notice {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }

    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Notice {
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("message", Text::new("Saved."));
        parts.add("button:0", Button::new("OK"));
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let mut placed = Vec::new();

        if let Some(message) = self.parts.get("message") {
            placed.push((message, Rect::new(10, 10, 180, 20)));
        }

        for (index, button) in self.parts.list("button").into_iter().enumerate() {
            placed.push((button, Rect::new(10 + index as i32 * 70, 40, 60, 20)));
        }

        placed
    }
}

fn notice() -> Notice {
    Notice {
        core: WidgetCore::new(point(0, 0), size(200, 70)),
        parts: PartList::new(),
    }
}

fn click_at(x: i32, y: i32) -> [InputEvent; 3] {
    [
        InputEvent::MouseMove {
            position: point(x, y),
        },
        InputEvent::MouseDown {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
        InputEvent::MouseUp {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
    ]
}

#[test]
fn changing_a_part_through_ui_get_mut_repaints_it() {
    let mut engine: Engine<()> = Engine::new(size(200, 70), 30);
    let notice_id = engine.add_widget(notice());
    let message = engine.widgets().part(notice_id, "message").unwrap();
    let ok = engine.widgets().part(notice_id, "button:0").unwrap();
    let mut backend = HeadlessBackend::new(size(200, 70));

    engine.widgets().draw_loop(&mut backend);
    backend.painted.clear();

    engine.on_click(ok, move |_state, ui| {
        let text = ui.get_mut::<Text>(message).unwrap();

        text.set_font(Font::new("", 18, FontStyle::BOLD));
        text.set_font_color(Color::RED);
        text.set_text("Saved twice.".to_string());
    });

    for event in click_at(20, 45) {
        engine.dispatch(event, &mut ());
    }

    let text = engine.widgets().get_as::<Text>(message).unwrap();

    assert_eq!(text.get_text(), "Saved twice.");
    assert_eq!(text.get_font_color(), Color::RED);
    assert_eq!(text.get_font().size(), 18);
    assert!(text.is_invalidated());

    engine.widgets().draw_loop(&mut backend);

    assert!(backend.painted.iter().any(|(id, _)| *id == message));
}

#[test]
fn a_button_part_is_added_and_laid_out_beside_the_first() {
    let mut engine: Engine<()> = Engine::new(size(200, 70), 30);
    let notice_id = engine.add_widget(notice());
    let help = engine
        .widgets()
        .insert_list_part(
            notice_id,
            "button",
            usize::MAX,
            Box::new(Button::new("Help")),
        )
        .unwrap();

    assert_eq!(engine.widgets().part(notice_id, "button:1"), Some(help));
    assert_eq!(
        engine.widgets().get(help).unwrap().get_origin(),
        point(80, 40)
    );
    assert_eq!(
        engine.widgets().get_as::<Button>(help).unwrap().get_text(),
        "Help"
    );
}
