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

//! Lists the handlers that wire to a form's widgets, and writes the form's wiring file.
//!
//! Run with `cargo run -p chassis --example wiring -- <project directory> <form>`, for example
//! `... -- demo MainWindow`.  The file is written to `src/generated/<form>_wiring.rs`.

use chassis::Project;
use chassis::codegen::wiring::{match_handlers, write_wiring};
use pushrod::registry::WidgetRegistry;

/// Prints each wired handler, then writes the wiring file, or reports why Chassis refused.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let [dir, form] = args.as_slice() else {
        eprintln!("usage: wiring <project directory> <form>");
        std::process::exit(2);
    };
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    let result = Project::load(dir).and_then(|project| {
        for w in match_handlers(&project, &registry, form)? {
            println!("{} -> {} {} ({})", w.handler, w.widget, w.event, w.callback);
        }

        let rel = write_wiring(&project, &registry, form)?;

        println!("wrote {}", rel.display());

        Ok(())
    });

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
