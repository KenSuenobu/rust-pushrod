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

//! A dialog built with rows and columns: a `Column` of fields above a `Row` of buttons, with
//! a `Spacer` pushing OK and Cancel to the right.
//!
//! - **Wider / Narrower** resizes the dialog.  The fields stretch to the new width, the
//!   "Notes" field stops at its 260 pixel maximum, and OK and Cancel stay at the right.  At the
//!   narrow width the buttons keep their minimum size.
//! - The status line shows each `LayoutChanged` engine event.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example row_and_column`.  All sample data is
//! fictional.

use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::size;
use pushrod::layout::{Align, Alignment, Column, LayoutParams, Margin, Row, SizeHint, Spacer};
use pushrod::widget::{Widget, WidgetId};
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The dialog's two widths, in pixels.
const NARROW: u32 = 300;
const WIDE: u32 = 520;

/// The dialog's height, in pixels.
const DIALOG_H: u32 = 200;

/// Application state: whether the dialog is at its wide size.
#[derive(Default)]
struct AppState {
    wide: bool,
}

/// Layout parameters that make a child fill the column's width.
///
/// # Returns
///
/// Horizontal fill, top alignment, no stretch, and no margin.
fn fill_width() -> LayoutParams {
    LayoutParams {
        alignment: Alignment::new(Align::Fill, Align::Start),
        ..LayoutParams::default()
    }
}

/// Adds a text field that fills the column's width.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `dialog` - The column the field goes in.
/// * `label` - The field's label.
/// * `text` - The field's starting text.
///
/// # Returns
///
/// The field's ID.
fn add_field(engine: &mut Engine<AppState>, dialog: WidgetId, label: &str, text: &str) -> WidgetId {
    let field = engine.add_child_widget(
        dialog,
        TextField::new()
            .size(200, 24)
            .with_label(label)
            .with_text(text),
    );

    if let Some(widget) = engine.widgets().get_mut(field) {
        widget.set_layout_params(fill_width());
    }

    field
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(size(560, 320), 30);
    let toggle = engine.add_widget(Button::new("Wider").at(20, 20).size(120, 24));
    let status = engine.add_widget(
        Text::new("LayoutChanged events appear here.")
            .at(20, 284)
            .size(520, 20),
    );
    let mut panel = Panel::new().at(20, 64).size(NARROW, DIALOG_H);

    panel.set_color(Color::RGBA(225, 225, 225, 255));

    let dialog = engine.add_widget(panel);

    if let Some(widget) = engine.widgets().get_mut(dialog) {
        widget.set_layout(Box::new(Column::new().spacing(8).margins(Margin::all(12))));
    }

    add_field(&mut engine, dialog, "Name:", "Jane Doe");
    add_field(&mut engine, dialog, "Email:", "jane.doe@example.com");

    let notes = add_field(&mut engine, dialog, "Notes:", "Anytown office");

    if let Some(widget) = engine.widgets().get_mut(notes) {
        widget.set_size_hint(SizeHint::new(size(120, 24), size(200, 24), size(260, 24)));
    }

    // The gap above the buttons takes the extra height, so the button row sits at the bottom.
    engine.add_child_widget(dialog, Spacer::stretch());

    let mut row = Panel::new();

    row.set_color(Color::RGBA(225, 225, 225, 255));

    let buttons = engine.add_child_widget(dialog, row);

    if let Some(widget) = engine.widgets().get_mut(buttons) {
        widget.set_layout_params(fill_width());
        widget.set_layout(Box::new(Row::new().spacing(8)));
    }

    // The spacer in the row pushes OK and Cancel to the right.
    engine.add_child_widget(buttons, Spacer::stretch());
    engine.add_child_widget(buttons, Button::new("OK").size(80, 24).default_action());
    engine.add_child_widget(buttons, Button::new("Cancel").size(80, 24).cancel_action());

    engine.on_click(toggle, move |state, ui| {
        state.wide = !state.wide;

        let width = if state.wide { WIDE } else { NARROW };

        if let Some(widget) = ui.widgets().get_mut(dialog) {
            widget.set_size(size(width, DIALOG_H));
        }

        if let Some(button) = ui.get_mut::<Button>(toggle) {
            button.set_text(if state.wide { "Narrower" } else { "Wider" }.to_string());
        }
    });

    engine.on_engine_event(move |_state, ui, event| {
        if let EngineEvent::LayoutChanged { container } = event
            && let Some(text) = ui.get_mut::<Text>(status)
        {
            text.set_text(format!("LayoutChanged {{ container: {container} }}"));
        }
    });

    pushrod_sdl2::run(
        engine,
        WindowSettings::new("pushrod: row and column", size(560, 320)),
        AppState::default(),
    )?;

    Ok(())
}
