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

//! What a screen reader is told about a small form, and how its requests reach the callbacks.
//! The example plays the part of the platform adapter, headlessly.  Run it with
//! `cargo run -p pushrod-widgets --example accessibility`; a real backend forwards the same
//! updates to a screen reader (see the Accessibility concept page).

use pushrod::accessibility::{AccessAction, AccessUpdate};
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::input::InputEvent;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::text_field::TextField;

/// Prints each node of an update the way a screen reader would announce it.
///
/// # Arguments
///
/// * `title` - What caused the update.
/// * `update` - The nodes that changed.
fn show(title: &str, update: &AccessUpdate) {
    println!(
        "{title}: {} node(s), focus {:?}",
        update.nodes.len(),
        update.focus
    );

    for node in &update.nodes {
        println!(
            "  #{} {:?} \"{}\" value={:?} toggled={:?}",
            node.id, node.info.role, node.info.name, node.info.value, node.info.toggled
        );
    }
}

fn main() {
    let mut engine: Engine<u32> = Engine::new(size(320, 160), 30);
    let mut saves = 0;

    let name = engine.add_widget(TextField::new().with_label("Name").at(10, 10).size(200, 24));
    let remember = engine.add_widget(CheckBox::new("_Remember me").at(10, 44));
    let save = engine.add_widget(Button::new("_Save").at(10, 80).size(80, 24));

    engine.on_click(save, |saves: &mut u32, _ui| *saves += 1);

    // A screen reader connects: the backend reports it, and the first update is the whole tree.
    engine.dispatch(InputEvent::AccessibilityActivated(true), &mut saves);

    if let Some(update) = engine.accessibility_update() {
        show("connected", &update);
    }

    // The reader asks for the check box: focus it, then press it.
    engine.perform_access_action(remember, AccessAction::Focus, &mut saves);
    engine.perform_access_action(remember, AccessAction::Click, &mut saves);

    if let Some(update) = engine.accessibility_update() {
        show("after Focus and Click", &update);
    }

    // The reader types into the field, then presses the button: the same callbacks run.
    engine.perform_access_action(
        name,
        AccessAction::SetText("Ada Example".into()),
        &mut saves,
    );
    engine.perform_access_action(save, AccessAction::Click, &mut saves);

    if let Some(update) = engine.accessibility_update() {
        show("after SetText and Click", &update);
    }

    println!("save callback ran {saves} time(s)");
}
