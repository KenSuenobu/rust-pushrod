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

//! Build script for the `form` example: writes the sample form file and generates its ID
//! constants with `pushrod::form::codegen::write_ids`, the way an application whose form comes from
//! the designer would.  The example includes the generated `form_ids.rs` and loads `sample.prsc`.

use pushrod::form::codegen::write_ids;
use pushrod::form::{FormFile, FormValue, ObjectRecord, Property, SymbolKind};
use std::path::PathBuf;

/// Builds the sample form file: a panel holding a label and an OK button.
///
/// # Returns
///
/// The form file, with one form named `main`.
fn sample_file() -> FormFile {
    let mut file = FormFile::new();
    let mut panel = ObjectRecord::new("pushrod.panel");

    panel.width = 200;
    panel.height = 80;
    let root = file.add_object(None, panel);

    let mut label = ObjectRecord::new("pushrod.text");

    label.x = 10;
    label.y = 10;
    label.width = 120;
    label.height = 20;
    label.properties.push(Property::named(
        "text",
        FormValue::Text("Sample name".into()),
    ));
    file.add_object(Some(root), label);

    let mut ok = ObjectRecord::new("pushrod.button");

    ok.x = 10;
    ok.y = 40;
    ok.width = 60;
    ok.height = 24;
    ok.properties
        .push(Property::named("text", FormValue::Text("OK".into())));
    let ok = file.add_object(Some(root), ok);

    file.add_form("main", root);
    file.add_symbol(SymbolKind::Id, ok, "OK_BUTTON");
    file.resources
        .strings
        .add("GREETING", "Hello, Sample".to_string())
        .expect("the sample string is added");

    file
}

/// Writes `sample.prsc` and `form_ids.rs` into `OUT_DIR`.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let form = out.join("sample.prsc");
    let bytes = sample_file().to_bytes().expect("the sample form encodes");

    std::fs::write(&form, bytes).expect("the form file is written");
    write_ids(&form, out.join("form_ids.rs")).expect("the ID constants are generated");
}
