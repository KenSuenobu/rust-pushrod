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

//! Tests for the resource manager (`pushrod::resource`): loading from a file and from bytes,
//! typed getters, stacked sets, shared images, malformed input, and the engine events.  Sample
//! data is fictional.

use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::event::EngineEvent;
use pushrod::form::{AlertKind, AlertResource, FileResource, FormFile, IconResource, ObjectRecord};
use pushrod::form::{MenuItem, MenuKind, MenuResource};
use pushrod::geometry::size;
use pushrod::resource::{AlertId, FileId, FormId, IconId, ImageId, MenuId, Resources, StringId};

/// Builds a file with one entry of every kind; `greeting` is the text of string 1.
fn sample(greeting: &str) -> FormFile {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    file.add_form("main", root);
    let r = &mut file.resources;
    let hello = r.strings.add("HELLO", greeting.to_string()).unwrap();
    let title = r.strings.add("TITLE", "Careful".to_string()).unwrap();
    let logo = r.images.add("LOGO", vec![1, 2, 3, 4]).unwrap();
    r.icons
        .add(
            "APP",
            IconResource {
                image: logo,
                mask: None,
                selected: None,
                caption: "Sample app".to_string(),
            },
        )
        .unwrap();
    r.files
        .add(
            "LICENSE",
            FileResource {
                mime: "text/plain".to_string(),
                data: b"Sample license".to_vec(),
            },
        )
        .unwrap();
    r.menus
        .add(
            "MAIN",
            MenuResource {
                kind: MenuKind::Bar,
                items: vec![MenuItem::command("Quit", "quit")],
            },
        )
        .unwrap();
    r.alerts
        .add(
            "WARN",
            AlertResource {
                kind: AlertKind::Warning,
                title,
                message: hello,
                buttons: vec![title],
            },
        )
        .unwrap();

    file
}

/// Writes a sample file to the scratch directory and returns its path.
fn write_sample(name: &str) -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("pushrod-resources-{}-{name}", std::process::id()));

    std::fs::write(&path, sample("Hello").to_bytes().unwrap()).unwrap();

    path
}

#[test]
fn file_and_embedded_bytes_give_the_same_results() {
    let path = write_sample("same.prsc");
    let from_file = Resources::load(&path).unwrap();
    let from_bytes = Resources::from_bytes(&sample("Hello").to_bytes().unwrap()).unwrap();

    std::fs::remove_file(&path).unwrap();

    assert_eq!(
        from_file.string(StringId(1)),
        from_bytes.string(StringId(1))
    );
    assert_eq!(from_file.file(FileId(1)), from_bytes.file(FileId(1)));
    assert_eq!(
        from_file.image(ImageId(1)).unwrap().data(),
        from_bytes.image(ImageId(1)).unwrap().data()
    );
    assert_eq!(from_file.sets(), 1);
}

#[test]
fn every_kind_is_retrievable_by_typed_id() {
    let resources = Resources::from_bytes(&sample("Hello").to_bytes().unwrap()).unwrap();

    assert_eq!(resources.string(StringId(1)), Some("Hello"));
    assert_eq!(resources.image(ImageId(1)).unwrap().data(), [1, 2, 3, 4]);
    let icon = resources.icon(IconId(1)).unwrap();
    assert_eq!(icon.caption, "Sample app");
    assert!(icon.image.is_same(&resources.image(ImageId(1)).unwrap()));
    assert_eq!(resources.file(FileId(1)), Some(&b"Sample license"[..]));
    assert_eq!(resources.menu(MenuId(1)).unwrap().items.len(), 1);
    assert_eq!(resources.alert(AlertId(1)).unwrap().title, 2);
    assert_eq!(resources.form(FormId(1)).unwrap().1, "main");

    assert!(resources.string(StringId(99)).is_none());
    assert!(resources.form(FormId(0)).is_none());
    assert!(resources.form(FormId(2)).is_none());
}

#[test]
fn ids_of_the_wrong_kind_do_not_compile() {
    // `Resources::image(StringId(1))` is a compile error; the compile_fail doctest in
    // `resource::id` proves it.  Here the typed getters accept only their own ID type.
    let resources = Resources::new();
    let _: Option<&str> = resources.string(StringId(1));
    let _: Option<&[u8]> = resources.file(FileId(1));
}

#[test]
fn find_looks_up_ids_by_name() {
    let resources = Resources::from_bytes(&sample("Hello").to_bytes().unwrap()).unwrap();

    assert_eq!(resources.find::<ImageId>("LOGO"), Some(ImageId(1)));
    assert_eq!(resources.find::<StringId>("TITLE"), Some(StringId(2)));
    assert_eq!(resources.find::<FormId>("main"), Some(FormId(1)));
    assert_eq!(resources.find::<AlertId>("WARN"), Some(AlertId(1)));
    assert_eq!(resources.find::<ImageId>("HELLO"), None);
}

#[test]
fn stacked_sets_resolve_in_order() {
    let app = Resources::from_bytes(&sample("App hello").to_bytes().unwrap()).unwrap();

    let mut library_file = sample("Library hello");
    library_file
        .resources
        .strings
        .add("LIBRARY_ONLY", "Shared".to_string())
        .unwrap();
    let library = Resources::from_bytes(&library_file.to_bytes().unwrap()).unwrap();

    let stack = app.with_fallback(library);

    assert_eq!(stack.sets(), 2);
    // The application overrides the library's string 1 ...
    assert_eq!(stack.string(StringId(1)), Some("App hello"));
    // ... and a string only the library has still resolves.
    assert_eq!(stack.string(StringId(3)), Some("Shared"));
    assert_eq!(stack.find::<StringId>("LIBRARY_ONLY"), Some(StringId(3)));
}

#[test]
fn an_image_is_one_shared_upload_key() {
    let resources = Resources::from_bytes(&sample("Hello").to_bytes().unwrap()).unwrap();
    let a = resources.image(ImageId(1)).unwrap();
    let b = resources.image(ImageId(1)).unwrap();
    let icon = resources.icon(IconId(1)).unwrap();

    assert!(a.is_same(&b));
    assert_eq!(a.key(), b.key());
    assert_eq!(a.key(), icon.image.key());

    // The same file loaded twice is two sets, so their keys differ.
    let other = Resources::from_bytes(&sample("Hello").to_bytes().unwrap()).unwrap();

    assert_ne!(other.image(ImageId(1)).unwrap().key(), a.key());
}

#[test]
fn malformed_input_is_an_error_never_a_panic() {
    let good = sample("Hello").to_bytes().unwrap();

    assert!(matches!(
        Resources::from_bytes(b""),
        Err(Error::FormFile { .. })
    ));
    assert!(matches!(
        Resources::from_bytes(b"not a resource file"),
        Err(Error::FormFile { .. })
    ));

    // Every truncation is an error or a valid file; none panics.
    for end in 0..good.len() {
        let _ = Resources::from_bytes(&good[..end]);
    }

    let missing = std::env::temp_dir().join("pushrod-resources-does-not-exist.prsc");

    assert!(matches!(
        Resources::load(&missing),
        Err(Error::FormFile { .. })
    ));
}

#[test]
fn the_engine_owns_resources_and_emits_events() {
    let mut engine: Engine<Vec<EngineEvent>> = Engine::new(size(100, 100), 30);
    let mut events = Vec::new();

    engine.on_engine_event(|log: &mut Vec<EngineEvent>, _ui, event| log.push(event.clone()));
    assert_eq!(engine.resources().sets(), 0);

    engine.set_resources(Resources::from_bytes(&sample("Hello").to_bytes().unwrap()).unwrap());
    engine.run_idle(&mut events);
    assert_eq!(engine.resources().string(StringId(1)), Some("Hello"));

    // Replacing frees the old sets, then loads the new ones.
    engine.set_resources(Resources::from_bytes(&sample("Again").to_bytes().unwrap()).unwrap());
    engine.clear_resources();
    engine.clear_resources();
    engine.run_idle(&mut events);

    let resource_events: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                EngineEvent::ResourcesLoaded { .. } | EngineEvent::ResourcesFreed { .. }
            )
        })
        .cloned()
        .collect();

    assert_eq!(
        resource_events,
        vec![
            EngineEvent::ResourcesLoaded { sets: 1 },
            EngineEvent::ResourcesFreed { sets: 1 },
            EngineEvent::ResourcesLoaded { sets: 1 },
            EngineEvent::ResourcesFreed { sets: 1 },
        ]
    );
    assert_eq!(engine.resources().sets(), 0);
}

#[test]
fn a_missing_form_id_is_an_error() {
    let mut engine: Engine = Engine::new(size(100, 100), 30);

    assert!(matches!(
        engine.load_resource_form(FormId(1)),
        Err(Error::FormFile { .. })
    ));
}
