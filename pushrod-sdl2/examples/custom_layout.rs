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

//! A custom layout manager: `Flow` places its children left to right and wraps to a new row
//! when the next one does not fit.  The application wrote it by implementing `Layout`.
//!
//! - **Add label** adds a label; the labels are measured through the text layout, so each is as
//!   wide as its text.
//! - **Hide/show first** toggles the first label; the others move to fill the gap without being
//!   repainted.
//! - **Longer text** makes the first label longer; it is resized and repainted, and the rest move.
//! - The status line shows each `LayoutChanged` engine event.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example custom_layout`.

use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::{Rect, Size, size};
use pushrod::layout::{ChildInfo, Layout, SizeHint};
use pushrod::widget::WidgetId;
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;

/// The space between labels, in pixels.
const GAP: u32 = 6;

/// Application state: how many labels were added.
#[derive(Default)]
struct AppState {
    labels: u32,
    long: bool,
    hidden: bool,
}

/// Places children left to right, starting a new row when the next child would pass the right edge.
#[derive(Debug)]
struct Flow;

impl Layout for Flow {
    /// Reports the width of the widest child and the height of one row as the minimum, and
    /// everything on one row as the preferred size.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children.
    /// * `_available` - The container's size; not used.
    ///
    /// # Returns
    ///
    /// The container's hint.
    fn measure(&self, children: &[ChildInfo], _available: Size) -> SizeHint {
        let widest = children
            .iter()
            .map(|c| c.hint.preferred.w)
            .max()
            .unwrap_or(0);
        let tallest = children
            .iter()
            .map(|c| c.hint.preferred.h)
            .max()
            .unwrap_or(0);
        let total: u32 = children.iter().map(|c| c.hint.preferred.w + GAP).sum();

        SizeHint::new(
            size(widest, tallest),
            size(total, tallest),
            size(pushrod::layout::UNBOUNDED, pushrod::layout::UNBOUNDED),
        )
    }

    /// Places each child at its preferred size, wrapping rows at the container's width.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children.
    /// * `bounds` - The container's area.
    ///
    /// # Returns
    ///
    /// One rectangle per child.
    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
        let (mut x, mut y, mut row_height) = (0u32, 0u32, 0u32);

        children
            .iter()
            .map(|child| {
                let Size { w, h } = child.hint.preferred;

                if x > 0 && x + w > bounds.w {
                    x = 0;
                    y += row_height + GAP;
                    row_height = 0;
                }

                let placed = Rect::new(x as i32, y as i32, w, h);

                x += w + GAP;
                row_height = row_height.max(h);

                placed
            })
            .collect()
    }
}

/// Adds a control button.
fn control(engine: &mut Engine<AppState>, label: &str, x: i32) -> WidgetId {
    engine.add_widget(Button::new(label).at(x, 20).size(130, 24))
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(size(480, 240), 30);
    let add = control(&mut engine, "Add label", 20);
    let hide = control(&mut engine, "Hide/show first", 160);
    let longer = control(&mut engine, "Longer text", 300);
    let status = engine.add_widget(
        Text::new("LayoutChanged events appear here.")
            .at(20, 200)
            .size(440, 20),
    );
    let area = engine.add_widget(Panel::new().at(20, 60).size(440, 120));

    if let Some(panel) = engine.widgets().get_mut(area) {
        panel.set_layout(Box::new(Flow));
    }

    for text in ["Open", "Save", "Print"] {
        engine.add_child_widget(area, Text::new(text));
    }

    engine.on_click(add, move |state, ui| {
        state.labels += 1;
        ui.widgets()
            .add_child(area, Box::new(Text::new(format!("Label {}", state.labels))));
    });

    engine.on_click(hide, move |state, ui| {
        state.hidden = !state.hidden;

        let first = ui.widgets().children_of(area).first().copied();

        if let Some(first) = first
            && let Some(label) = ui.widgets().get_mut(first)
        {
            label.set_hidden(state.hidden);
        }
    });

    engine.on_click(longer, move |state, ui| {
        state.long = !state.long;

        let first = ui.widgets().children_of(area).first().copied();

        if let Some(first) = first
            && let Some(label) = ui.get_mut::<Text>(first)
        {
            label.set_text(
                if state.long {
                    "Open a longer document"
                } else {
                    "Open"
                }
                .to_string(),
            );
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
        WindowSettings::new("pushrod: custom layout", size(480, 240)),
        AppState::default(),
    )?;

    Ok(())
}
