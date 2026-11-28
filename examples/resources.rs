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

//! Resources: load a resource file from bytes, stack a library set under an application set, and
//! get strings, images, and files by typed ID or by name.  It opens no window.
//!
//! Run it with `cargo run --example resources`.

use pushrod::form::{FileResource, FormFile, ObjectRecord};
use pushrod::resource::{FileId, ImageId, Resources, StringId};

/// Builds a resource file in memory.  A real program ships a file made by a tool and embeds it with
/// `include_bytes!("app.prsc")`.
fn build(greeting: &str) -> Vec<u8> {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));

    file.add_form("main", root);
    file.resources
        .strings
        .add("GREETING", greeting.to_string())
        .unwrap();
    file.resources
        .images
        .add("LOGO", vec![0x89, b'P', b'N', b'G'])
        .unwrap();
    file.resources
        .files
        .add(
            "NOTES",
            FileResource {
                mime: "text/plain".to_string(),
                data: b"Sample notes".to_vec(),
            },
        )
        .unwrap();

    file.to_bytes().unwrap()
}

fn main() {
    // The application's set is searched first; the library's set fills in what it lacks.
    let app = Resources::from_bytes(&build("Hello from the app")).unwrap();
    let library = Resources::from_bytes(&build("Hello from the library")).unwrap();
    let resources = app.with_fallback(library);

    // Generated constants would normally name these IDs.
    let greeting = resources.find::<StringId>("GREETING").unwrap();

    println!("sets: {}", resources.sets());
    println!("greeting: {}", resources.string(greeting).unwrap());

    let logo = resources.image(ImageId(1)).unwrap();

    println!("logo: {} bytes, key {}", logo.data().len(), logo.key());
    println!(
        "notes: {}",
        String::from_utf8_lossy(resources.file(FileId(1)).unwrap())
    );
}
