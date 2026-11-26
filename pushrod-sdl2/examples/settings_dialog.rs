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

//! A settings dialog opened with `Ui::open_modal`, with a nested confirmation alert.
//!
//! - **Settings...** opens the dialog.  While it is open, the main window is drawn but inert:
//!   clicks, hover, and Tab do not reach it.
//! - **Save** (or Return) keeps the changes; **Cancel** (or Escape) discards them.  Both are
//!   `dismisses_dialog` buttons, so either one closes the dialog and the dismissal callback is
//!   told which one it was.
//! - **Reset...** opens a second, nested modal on top of the dialog.  **Reset** puts the check
//!   boxes back to their defaults; **Keep** leaves them.  Only the alert takes input until it
//!   closes.
//! - The status line shows the `ModalOpened`, `DialogDismissRequested`, and `ModalClosed`
//!   engine events.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example settings_dialog`.  Pass
//! `--screenshot <file.png>` to click **Settings...**, draw one frame, save it, and exit; this is
//! how the documentation screenshot is made.  All sample data is fictional.

use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::{Point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::{Widget, WidgetId};
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;

/// The window size, in pixels.
const WINDOW_W: u32 = 480;
const WINDOW_H: u32 = 320;

/// The dialog's background, which its labels share.
const DIALOG_COLOR: Color = Color::RGBA(225, 225, 225, 255);

/// The alert's background.
const ALERT_COLOR: Color = Color::RGBA(250, 235, 200, 255);

/// Where the **Settings...** button is, for the screenshot's scripted click.
const SETTINGS_BUTTON: Point = Point { x: 80, y: 68 };

/// The saved settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AppState {
    dark_theme: bool,
    autosave: bool,
}

impl Default for AppState {
    /// The default settings: light theme, autosave on.
    fn default() -> Self {
        Self {
            dark_theme: false,
            autosave: true,
        }
    }
}

/// Describes the saved settings for the main window.
///
/// # Arguments
///
/// * `state` - The saved settings.
///
/// # Returns
///
/// A one-line summary.
fn summary(state: &AppState) -> String {
    format!(
        "Theme: {}    Autosave: {}",
        if state.dark_theme { "dark" } else { "light" },
        if state.autosave { "on" } else { "off" },
    )
}

/// Adds a hidden panel to hold a dialog.
///
/// # Arguments
///
/// * `engine` - The engine to add it to.
/// * `panel` - The panel, placed and sized.
/// * `color` - Its background.
///
/// # Returns
///
/// The panel's ID.
fn add_dialog(engine: &mut Engine<AppState>, mut panel: Panel, color: Color) -> WidgetId {
    panel.set_color(color);

    let id = engine.add_widget(panel);

    if let Some(widget) = engine.widgets().get_mut(id) {
        widget.set_hidden(true);
    }

    id
}

/// Adds a label that shares its parent's background.
///
/// # Arguments
///
/// * `engine` - The engine to add it to.
/// * `parent` - The dialog that holds it.
/// * `text` - The label, placed and sized.
/// * `color` - The dialog's background.
fn add_label(engine: &mut Engine<AppState>, parent: WidgetId, mut text: Text, color: Color) {
    text.set_color(color);
    engine.add_child_widget(parent, text);
}

/// Clicks **Settings...**, draws one frame with the dialog open, and saves it.
///
/// # Arguments
///
/// * `engine` - The application, built.
/// * `settings` - The window to draw into.
/// * `path` - Where to save the PNG.
///
/// # Returns
///
/// Nothing on success.
///
/// # Errors
///
/// Returns the backend's error if the window cannot be created or the image cannot be saved.
fn save_screenshot(
    mut engine: Engine<AppState>,
    settings: &WindowSettings,
    path: &str,
) -> Result<(), pushrod::error::Error> {
    let mut backend = Sdl2Backend::new(settings)?;
    let mut state = AppState::default();

    engine.widgets().draw_loop(&mut backend);

    for down in [true, false] {
        let position = SETTINGS_BUTTON;
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

        engine.dispatch(event, &mut state);
    }

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);
    backend.save_screenshot(path)
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let settings = WindowSettings::new("pushrod: settings dialog", size(WINDOW_W, WINDOW_H));
    let mut engine: Engine<AppState> = Engine::new(size(WINDOW_W, WINDOW_H), 30);

    // The main window.
    let current = engine.add_widget(
        Text::new(summary(&AppState::default()))
            .at(20, 20)
            .size(440, 20),
    );
    let open = engine.add_widget(
        Button::new("Settings...")
            .at(20, 56)
            .size(120, 24)
            .default_action(),
    );
    let status = engine.add_widget(
        Text::new("Modal engine events appear here.")
            .at(20, 292)
            .size(440, 20),
    );

    // The settings dialog.  Child origins are relative to the dialog.
    let dialog = add_dialog(
        &mut engine,
        Panel::new().at(60, 40).size(360, 200),
        DIALOG_COLOR,
    );

    add_label(
        &mut engine,
        dialog,
        Text::new("Settings").at(16, 12).size(200, 20),
        DIALOG_COLOR,
    );

    let mut dark = CheckBox::new("Dark theme").at(16, 48).size(200, 24);
    let mut autosave = CheckBox::new("Autosave every 5 minutes")
        .at(16, 80)
        .size(260, 24);

    dark.set_color(DIALOG_COLOR);
    autosave.set_color(DIALOG_COLOR);

    let dark = engine.add_child_widget(dialog, dark);
    let autosave = engine.add_child_widget(dialog, autosave);
    let reset = engine.add_child_widget(dialog, Button::new("Reset...").at(16, 160).size(90, 24));
    let cancel = engine.add_child_widget(
        dialog,
        Button::new("Cancel")
            .at(160, 160)
            .size(84, 24)
            .cancel_action()
            .dismisses_dialog(),
    );
    let save = engine.add_child_widget(
        dialog,
        Button::new("Save")
            .at(256, 160)
            .size(84, 24)
            .default_action()
            .dismisses_dialog(),
    );

    // The confirmation alert, opened on top of the dialog.
    let alert = add_dialog(
        &mut engine,
        Panel::new().at(110, 100).size(260, 110),
        ALERT_COLOR,
    );

    add_label(
        &mut engine,
        alert,
        Text::new("Reset settings to their defaults?")
            .at(16, 16)
            .size(230, 20),
        ALERT_COLOR,
    );

    let keep = engine.add_child_widget(
        alert,
        Button::new("Keep")
            .at(60, 70)
            .size(84, 24)
            .cancel_action()
            .dismisses_dialog(),
    );
    let confirm = engine.add_child_widget(
        alert,
        Button::new("Reset")
            .at(156, 70)
            .size(84, 24)
            .default_action()
            .dismisses_dialog(),
    );

    // Settings...: show the saved values, then run the dialog until Save or Cancel.
    engine.on_click(open, move |state, ui| {
        if let Some(check) = ui.get_mut::<CheckBox>(dark) {
            check.set_checked(state.dark_theme);
        }

        if let Some(check) = ui.get_mut::<CheckBox>(autosave) {
            check.set_checked(state.autosave);
        }

        let opened = ui.open_modal(dialog, move |state, ui, dismissed_by| {
            if dismissed_by != save {
                return;
            }

            state.dark_theme = ui.get::<CheckBox>(dark).is_some_and(|c| c.is_checked());
            state.autosave = ui.get::<CheckBox>(autosave).is_some_and(|c| c.is_checked());

            if let Some(text) = ui.get_mut::<Text>(current) {
                text.set_text(summary(state));
            }
        });

        if let Err(error) = opened {
            eprintln!("could not open the settings dialog: {error}");
        }
    });

    // Reset...: a nested modal; only the alert takes input until Reset or Keep.
    engine.on_click(reset, move |_state, ui| {
        let opened = ui.open_modal(alert, move |_state, ui, dismissed_by| {
            if dismissed_by != confirm {
                return;
            }

            let defaults = AppState::default();

            if let Some(check) = ui.get_mut::<CheckBox>(dark) {
                check.set_checked(defaults.dark_theme);
            }

            if let Some(check) = ui.get_mut::<CheckBox>(autosave) {
                check.set_checked(defaults.autosave);
            }
        });

        if let Err(error) = opened {
            eprintln!("could not open the reset alert: {error}");
        }
    });

    engine.on_engine_event(move |_state, ui, event| {
        let name = |id: WidgetId| match id {
            id if id == dialog => "settings dialog",
            id if id == alert => "reset alert",
            id if id == save => "Save",
            id if id == cancel => "Cancel",
            id if id == confirm => "Reset",
            id if id == keep => "Keep",
            _ => "widget",
        };
        let line = match event {
            EngineEvent::ModalOpened { root } => format!("ModalOpened: {}", name(*root)),
            EngineEvent::DialogDismissRequested { id } => {
                format!("DialogDismissRequested: {}", name(*id))
            }
            EngineEvent::ModalClosed { root, dismissed_by } => {
                format!("ModalClosed: {} by {}", name(*root), name(*dismissed_by))
            }
            _ => return,
        };

        if let Some(text) = ui.get_mut::<Text>(status) {
            text.set_text(line);
        }
    });

    let args: Vec<String> = std::env::args().collect();

    if let Some(index) = args.iter().position(|a| a == "--screenshot")
        && let Some(path) = args.get(index + 1)
    {
        return save_screenshot(engine, &settings, path);
    }

    pushrod_sdl2::run(engine, settings, AppState::default())?;

    Ok(())
}
