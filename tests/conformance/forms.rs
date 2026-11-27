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

//! Conformance for `docs/specs/forms.md`: the form file format.

use pushrod::form::{
    FORMAT_VERSION, FormError, FormFile, FormValue, ObjectRecord, Property, SymbolKind,
};

/// Spec "The form file format": a form written with fictional sample data reads back as the same
/// tree, and symbols find objects by name.
#[test]
fn form_file_round_trips_a_widget_tree() {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    let mut button = ObjectRecord::new("pushrod.button");
    button
        .properties
        .push(Property::named("text", FormValue::Text("OK".to_string())));
    let ok = file.add_object(Some(root), button);
    file.add_form("Sample", root);
    file.add_symbol(SymbolKind::Id, ok, "OK_BUTTON");

    let bytes = file.to_bytes().unwrap();
    assert_eq!(&bytes[0..4], b"PRSC");
    let loaded = FormFile::from_bytes(&bytes).unwrap();
    assert_eq!(loaded, file);
    assert_eq!(loaded.children(root), vec![ok]);
    assert_eq!(loaded.id_of("OK_BUTTON"), Some(ok));
}

/// Spec "The form file format": a reader rejects a newer version and never panics on damage.
#[test]
fn form_file_is_versioned_and_checked() {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    file.add_form("Sample", root);
    let mut bytes = file.to_bytes().unwrap();

    bytes[4..6].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
    assert!(matches!(
        FormFile::from_bytes(&bytes),
        Err(FormError::UnsupportedVersion { .. })
    ));
    assert!(FormFile::from_bytes(&bytes[..10]).is_err());
}
