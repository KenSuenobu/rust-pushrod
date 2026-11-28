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

//! Build script: reads each page's source file and writes the text of its `build` function
//! to `$OUT_DIR/page_sources.rs`, so the source view shows code that cannot go stale.

#[path = "src/extract.rs"]
mod extract;

use std::fmt::Write as _;
use std::path::Path;
use std::{env, fs};

fn main() {
    let pages =
        Path::new(&env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR")).join("src/pages");
    let mut files: Vec<_> = fs::read_dir(&pages)
        .expect("src/pages")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    let mut out = String::from(
        "/// Each page file's name and the text of its `build` function.\npub const PAGE_SOURCES: &[(&str, &str)] = &[\n",
    );

    files.sort();
    println!("cargo:rerun-if-changed=src/pages");
    println!("cargo:rerun-if-changed=src/extract.rs");

    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());

        let text = fs::read_to_string(&path).expect("read page");

        if let Some(code) = extract::build_fn(&text) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("file name");

            writeln!(out, "    ({name:?}, {code:?}),").expect("write");
        }
    }

    out.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").expect("OUT_DIR")).join("page_sources.rs"),
        out,
    )
    .expect("write page_sources.rs");
}
