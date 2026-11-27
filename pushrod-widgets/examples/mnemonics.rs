// Mnemonics
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

//! Keyboard mnemonics on a small form, driven headlessly: Alt plus the underlined letter clicks a
//! button, toggles a check box, or, on a label, focuses the next field.  Run it with
//! `cargo run -p pushrod-widgets --example mnemonics`.

use pushrod::cache::WidgetCache;
use pushrod::event::Event;
use pushrod::input::{InputEvent, Key, Modifiers};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;

/// Presses Alt plus a letter and prints what the form reported.
///
/// # Arguments
///
/// * `cache` - The form.
/// * `letter` - The letter to press.
fn alt(cache: &mut WidgetCache, letter: char) {
    let events: Vec<Event> = cache.handle_event(InputEvent::KeyDown {
        key: Key::Char(letter),
        modifiers: Modifiers::ALT,
        repeat: false,
    });

    println!(
        "Alt+{letter}: focus {:?}, events {events:?}",
        cache.focused()
    );
}

fn main() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 120)));

    let name = cache.add_child(0, Box::new(Text::new("_Name").at(10, 10).size(60, 20)));
    let _field = cache.add_child(0, Box::new(CheckBox::new("Nickname").at(80, 10)));
    let _remember = cache.add_child(0, Box::new(CheckBox::new("_Remember me").at(10, 40)));
    let _save = cache.add_child(0, Box::new(Button::new("_Save").at(10, 70).size(80, 24)));

    println!("label {name}; conflicts: {:?}", cache.mnemonic_conflicts());

    alt(&mut cache, 'n');
    alt(&mut cache, 'r');
    alt(&mut cache, 's');
}
