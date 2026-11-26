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

//! The reference application: a contact-card editor built only with Pushrod.
//!
//! - **The list** on the left is a `Container` of radio buttons, one per contact.  Click a
//!   name, or Tab to the list and use the arrow keys, to show that contact in the form.
//! - **The form** has text fields with validation (a required name, an e-mail address checked
//!   as it is typed, a telephone mask), radio buttons, check boxes, and a slider.  While the
//!   form cannot be saved, the message under it says why and **Save** is disabled.
//! - **Save** is the default button (Return); **Revert** is the cancel button (Escape).
//! - **New** adds a contact; **Delete...** asks for confirmation in an alert first.
//! - **Resize the window**: every container has a layout, so the list grows taller and the
//!   fields grow wider.  On a high-DPI display everything is drawn at the display's resolution;
//!   the lower right corner shows the window size and scale factor.
//! - Everything works from the keyboard: Tab and Shift-Tab move between the list, the fields,
//!   the radio group, the check boxes, the slider, and the buttons; Space presses the focused
//!   button.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example contacts`.  Pass
//! `--screenshot <file.png>` to draw one frame, save it, and exit; add `invalid`, `delete`, or
//! `resized` after the file name to capture that screen.  Pass `--screenshots <directory>` to
//! save every screen there as `contacts-<screen>.png`; this is how the documentation
//! screenshots are made.  All sample data is fictional.

mod key_relay;
mod model;
mod window;

use model::ContactBook;
use pushrod_sdl2::WindowSettings;
use window::{App, Screen};

pub fn main() -> Result<(), pushrod::error::Error> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(index) = args.iter().position(|a| a == "--screenshot")
        && let Some(path) = args.get(index + 1)
    {
        let screen = Screen::from_word(args.get(index + 2).map(String::as_str));

        return window::render_screen(screen, path);
    }

    if let Some(index) = args.iter().position(|a| a == "--screenshots")
        && let Some(dir) = args.get(index + 1)
    {
        for screen in Screen::ALL {
            window::render_screen(screen, &format!("{dir}/{}.png", screen.name()))?;
        }

        return Ok(());
    }

    let app = App::new(ContactBook::default());
    let (engine, _ids) = window::build(window::WINDOW, &app);
    let mut settings = WindowSettings::new(window::TITLE, window::WINDOW);

    settings.resizable = true;
    pushrod_sdl2::run(engine, settings, app)?;

    Ok(())
}
