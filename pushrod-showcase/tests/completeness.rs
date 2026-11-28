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

//! The completeness test: every widget type a library registers has a showcase page.

use pushrod::registry::WidgetRegistry;
use pushrod_showcase::pages;
use pushrod_showcase::registry::Registry;

/// Returns the type names of every widget library that registers itself.  A new library
/// (such as `pushrod-extra-widgets`) adds its `register` call here.
fn registered_types() -> Vec<String> {
    let mut widgets = WidgetRegistry::new();

    pushrod_widgets::register(&mut widgets);
    widgets.names().into_iter().map(String::from).collect()
}

/// Builds the showcase registry with every page.
fn showcase() -> Registry {
    let mut registry = Registry::new();

    pages::register_all(&mut registry);
    registry
}

#[test]
fn every_registered_widget_has_a_page() {
    let types = registered_types();
    let names: Vec<&str> = types.iter().map(String::as_str).collect();

    assert!(!names.is_empty());
    assert_eq!(
        showcase().missing_pages(&names),
        Vec::<&str>::new(),
        "registered widget types with no showcase page"
    );
}

#[test]
fn a_type_with_no_page_is_reported() {
    let missing = showcase().missing_pages(&["pushrod.button", "pushrod.imaginary"]);

    assert_eq!(missing, vec!["pushrod.imaginary"]);
}

#[test]
fn every_page_lists_only_registered_types() {
    let types = registered_types();
    let registry = showcase();

    for index in 0..registry.len() {
        for name in registry.get(index).unwrap().widget_types() {
            assert!(
                types.iter().any(|t| t == name),
                "unknown widget type {name}"
            );
        }
    }
}
