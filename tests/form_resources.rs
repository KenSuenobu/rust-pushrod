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

//! Tests for the resource kinds of the form file format (`docs/FORM_FORMAT.md`, "Resource
//! section"): round trips for every kind, stable IDs, skipped unknown tables, and damaged
//! sections.  Sample data is fictional.

use pushrod::form::{
    AlertKind, AlertResource, FileResource, FormError, FormFile, IconResource, MAX_MENU_DEPTH,
    MenuItem, MenuKind, MenuResource, ObjectRecord, ResourceTable,
};

/// Builds a file holding one entry of every resource kind.
fn sample() -> FormFile {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    file.add_form("Sample", root);
    let r = &mut file.resources;
    let title = r
        .strings
        .add("SAVE_TITLE", "Save failed".to_string())
        .unwrap();
    let message = r
        .strings
        .add("SAVE_MESSAGE", "Could not save \u{00e9}".to_string())
        .unwrap();
    let retry = r.strings.add("RETRY", "Retry".to_string()).unwrap();
    let logo = r.images.add("LOGO", vec![0x89, b'P', b'N', b'G']).unwrap();
    let mask = r.images.add("LOGO_MASK", vec![1, 2, 3]).unwrap();
    r.icons
        .add(
            "APP",
            IconResource {
                image: logo,
                mask: Some(mask),
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
                data: b"Sample license text".to_vec(),
            },
        )
        .unwrap();
    r.menus
        .add(
            "MAIN_MENU",
            MenuResource {
                kind: MenuKind::Bar,
                items: vec![MenuItem::Submenu {
                    label: "File".to_string(),
                    items: vec![
                        MenuItem::Command {
                            label: "Save".to_string(),
                            action: "save".to_string(),
                            shortcut: Some("Ctrl+S".to_string()),
                            enabled: false,
                        },
                        MenuItem::Separator,
                        MenuItem::command("Quit", "quit"),
                    ],
                }],
            },
        )
        .unwrap();
    r.alerts
        .add(
            "SAVE_FAILED",
            AlertResource {
                kind: AlertKind::Error,
                title,
                message,
                buttons: vec![retry],
            },
        )
        .unwrap();
    file
}

#[test]
fn every_kind_round_trips() {
    let file = sample();
    let back = FormFile::from_bytes(&file.to_bytes().unwrap()).unwrap();
    assert_eq!(back, file);
    assert_eq!(back.resources.strings.len(), 3);
    assert_eq!(
        back.resources.strings.by_name("RETRY").unwrap().value,
        "Retry"
    );
    assert_eq!(back.resources.icons.get(1).unwrap().value.mask, Some(2));
}

#[test]
fn every_alert_and_menu_kind_round_trips() {
    let mut file = sample();
    for (i, kind) in [AlertKind::Info, AlertKind::Warning, AlertKind::Question]
        .into_iter()
        .enumerate()
    {
        file.resources
            .alerts
            .add(
                &format!("A{i}"),
                AlertResource {
                    kind,
                    title: 1,
                    message: 2,
                    buttons: vec![],
                },
            )
            .unwrap();
    }
    file.resources
        .menus
        .add(
            "CONTEXT",
            MenuResource {
                kind: MenuKind::Context,
                items: vec![],
            },
        )
        .unwrap();
    let back = FormFile::from_bytes(&file.to_bytes().unwrap()).unwrap();
    assert_eq!(back, file);
}

#[test]
fn a_file_without_resources_keeps_the_short_header() {
    let mut file = sample();
    file.resources = Default::default();
    let bytes = file.to_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 56);
    assert!(FormFile::from_bytes(&bytes).unwrap().resources.is_empty());
    let with = sample().to_bytes().unwrap();
    assert_eq!(u16::from_le_bytes([with[6], with[7]]), 64);
}

#[test]
fn ids_start_at_one_and_are_never_reused() {
    let mut t = ResourceTable::new();
    assert_eq!(t.add("A", 1).unwrap(), 1);
    assert_eq!(t.add("B", 2).unwrap(), 2);
    assert_eq!(t.add("C", 3).unwrap(), 3);
    t.remove(3);
    // The removed ID stays retired, even for the last entry.
    assert_eq!(t.add("D", 4).unwrap(), 4);
    assert_eq!(t.next_id(), 5);
    assert!(t.remove(99).is_none());
}

#[test]
fn ids_stay_stable_when_removed_reordered_and_saved() {
    let mut file = sample();
    let strings = &mut file.resources.strings;
    strings.remove(1);
    assert!(strings.move_to(3, 0));
    assert!(!strings.move_to(77, 0));
    let new = strings.add("NEW", "New".to_string()).unwrap();
    assert_eq!(new, 4);
    // The alert still refers to string 3 and 2 by ID: the references stay valid.
    file.resources.alerts.get_mut(1).unwrap().title = 2;

    let back = FormFile::from_bytes(&file.to_bytes().unwrap()).unwrap();
    let ids: Vec<u32> = back.resources.strings.iter().map(|e| e.id).collect();
    assert_eq!(ids, vec![3, 2, 4]);
    assert_eq!(back.resources.strings.next_id(), 5);
    assert_eq!(back.resources.strings.get(2).unwrap().name, "SAVE_MESSAGE");
    // A retired ID stays retired after saving and loading.
    let mut back = back;
    assert_eq!(
        back.resources.strings.add("LATER", String::new()).unwrap(),
        5
    );
}

#[test]
fn insert_with_id_moves_the_next_id_past_it() {
    let mut t = ResourceTable::new();
    t.insert_with_id(10, "TEN", ()).unwrap();
    assert_eq!(t.next_id(), 11);
    t.insert_with_id(3, "THREE", ()).unwrap();
    assert_eq!(t.next_id(), 11);
    assert!(t.insert_with_id(0, "ZERO", ()).is_err());
    assert!(t.insert_with_id(10, "OTHER", ()).is_err());
    assert!(t.insert_with_id(u32::MAX, "MAX", ()).is_err());
    assert!(t.add("TEN", ()).is_err(), "names are unique");
}

/// Replaces the resource section of a written file with `section` (the section is last).
fn with_section(file: &FormFile, section: &[u8]) -> Vec<u8> {
    let mut bytes = file.to_bytes().unwrap();
    let at = u32::from_le_bytes(bytes[56..60].try_into().unwrap()) as usize;
    bytes.truncate(at);
    bytes[60..64].copy_from_slice(&(section.len() as u32).to_le_bytes());
    bytes.extend_from_slice(section);
    bytes
}

#[test]
fn unknown_tables_are_skipped() {
    let file = sample();
    let mut section = Vec::new();
    // Tables 90 and 91 are unknown to this version, before and after the known ones.
    section.extend_from_slice(&90u16.to_le_bytes());
    section.extend_from_slice(&3u32.to_le_bytes());
    section.extend_from_slice(&[7, 7, 7]);
    let at = u32::from_le_bytes(file.to_bytes().unwrap()[56..60].try_into().unwrap()) as usize;
    section.extend_from_slice(&file.to_bytes().unwrap()[at..]);
    section.extend_from_slice(&91u16.to_le_bytes());
    section.extend_from_slice(&0u32.to_le_bytes());

    let back = FormFile::from_bytes(&with_section(&file, &section)).unwrap();
    assert_eq!(back, file);
}

#[test]
fn a_header_without_a_resource_section_reads_as_none() {
    let mut file = sample();
    file.resources = Default::default();
    assert!(
        FormFile::from_bytes(&file.to_bytes().unwrap())
            .unwrap()
            .resources
            .is_empty()
    );
}

#[test]
fn writer_rejects_dangling_references() {
    let mut file = sample();
    file.resources.alerts.get_mut(1).unwrap().buttons.push(99);
    assert!(matches!(file.to_bytes(), Err(FormError::Invalid { .. })));

    let mut file = sample();
    file.resources.icons.get_mut(1).unwrap().selected = Some(42);
    assert!(matches!(file.to_bytes(), Err(FormError::Invalid { .. })));
}

#[test]
fn writer_rejects_menus_nested_too_deeply() {
    let mut items = vec![MenuItem::Separator];
    for _ in 0..MAX_MENU_DEPTH {
        items = vec![MenuItem::Submenu {
            label: "More".to_string(),
            items,
        }];
    }
    let mut file = FormFile::new();
    file.resources
        .menus
        .add(
            "DEEP",
            MenuResource {
                kind: MenuKind::Bar,
                items: items.clone(),
            },
        )
        .unwrap();
    assert!(file.to_bytes().is_err());

    // One level less is the deepest allowed menu, and it round trips.
    let MenuItem::Submenu { items: inner, .. } = items.remove(0) else {
        unreachable!()
    };
    let mut file = FormFile::new();
    file.resources
        .menus
        .add(
            "OK",
            MenuResource {
                kind: MenuKind::Bar,
                items: inner,
            },
        )
        .unwrap();
    let back = FormFile::from_bytes(&file.to_bytes().unwrap()).unwrap();
    assert_eq!(back, file);
}

#[test]
fn unknown_menu_items_are_skipped() {
    let mut file = FormFile::new();
    file.resources
        .menus
        .add(
            "M",
            MenuResource {
                kind: MenuKind::Context,
                items: vec![MenuItem::Separator],
            },
        )
        .unwrap();
    let bytes = file.to_bytes().unwrap();
    let at = u32::from_le_bytes(bytes[56..60].try_into().unwrap()) as usize;
    let mut section = bytes[at..].to_vec();
    // Table body: tag(2) len(4) | next_id(4) count(4) | id(4) name(4+1) kind(1) | count(4) ...
    let count_at = 2 + 4 + 4 + 4 + 4 + 5 + 1;
    section[count_at..count_at + 4].copy_from_slice(&2u32.to_le_bytes());
    // Add an item of unknown kind 77 with a 2-byte body.
    section.extend_from_slice(&[77, 2, 0, 0, 0, 9, 9]);
    let table_len = (section.len() - 6) as u32;
    section[2..6].copy_from_slice(&table_len.to_le_bytes());

    let back = FormFile::from_bytes(&with_section(&file, &section)).unwrap();
    assert_eq!(back, file);
}

#[test]
fn damaged_sections_are_errors_not_panics() {
    let bytes = sample().to_bytes().unwrap();
    for len in 0..bytes.len() {
        assert!(FormFile::from_bytes(&bytes[..len]).is_err(), "length {len}");
    }
    for i in 0..bytes.len() {
        let mut damaged = bytes.clone();
        damaged[i] ^= 0xFF;
        let _ = FormFile::from_bytes(&damaged);
    }
}

#[test]
fn duplicate_ids_and_repeated_tables_are_rejected() {
    // A strings table with next_id 3 and two entries that both use ID 1.
    let mut body = Vec::new();
    body.extend_from_slice(&3u32.to_le_bytes());
    body.extend_from_slice(&2u32.to_le_bytes());
    for name in ["A", "B"] {
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(name.as_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
    }
    let mut section = 1u16.to_le_bytes().to_vec();
    section.extend_from_slice(&(body.len() as u32).to_le_bytes());
    section.extend_from_slice(&body);
    let file = sample();
    assert!(FormFile::from_bytes(&with_section(&file, &section)).is_err());

    let empty = {
        let mut s = 1u16.to_le_bytes().to_vec();
        s.extend_from_slice(&8u32.to_le_bytes());
        s.extend_from_slice(&1u32.to_le_bytes());
        s.extend_from_slice(&0u32.to_le_bytes());
        s
    };
    let twice = [empty.clone(), empty].concat();
    assert!(FormFile::from_bytes(&with_section(&file, &twice)).is_err());
}

#[test]
fn an_id_at_or_past_the_next_id_is_rejected() {
    let mut body = Vec::new();
    body.extend_from_slice(&1u32.to_le_bytes()); // next_id 1
    body.extend_from_slice(&1u32.to_le_bytes());
    body.extend_from_slice(&1u32.to_le_bytes()); // id 1 >= next_id
    body.extend_from_slice(&1u32.to_le_bytes());
    body.push(b'A');
    body.extend_from_slice(&0u32.to_le_bytes());
    let mut section = 1u16.to_le_bytes().to_vec();
    section.extend_from_slice(&(body.len() as u32).to_le_bytes());
    section.extend_from_slice(&body);
    assert!(FormFile::from_bytes(&with_section(&sample(), &section)).is_err());
}
