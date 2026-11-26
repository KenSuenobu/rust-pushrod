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

//! Composite widgets: a button bar built from a title and a row of buttons, all of them
//! ordinary widgets the application can find, restyle, add, and remove.
//!
//! - **Add button** / **Insert first** / **Remove last** change the bar's list of buttons; the
//!   bar lays the row out again each time.
//! - **Restyle title** changes the title part's font, color, and text through `ui.get_mut`.
//! - Clicking a button in the bar runs the callback registered on that part, and the bar
//!   translates the click into its own `BarPressed` event.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example composite`.

use pushrod::color::{Color, FontStyle};
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, Event, EventCtx, EventTarget};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore, WidgetId};
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::button::Button;
use pushrod_widgets::text::Text;

/// The height of the bar's title row, in pixels.
const TITLE_HEIGHT: u32 = 24;
/// The width of each button in the bar, in pixels.
const BUTTON_WIDTH: u32 = 90;
/// The space around and between the bar's buttons, in pixels.
const GAP: i32 = 8;

/// The bar's own event, posted when one of its buttons is clicked.
struct BarPressed {
    /// The button's position in the bar.
    index: usize,
    /// The button's part ID.
    part: WidgetId,
}

/// The bar's structure, by role.
struct ButtonBarParts {
    /// The title text.
    title: WidgetId,
    /// The buttons, left to right.
    buttons: Vec<WidgetId>,
}

/// A titled row of buttons, made of a `Text` part (`"title"`) and `Button` parts
/// (`"button:0"`, `"button:1"`, ...).
struct ButtonBar {
    core: WidgetCore,
    parts: PartList,
    title: String,
    labels: Vec<String>,
}

impl ButtonBar {
    /// Creates a bar.
    ///
    /// # Arguments
    ///
    /// * `title` - The title shown above the buttons.
    /// * `labels` - The first buttons' labels.
    ///
    /// # Returns
    ///
    /// The bar, at (20, 20), 560 pixels wide.
    fn new(title: &str, labels: &[&str]) -> Self {
        let mut bar = Self {
            core: WidgetCore::new(point(20, 20), size(560, 70)),
            parts: PartList::new(),
            title: title.to_string(),
            labels: labels.iter().map(|label| label.to_string()).collect(),
        };

        bar.set_color(Color::RGB(220, 220, 220));
        bar
    }
}

impl Widget for ButtonBar {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
        paint::border(p, self.get_size(), Color::GRAY, 1);
    }

    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for ButtonBar {
    fn build(&mut self, parts: &mut PartsBuilder) {
        let mut title = Text::new(self.title.clone());

        title.set_color(self.get_color());
        parts.add("title", title);

        for (index, label) in self.labels.iter().enumerate() {
            parts.add(format!("button:{index}"), Button::new(label.clone()));
        }
    }

    fn part_list(&self) -> &PartList {
        &self.parts
    }

    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    /// The title across the top, then the buttons left to right.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let width = self.get_size().w;
        let mut placed = Vec::new();

        if let Some(title) = self.parts.get("title") {
            placed.push((
                title,
                Rect::new(GAP, 4, width - 2 * GAP as u32, TITLE_HEIGHT - 4),
            ));
        }

        for (index, button) in self.parts.list("button").into_iter().enumerate() {
            let x = GAP + index as i32 * (BUTTON_WIDTH as i32 + GAP);

            placed.push((
                button,
                Rect::new(x, TITLE_HEIGHT as i32 + GAP, BUTTON_WIDTH, 24),
            ));
        }

        placed
    }

    /// A button's click becomes the bar's own `BarPressed`.
    fn handle_part_event(&mut self, part: WidgetId, event: &Event, ctx: &mut EventCtx) {
        if !matches!(event, Event::Clicked { .. }) {
            return;
        }

        if let Some(index) = self.parts.list("button").iter().position(|&id| id == part) {
            ctx.post(EventTarget::ApplicationOnly, BarPressed { index, part });
        }
    }
}

impl TypedParts for ButtonBar {
    type Parts = ButtonBarParts;

    fn typed_parts(&self) -> ButtonBarParts {
        ButtonBarParts {
            title: self.parts.get("title").unwrap_or(self.get_id()),
            buttons: self.parts.list("button"),
        }
    }
}

/// Application state, passed to every callback.
#[derive(Default)]
struct AppState {
    /// How many buttons were added so far, for their labels.
    added: u32,
    /// Whether the title is in its restyled look.
    restyled: bool,
}

/// Adds a control button under the bar.
fn control(engine: &mut Engine<AppState>, label: &str, x: i32) -> WidgetId {
    engine.add_widget(Button::new(label).at(x, 110).size(130, 24))
}

/// Adds a gray status line.
fn status_line(engine: &mut Engine<AppState>, text: &str, y: i32) -> WidgetId {
    engine.add_widget(
        Text::new(text)
            .at(20, y)
            .size(560, 20)
            .font_color(Color::RGB(60, 60, 60)),
    )
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(size(600, 240), 30);
    let bar = engine.add_widget(ButtonBar::new("Button bar", &["Open", "Save"]));
    let parts = engine
        .widgets()
        .get_as::<ButtonBar>(bar)
        .map(|bar| bar.typed_parts())
        .ok_or(pushrod::error::Error::UnknownWidget { id: bar })?;

    let add = control(&mut engine, "Add button", 20);
    let insert = control(&mut engine, "Insert first", 160);
    let remove = control(&mut engine, "Remove last", 300);
    let restyle = control(&mut engine, "Restyle title", 440);
    let status = status_line(&mut engine, "Click a button in the bar.", 160);
    let engine_status = status_line(&mut engine, "Engine events appear here.", 190);

    // A callback on a part's ID works like on any widget.
    if let Some(&open) = parts.buttons.first() {
        engine.on_click(open, move |_state, ui| {
            if let Some(text) = ui.get_mut::<Text>(status) {
                text.set_text("on_click on part \"button:0\" ran".to_string());
            }
        });
    }

    // The bar's translated event names the button by position.
    engine.on_custom::<BarPressed, _>(move |_state, ui, _source, pressed| {
        let label = ui
            .get::<Button>(pressed.part)
            .map(|button| button.get_text())
            .unwrap_or_default();

        if let Some(text) = ui.get_mut::<Text>(status) {
            text.set_text(format!(
                "BarPressed: button {} (\"{label}\")",
                pressed.index
            ));
        }
    });

    engine.on_click(add, move |state, ui| {
        state.added += 1;

        let label = format!("Item {}", state.added);
        let _ =
            ui.widgets()
                .insert_list_part(bar, "button", usize::MAX, Box::new(Button::new(label)));
    });

    engine.on_click(insert, move |state, ui| {
        state.added += 1;

        let label = format!("Item {}", state.added);
        let _ = ui
            .widgets()
            .insert_list_part(bar, "button", 0, Box::new(Button::new(label)));
    });

    engine.on_click(remove, move |_state, ui| {
        let count = ui
            .widgets()
            .get_as::<ButtonBar>(bar)
            .map_or(0, |bar| bar.typed_parts().buttons.len());

        if count > 0 {
            let _ = ui.widgets().remove_list_part(bar, "button", count - 1);
        }
    });

    // Parts are changed through the normal API, and repaint themselves.
    engine.on_click(restyle, move |state, ui| {
        state.restyled = !state.restyled;

        if let Some(title) = ui.get_mut::<Text>(parts.title) {
            if state.restyled {
                title.set_font(Font::new("", 16, FontStyle::BOLD));
                title.set_font_color(Color::RED);
                title.set_text("Button bar (restyled through its \"title\" part)".to_string());
            } else {
                title.set_font(Font::default());
                title.set_font_color(Color::BLACK);
                title.set_text("Button bar".to_string());
            }
        }
    });

    engine.on_engine_event(move |_state, ui, event| {
        let line = match event {
            EngineEvent::WidgetAdded { id, parent } => {
                format!("WidgetAdded {{ id: {id}, parent: {parent} }}")
            }
            EngineEvent::WidgetRemoved { id, parent } => {
                format!("WidgetRemoved {{ id: {id}, parent: {parent} }}")
            }
            EngineEvent::PartsArranged { id, placed } => {
                format!("PartsArranged {{ id: {id}, placed: {placed} }}")
            }
            _ => return,
        };

        if let Some(text) = ui.get_mut::<Text>(engine_status) {
            text.set_text(line);
        }
    });

    pushrod_sdl2::run(
        engine,
        WindowSettings::new("pushrod: composite widgets", size(600, 240)),
        AppState::default(),
    )?;

    Ok(())
}
