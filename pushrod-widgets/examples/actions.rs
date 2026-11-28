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

//! One Save action and one Bold action shared by a "toolbar" and a "menu", plus the keyboard
//! shortcuts.  The controls are check boxes and buttons; the menu bar and toolbar widgets bind to
//! actions the same way.  Run it headlessly with `cargo run -p pushrod-widgets --example actions`.

use pushrod::action::{Action, Shortcut, StandardShortcut};
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;

/// What the application remembers.
#[derive(Default)]
struct Document {
    saves: u32,
    bold: bool,
}

/// Clicks at a point, as the mouse would.
///
/// # Arguments
///
/// * `engine` - The engine receiving the input.
/// * `doc` - The application state.
/// * `x`, `y` - Where to click.
fn click(engine: &mut Engine<Document>, doc: &mut Document, x: i32, y: i32) {
    let position = point(x, y);

    engine.dispatch(InputEvent::MouseMove { position }, doc);
    engine.dispatch(
        InputEvent::MouseDown {
            position,
            button: MouseButton::Left,
            clicks: 1,
        },
        doc,
    );
    engine.dispatch(
        InputEvent::MouseUp {
            position,
            button: MouseButton::Left,
            clicks: 1,
        },
        doc,
    );
}

fn main() {
    let mut engine: Engine<Document> = Engine::new(size(320, 160), 30);

    // Each command is described once.
    let command = if cfg!(target_os = "macos") {
        Modifiers::GUI
    } else {
        Modifiers::CTRL
    };
    let save = engine.add_action(
        Action::new("Save")
            .standard_shortcut(StandardShortcut::Save)
            .tooltip("Save the document"),
    );
    let bold = engine.add_action(
        Action::new("Bold")
            .checkable(false)
            .shortcut(Shortcut::new(Key::Char('b'), command)),
    );

    // The same action drives a toolbar control, a menu control, and the shortcut.
    let toolbar_save = engine.add_widget(Button::new("").at(10, 10).size(80, 24));
    let toolbar_bold = engine.add_widget(CheckBox::new("").at(10, 50).size(100, 20));
    let menu_bold = engine.add_widget(CheckBox::new("").at(10, 90).size(100, 20));

    engine.bind_widget(save, toolbar_save);
    engine.bind_widget(bold, toolbar_bold);
    engine.bind_widget(bold, menu_bold);
    engine.on_action(save, |doc, _ui| doc.saves += 1);
    engine.on_action(bold, move |doc, ui| {
        doc.bold = ui.action(bold).is_some_and(|a| a.checked);
    });

    let mut doc = Document::default();

    click(&mut engine, &mut doc, 20, 20);
    println!("Save from the toolbar: saves = {}", doc.saves);

    click(&mut engine, &mut doc, 15, 60);
    println!(
        "Bold from the toolbar: bold = {}, menu item checked = {}",
        doc.bold,
        engine
            .widgets()
            .get_as::<CheckBox>(menu_bold)
            .is_some_and(|c| c.is_checked())
    );

    engine.dispatch(
        InputEvent::KeyDown {
            key: Key::Char('b'),
            modifiers: command,
            repeat: false,
        },
        &mut doc,
    );
    println!("Bold from the shortcut: bold = {}", doc.bold);

    engine.set_action_enabled(save, false);
    click(&mut engine, &mut doc, 20, 20);
    println!("Save while disabled: saves = {}", doc.saves);
}
