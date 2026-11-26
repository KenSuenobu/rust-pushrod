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

//! A resizable, high-DPI window whose root `Widget` has a column layout.
//!
//! - **Drag a window edge.**  The root follows the window, so the fields stretch to the new
//!   width and OK and Cancel stay at the bottom right.  The window cannot be made smaller than
//!   the layout's minimum size.
//! - **Move the window to a display with another pixel density** (a Retina laptop screen and a
//!   standard monitor).  Every `Widget` repaints at the new scale, so text and the one-pixel
//!   borders stay sharp.
//! - The status line shows the window size, the scale factor, and the minimum size, from the
//!   `LayoutChanged`, `ScaleFactorChanged` and `MinimumWindowSizeChanged` engine events.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example resizable`.  All sample data is fictional.

use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::Size;
use pushrod::layout::{Align, Alignment, Column, LayoutParams, Margin, Row, Spacer};
use pushrod::widget::{ROOT_WIDGET_ID, Widget, WidgetId};
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The window's starting size, in logical pixels.
const START: Size = Size { w: 480, h: 320 };

/// What the status line shows; it is only rewritten when this changes, so updating it never
/// starts another layout pass by itself.
#[derive(Default)]
struct AppState {
    /// The window size in logical pixels.
    window: Size,
    /// The scale factor in percent.
    scale_percent: u32,
    /// The minimum window size, once the layout has measured it.
    minimum: Option<Size>,
    /// The text last shown.
    shown: String,
}

impl AppState {
    /// Describes the window for the status line.
    ///
    /// # Returns
    ///
    /// For example `480 x 320 at 200%, minimum 304 x 196`.
    fn describe(&self) -> String {
        let minimum = self.minimum.map_or(String::from("not measured"), |m| {
            format!("{} x {}", m.w, m.h)
        });

        format!(
            "{} x {} at {}%, minimum {}",
            self.window.w, self.window.h, self.scale_percent, minimum
        )
    }
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

/// Adds a widget to the root's column, filling its width.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `widget` - The widget to add.
///
/// # Returns
///
/// The widget's ID.
fn add_filling<W: Widget>(engine: &mut Engine<AppState>, widget: W) -> WidgetId {
    let id = engine.add_widget(widget);

    if let Some(widget) = engine.widgets().get_mut(id) {
        widget.set_layout_params(fill_width());
    }

    id
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(START, 30);

    if let Some(root) = engine.widgets().get_mut(ROOT_WIDGET_ID) {
        root.set_layout(Box::new(Column::new().spacing(8).margins(Margin::all(16))));
    }

    let status = add_filling(&mut engine, Text::new("Measuring...").size(280, 20));

    add_filling(
        &mut engine,
        TextField::new()
            .size(280, 24)
            .with_label("Name:")
            .with_text("Jane Doe"),
    );
    add_filling(
        &mut engine,
        TextField::new()
            .size(280, 24)
            .with_label("City:")
            .with_text("Anytown"),
    );

    // The gap takes the extra height, so the button row sits at the bottom of the window.
    engine.add_widget(Spacer::stretch());

    let buttons = add_filling(&mut engine, Panel::new());

    if let Some(widget) = engine.widgets().get_mut(buttons) {
        widget.set_layout(Box::new(Row::new().spacing(8)));
    }

    engine.add_child_widget(buttons, Spacer::stretch());
    engine.add_child_widget(buttons, Button::new("OK").size(80, 24).default_action());
    engine.add_child_widget(buttons, Button::new("Cancel").size(80, 24).cancel_action());

    engine.on_engine_event(move |state, ui, event| {
        match event {
            EngineEvent::LayoutChanged { container } if *container == ROOT_WIDGET_ID => {
                if let Some(root) = ui.widgets().get(ROOT_WIDGET_ID) {
                    state.window = root.get_size();
                }
            }
            EngineEvent::ScaleFactorChanged { new_percent, .. } => {
                state.scale_percent = *new_percent;
            }
            EngineEvent::MinimumWindowSizeChanged { size } => state.minimum = Some(*size),
            _ => return,
        }

        let text = state.describe();

        if text != state.shown
            && let Some(label) = ui.get_mut::<Text>(status)
        {
            label.set_text(text.clone());
            state.shown = text;
        }
    });

    let mut settings = WindowSettings::new("pushrod: resizable", START);

    settings.resizable = true;

    // The engine starts at 100%; on a high-DPI display ScaleFactorChanged arrives as the loop
    // starts.
    pushrod_sdl2::run(
        engine,
        settings,
        AppState {
            scale_percent: 100,
            ..AppState::default()
        },
    )?;

    Ok(())
}
