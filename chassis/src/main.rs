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

//! The `chassis` command: `chassis generate [project]` and `chassis check [project]`.

use pushrod::registry::WidgetRegistry;

/// Runs the command line with every Pushrod widget registered, and exits with its code.
fn main() {
    let mut registry = WidgetRegistry::new();

    pushrod_widgets::register(&mut registry);

    let code = chassis::cli::run(
        std::env::args_os(),
        &registry,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );

    std::process::exit(code);
}
