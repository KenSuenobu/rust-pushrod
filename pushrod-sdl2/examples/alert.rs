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

//! One-call alerts opened with `ui.alert`.
//!
//! - **Delete...** opens a warning with **Delete** (the default, chosen by Return) and
//!   **Cancel** (chosen by Escape).  The status line shows the index the completion closure got.
//! - **Customized...** opens the same warning restyled through its parts: a bold red message,
//!   and an extra **Help** button added with `add_button`, placed at the lower left.
//! - **Quit...** asks for confirmation before quitting.
//! - While an alert is open the rest of the window is drawn, darkened, but inert.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example alert`.  Pass `--screenshot <file.png>` to
//! click **Delete...**, draw one frame, save it, and exit; add `custom` to click
//! **Customized...** instead.  This is how the documentation screenshots are made.  All sample
//! data is fictional.

use pushrod::color::{Color, FontStyle};
use pushrod::engine::{Engine, Ui};
use pushrod::font::Font;
use pushrod::geometry::{Point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::WidgetId;
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::alert::{AlertExt, AlertKind};
use pushrod_widgets::button::Button;
use pushrod_widgets::text::Text;

/// The window size, in pixels.
const WINDOW_W: u32 = 480;
const WINDOW_H: u32 = 320;

/// Where the **Delete...** button is, for the screenshot's scripted click.
const DELETE_BUTTON: Point = Point { x: 80, y: 68 };

/// Where the **Customized...** button is, for the screenshot's scripted click.
const CUSTOM_BUTTON: Point = Point { x: 220, y: 68 };

/// Opens the delete warning and reports the choice on the status line.
///
/// # Arguments
///
/// * `ui` - The callback's `Ui`.
/// * `status` - The status line's ID.
/// * `customize` - `true` to restyle the message and add a **Help** button.
fn open_delete_alert(ui: &mut Ui<()>, status: WidgetId, customize: bool) {
    let mut alert = ui.alert(
        AlertKind::Warning,
        "Delete \"report.txt\"?",
        "This cannot be undone.",
        &["Delete", "Cancel"],
        0,
        move |_state, ui, choice| {
            let line = match choice {
                0 => "Chose 0: Delete".to_string(),
                1 => "Chose 1: Cancel".to_string(),
                other => format!("Chose {other}: Help"),
            };

            if let Some(text) = ui.get_mut::<Text>(status) {
                text.set_text(line);
            }
        },
    );

    if customize {
        if let Some(message) = ui.get_mut::<Text>(alert.parts().message) {
            message.set_font_color(Color::RED);
            message.set_font(Font::new("", 14, FontStyle::BOLD));
        }

        alert.add_button(ui, "Help");
    }
}

/// Clicks the left mouse button at a point.
///
/// # Arguments
///
/// * `engine` - The application.
/// * `position` - Where to click, in window coordinates.
fn click(engine: &mut Engine, position: Point) {
    for down in [true, false] {
        let event = if down {
            InputEvent::MouseDown {
                position,
                button: MouseButton::Left,
                clicks: 1,
            }
        } else {
            InputEvent::MouseUp {
                position,
                button: MouseButton::Left,
                clicks: 1,
            }
        };

        engine.dispatch(event, &mut ());
    }
}

/// Clicks **Delete...** (or **Customized...**), draws one frame, and saves it.
///
/// # Arguments
///
/// * `engine` - The application, built.
/// * `settings` - The window to draw into.
/// * `path` - Where to save the PNG.
/// * `custom` - `true` to open the customized alert.
///
/// # Returns
///
/// Nothing on success.
///
/// # Errors
///
/// Returns the backend's error if the window cannot be created or the image cannot be saved.
fn save_screenshot(
    mut engine: Engine,
    settings: &WindowSettings,
    path: &str,
    custom: bool,
) -> Result<(), pushrod::error::Error> {
    let mut backend = Sdl2Backend::new(settings)?;

    engine.widgets().draw_loop(&mut backend);
    click(
        &mut engine,
        if custom { CUSTOM_BUTTON } else { DELETE_BUTTON },
    );
    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);
    backend.save_screenshot(path)
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let settings = WindowSettings::new("pushrod: alert", size(WINDOW_W, WINDOW_H));
    let mut engine: Engine = Engine::new(size(WINDOW_W, WINDOW_H), 30);

    engine.add_widget(
        Text::new("Each button opens an alert.")
            .at(20, 20)
            .size(440, 20),
    );

    let delete = engine.add_widget(Button::new("Delete...").at(20, 56).size(120, 24));
    let custom = engine.add_widget(Button::new("Customized...").at(160, 56).size(120, 24));
    let quit = engine.add_widget(Button::new("Quit...").at(300, 56).size(120, 24));
    let status = engine.add_widget(Text::new("No choice yet.").at(20, 292).size(440, 20));

    engine.on_click(delete, move |_state, ui| {
        open_delete_alert(ui, status, false)
    });
    engine.on_click(custom, move |_state, ui| {
        open_delete_alert(ui, status, true)
    });
    engine.on_click(quit, |_state, ui| {
        ui.alert(
            AlertKind::Question,
            "Quit the alert example?",
            "Any choices shown on the status line will be forgotten.",
            &["Quit", "Cancel"],
            0,
            |_state, ui, choice| {
                if choice == 0 {
                    ui.shutdown();
                }
            },
        );
    });

    let args: Vec<String> = std::env::args().collect();

    if let Some(index) = args.iter().position(|a| a == "--screenshot")
        && let Some(path) = args.get(index + 1)
    {
        let custom = args.get(index + 2).map(String::as_str) == Some("custom");

        return save_screenshot(engine, &settings, path, custom);
    }

    pushrod_sdl2::run(engine, settings, ())?;

    Ok(())
}
