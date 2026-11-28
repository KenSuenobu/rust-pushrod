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

//! Loads a Chassis project directory and prints its forms, resources, and handlers.
//!
//! Run with `cargo run -p chassis --example project -- <project directory>`.

use chassis::Project;

/// Loads the project named on the command line and reports it.
fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: project <project directory>");
        std::process::exit(2);
    };
    let project = match Project::load(&dir) {
        Ok(project) => project,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let counts = project.resource_counts();

    println!("{} {}", project.package().name, project.package().version);
    println!(
        "resources: {} strings, {} images, {} icons, {} files, {} menus, {} alerts",
        counts.strings, counts.images, counts.icons, counts.files, counts.menus, counts.alerts
    );

    for form in project.forms() {
        println!("form {} -> {}", form.name, form.handler_file.display());

        for handler in project.handlers_for(&form.name) {
            println!("  line {}: {}", handler.line, handler.signature);
        }
    }
}
