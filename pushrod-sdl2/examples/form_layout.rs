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

//! A settings form built with `FormLayout`: right-aligned labels in a column as wide as the
//! widest label, fields that fill the rest, and a full-width check box.
//!
//! - **Wider / Narrower** resizes the form.  Wide, the fields stretch to the new width.
//!   Narrow, every label moves above its field.
//! - **Left labels / Right labels** switches the label alignment.
//! - **Click a label** to focus its field.
//! - The status line shows each `LayoutChanged` and `FocusChanged` engine event.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example form_layout`.  All sample data is
//! fictional.

use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::size;
use pushrod::layout::{Align, FormLayout, Margin};
use pushrod::widget::{Widget, WidgetId};
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The form's two widths, in pixels.
const NARROW: u32 = 240;
const WIDE: u32 = 520;

/// The form's height, in pixels.
const FORM_H: u32 = 300;

/// The settings rows: a label and the field's starting text.
const ROWS: [(&str, &str); 4] = [
    ("Display name:", "Jane Doe"),
    ("Email:", "jane.doe@example.com"),
    ("Server address:", "mail.example.com"),
    ("Port:", "993"),
];

/// Application state: the form's width and label alignment.
#[derive(Default)]
struct AppState {
    wide: bool,
    left_labels: bool,
}

/// Builds the form layout with the rows and a label alignment.
///
/// # Arguments
///
/// * `rows` - The label and field of each row.
/// * `remember` - The full-width check box after the rows.
/// * `left` - `true` for left-aligned labels; `false` for right-aligned (the Mac style).
///
/// # Returns
///
/// The layout.
fn form_layout(rows: &[(WidgetId, WidgetId)], remember: WidgetId, left: bool) -> FormLayout {
    let align = if left { Align::Start } else { Align::End };
    let mut form = FormLayout::new()
        .spacing(8)
        .column_spacing(8)
        .margins(Margin::all(12))
        .label_align(align);

    for &(label, field) in rows {
        form.add_row(label, field);
    }

    form.add_full_row(remember);
    form
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(size(560, 400), 30);
    let toggle = engine.add_widget(Button::new("Wider").at(20, 20).size(120, 24));
    let align = engine.add_widget(Button::new("Left labels").at(150, 20).size(120, 24));
    let status = engine.add_widget(
        Text::new("LayoutChanged and FocusChanged events appear here.")
            .at(20, 372)
            .size(520, 20),
    );
    let mut panel = Panel::new().at(20, 60).size(NARROW, FORM_H);

    panel.set_color(Color::RGBA(225, 225, 225, 255));

    let form = engine.add_widget(panel);
    let mut rows = Vec::with_capacity(ROWS.len());

    for (label, text) in ROWS {
        let label = engine.add_child_widget(form, Text::new(label));
        let field = engine.add_child_widget(form, TextField::new().size(140, 24).with_text(text));

        rows.push((label, field));
    }

    let remember = engine.add_child_widget(form, CheckBox::new("Remember password").checked(true));

    if let Some(widget) = engine.widgets().get_mut(form) {
        widget.set_layout(Box::new(form_layout(&rows, remember, false)));
    }

    engine.on_click(toggle, move |state, ui| {
        state.wide = !state.wide;

        let width = if state.wide { WIDE } else { NARROW };

        if let Some(widget) = ui.widgets().get_mut(form) {
            widget.set_size(size(width, FORM_H));
        }

        if let Some(button) = ui.get_mut::<Button>(toggle) {
            button.set_text(if state.wide { "Narrower" } else { "Wider" }.to_string());
        }
    });

    let layout_rows = rows.clone();

    engine.on_click(align, move |state, ui| {
        state.left_labels = !state.left_labels;

        if let Some(widget) = ui.widgets().get_mut(form) {
            widget.set_layout(Box::new(form_layout(
                &layout_rows,
                remember,
                state.left_labels,
            )));
        }

        if let Some(button) = ui.get_mut::<Button>(align) {
            let text = if state.left_labels {
                "Right labels"
            } else {
                "Left labels"
            };

            button.set_text(text.to_string());
        }
    });

    engine.on_engine_event(move |_state, ui, event| {
        let line = match event {
            EngineEvent::LayoutChanged { container } => {
                format!("LayoutChanged {{ container: {container} }}")
            }
            EngineEvent::FocusChanged { old, new } => {
                format!("FocusChanged {{ old: {old:?}, new: {new:?} }}")
            }
            _ => return,
        };

        if let Some(text) = ui.get_mut::<Text>(status) {
            text.set_text(line);
        }
    });

    pushrod_sdl2::run(
        engine,
        WindowSettings::new("pushrod: form layout", size(560, 400)),
        AppState::default(),
    )?;

    Ok(())
}
