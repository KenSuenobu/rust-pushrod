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

//! Appends a handler stub to a form's handler file, and lists the handlers afterwards.
//!
//! Run with `cargo run -p chassis --example codegen -- <project directory> <form> <stub>`, for
//! example `... -- demo MainWindow "pub fn on_close() {}"`.

use chassis::Project;
use chassis::codegen::insert_handler_stub;

/// Inserts the stub named on the command line, or reports why Chassis refused.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let [dir, form, stub] = args.as_slice() else {
        eprintln!("usage: codegen <project directory> <form> <stub>");
        std::process::exit(2);
    };
    let result = Project::load(dir).and_then(|mut project| {
        let name = insert_handler_stub(&mut project, form, stub)?;

        println!("inserted {name}");

        for handler in project.handlers_for(form) {
            println!("  line {}: {}", handler.line, handler.signature);
        }

        Ok(())
    });

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
