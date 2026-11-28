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

//! Builds a small form in code, saves it to a form file with `FormWriter`, and loads the file
//! back into a second engine.  Run it with `cargo run -p pushrod-widgets --example form_writer`.

use pushrod::engine::Engine;
use pushrod::form::{FormFile, FormWriter, SymbolKind};
use pushrod::geometry::{point, size};
use pushrod::widget::Widget;
use pushrod_widgets::prelude::*;

fn main() {
    // Build the form in code.
    let mut designer: Engine = Engine::new(size(320, 200), 30);
    let mut panel = Panel::new();

    panel.set_size(size(200, 80));
    let panel = designer.add_widget(panel);

    let mut ok = Button::new("OK");

    ok.set_origin(point(10, 40));
    ok.set_size(size(60, 24));
    let ok = designer.add_child_widget(panel, ok);

    // Save it: the writer returns the object number of every widget it wrote.
    let mut writer = FormWriter::new();

    pushrod_widgets::register_form_types(&mut writer);

    let numbers = writer
        .add_form("main", designer.widgets(), panel)
        .expect("the form can be saved");

    writer.add_symbol(SymbolKind::Id, numbers[&ok], "OK_BUTTON");

    let path = std::env::temp_dir().join("pushrod-example.form");

    writer.write(&path).expect("the file can be written");
    println!(
        "wrote {} bytes to {}",
        std::fs::metadata(&path).unwrap().len(),
        path.display()
    );

    // Load it back, as an application would.
    let file = FormFile::from_bytes(&std::fs::read(&path).unwrap()).expect("the file is valid");
    let mut app: Engine = Engine::new(size(320, 200), 30);

    pushrod_widgets::register(app.widget_registry());

    let form = app.load_form(&file, "main").expect("the form loads");

    println!(
        "loaded {} widgets; OK button found: {}",
        form.len(),
        form.named("OK_BUTTON").is_some()
    );
    std::fs::remove_file(&path).ok();
}
