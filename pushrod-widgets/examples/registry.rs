// Widget registry example
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

//! Builds a small form from type names and properties with the widget registry, then shows an
//! unknown type name reported as an error, then lists the events each standard type declares.
//! Run it with
//! `cargo run -p pushrod-widgets --example registry`.

use pushrod::cache::WidgetCache;
use pushrod::registry::{Properties, WidgetRegistry};

fn main() {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    let mut cache = WidgetCache::new();
    let form = [
        (
            "pushrod.panel",
            Properties::new()
                .with_int("width", 300)
                .with_int("height", 100),
        ),
        (
            "pushrod.check_box",
            Properties::new()
                .with_text("text", "Remember me")
                .with_bool("checked", true)
                .with_int("x", 10)
                .with_int("y", 10),
        ),
        (
            "pushrod.button",
            Properties::new()
                .with_text("text", "Save")
                .with_int("x", 10)
                .with_int("y", 50)
                .with_int("width", 80)
                .with_int("height", 24),
        ),
        ("acme.tree_view", Properties::new()),
    ];

    for (name, props) in &form {
        match registry.create(name, props) {
            Ok(widget) => {
                let size = widget.get_size();
                let id = cache.add_child(0, widget);

                println!("{name}: widget {id}, {}x{}", size.w, size.h);
            }
            Err(error) => println!("{name}: {error}"),
        }
    }

    // The events a designer would list for each type: its own (default marked), then the shared ones.
    for name in registry.names() {
        let events = registry.events(name).unwrap_or_default();
        let list: Vec<String> = events
            .iter()
            .map(|e| {
                format!(
                    "{}{} ({})",
                    e.name,
                    if e.is_default { "*" } else { "" },
                    e.callback.unwrap_or("on_event")
                )
            })
            .collect();
        let total = registry.all_events(name).map_or(0, |all| all.len());

        println!(
            "{name}: {} (+{} shared)",
            list.join(", "),
            total - events.len()
        );
    }
}
