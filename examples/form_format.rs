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

//! Builds a small form in memory, writes it in the binary form file format, reads it back, and
//! prints the object table, the symbols, and the size of the file.  It opens no window.
//!
//! Run it with `cargo run --example form_format`.

use pushrod::form::{
    AlertKind, AlertResource, FORMAT_VERSION, FormFile, FormValue, LayoutSpec, MenuItem, MenuKind,
    MenuResource, NO_OBJECT, ObjectRecord, Property, SymbolKind,
};

fn main() {
    let mut file = FormFile::new();

    let mut panel = ObjectRecord::new("pushrod.panel");
    panel.width = 240;
    panel.height = 120;
    panel.properties.push(Property::Layout(LayoutSpec {
        kind: 2,
        params: vec![8],
    }));
    let root = file.add_object(None, panel);

    let mut label = ObjectRecord::new("pushrod.text");
    label.properties.push(Property::named(
        "text",
        FormValue::Text("Sample name".to_string()),
    ));
    file.add_object(Some(root), label);

    let mut button = ObjectRecord::new("pushrod.button");
    button
        .properties
        .push(Property::named("text", FormValue::Text("OK".to_string())));
    let ok = file.add_object(Some(root), button);

    file.add_form("Sample", root);
    file.add_symbol(SymbolKind::Id, ok, "OK_BUTTON");
    file.add_symbol(SymbolKind::Action, 1, "confirm");

    // Resources: strings, an alert that refers to them, and a menu, each with a stable ID.
    let title = file
        .resources
        .strings
        .add("SAVE_TITLE", "Save failed".to_string());
    let message = file
        .resources
        .strings
        .add("SAVE_MESSAGE", "Sample message".to_string());
    let (title, message) = (title.unwrap(), message.unwrap());
    file.resources
        .alerts
        .add(
            "SAVE_FAILED",
            AlertResource {
                kind: AlertKind::Error,
                title,
                message,
                buttons: vec![],
            },
        )
        .unwrap();
    file.resources
        .menus
        .add(
            "MAIN_MENU",
            MenuResource {
                kind: MenuKind::Bar,
                items: vec![MenuItem::command("Quit", "quit")],
            },
        )
        .unwrap();

    let bytes = file.to_bytes().expect("the sample form is consistent");
    println!("Form file version {FORMAT_VERSION}, {} bytes", bytes.len());

    let loaded = FormFile::from_bytes(&bytes).expect("the bytes were just written");
    assert_eq!(loaded, file);
    for (number, object) in loaded.objects.iter().enumerate() {
        let parent = if object.parent == NO_OBJECT {
            "none".to_string()
        } else {
            object.parent.to_string()
        };
        println!(
            "object {number}: {} parent={parent} properties={}",
            object.type_name,
            object.properties.len()
        );
    }
    for s in &loaded.resources.strings {
        println!("string {} is {}: {}", s.id, s.name, s.value);
    }
    println!(
        "{} alert(s), {} menu(s)",
        loaded.resources.alerts.len(),
        loaded.resources.menus.len()
    );
    println!("OK_BUTTON is object {}", loaded.id_of("OK_BUTTON").unwrap());
}
