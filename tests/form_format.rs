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

//! Tests for the form file format (`docs/FORM_FORMAT.md`): round trips, hand-built bytes, unknown
//! tags, versions, and damaged files.  Sample data is fictional.

use pushrod::color::Color;
use pushrod::form::{
    ChildLayout, FORMAT_VERSION, FormError, FormFile, FormValue, LayoutSpec, NO_OBJECT,
    ObjectRecord, Property, SymbolKind,
};

/// Builds a small form: a panel holding a label and a button, with symbols and an image.
fn sample() -> FormFile {
    let mut file = FormFile::new();
    let image = file.add_image("logo", vec![1, 2, 3, 4]);

    let mut panel = ObjectRecord::new("pushrod.panel");
    panel.width = 320;
    panel.height = 200;
    panel.properties.push(Property::Layout(LayoutSpec {
        kind: 2,
        params: vec![8, -4],
    }));
    let root = file.add_object(None, panel);

    let mut label = ObjectRecord::new("pushrod.text");
    label.x = 10;
    label.y = -5;
    label
        .properties
        .push(Property::named("text", FormValue::Text("Name".into())));
    label.properties.push(Property::named(
        "color",
        FormValue::Color(Color::RGBA(1, 2, 3, 4)),
    ));
    label.properties.push(Property::ChildLayout(ChildLayout {
        stretch: 1,
        horizontal: 2,
        vertical: 3,
        margins: [1, 2, 3, 4],
        cell: [0, 1, 2, 3],
    }));
    file.add_object(Some(root), label);

    let mut button = ObjectRecord::new("pushrod.button");
    button.behavior = 0x11;
    button.state = 0x22;
    button.anchors = 0x33;
    button
        .properties
        .push(Property::named("text", FormValue::Text("OK".into())));
    button
        .properties
        .push(Property::named("enabled", FormValue::Bool(true)));
    button
        .properties
        .push(Property::named("count", FormValue::Int(-42)));
    button
        .properties
        .push(Property::named("ratio", FormValue::Float(0.25)));
    button
        .properties
        .push(Property::named("icon", FormValue::Image(image)));
    let ok = file.add_object(Some(root), button);

    file.add_form("Sample", root);
    file.add_symbol(SymbolKind::Id, ok, "OK_BUTTON");
    file.add_symbol(SymbolKind::Action, 7, "save");
    file
}

#[test]
fn round_trip_keeps_everything() {
    let file = sample();
    let bytes = file.to_bytes().unwrap();
    let back = FormFile::from_bytes(&bytes).unwrap();
    assert_eq!(back, file);
    assert_eq!(back.children(0), vec![1, 2]);
    assert_eq!(back.id_of("OK_BUTTON"), Some(2));
    assert_eq!(back.form("Sample").unwrap().root, 0);
}

#[test]
fn header_has_magic_version_and_section_offsets() {
    let bytes = sample().to_bytes().unwrap();
    assert_eq!(&bytes[0..4], b"PRSC");
    assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), FORMAT_VERSION);
    assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 56);
    // The form table starts right after the header.
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 56);
}

/// Hand-builds a file with one form, one object, and the given property section, then reads it.
fn hand_built(props: &[u8], version: u16) -> Result<FormFile, FormError> {
    let mut strings = Vec::new();
    strings.extend_from_slice(&3u32.to_le_bytes());
    for s in ["Hand", "pushrod.button", "text"] {
        strings.extend_from_slice(&(s.len() as u32).to_le_bytes());
        strings.extend_from_slice(s.as_bytes());
    }
    let mut forms = 1u32.to_le_bytes().to_vec();
    forms.extend_from_slice(&0u32.to_le_bytes()); // name: string 0
    forms.extend_from_slice(&0u32.to_le_bytes()); // root: object 0
    let mut objects = 1u32.to_le_bytes().to_vec();
    for v in [1u32, NO_OBJECT, NO_OBJECT, NO_OBJECT] {
        objects.extend_from_slice(&v.to_le_bytes()); // type, parent, child, sibling
    }
    for v in [5i32, 6, 70, 80] {
        objects.extend_from_slice(&v.to_le_bytes()); // x, y, w, h
    }
    for v in [0u32, 0, 0, if props.is_empty() { u32::MAX } else { 0 }] {
        objects.extend_from_slice(&v.to_le_bytes()); // behavior, state, anchors, props
    }
    let mut property_section = Vec::new();
    if !props.is_empty() {
        property_section.extend_from_slice(&(props.len() as u32).to_le_bytes());
        property_section.extend_from_slice(props);
    }
    let images = 0u32.to_le_bytes().to_vec();
    let symbols = 0u32.to_le_bytes().to_vec();

    let sections = [
        &forms,
        &objects,
        &property_section,
        &strings,
        &images,
        &symbols,
    ];
    let mut out = b"PRSC".to_vec();
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&56u16.to_le_bytes());
    let mut offset = 56u32;
    for s in sections {
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&(s.len() as u32).to_le_bytes());
        offset += s.len() as u32;
    }
    for s in sections {
        out.extend_from_slice(s);
    }
    FormFile::from_bytes(&out)
}

#[test]
fn reads_hand_built_bytes() {
    // One TEXT property: tag 1, length 9, name = string 2 ("text"), kind 3, value = string 0.
    let mut prop = 1u16.to_le_bytes().to_vec();
    prop.extend_from_slice(&9u32.to_le_bytes());
    prop.extend_from_slice(&2u32.to_le_bytes());
    prop.push(3);
    prop.extend_from_slice(&0u32.to_le_bytes());
    let file = hand_built(&prop, 1).unwrap();
    let o = &file.objects[0];
    assert_eq!(o.type_name, "pushrod.button");
    assert_eq!((o.x, o.y, o.width, o.height), (5, 6, 70, 80));
    assert_eq!(file.forms[0].name, "Hand");
    assert_eq!(
        o.properties,
        vec![Property::named("text", FormValue::Text("Hand".into()))]
    );
}

#[test]
fn object_without_properties_reads_empty() {
    let file = hand_built(&[], 1).unwrap();
    assert!(file.objects[0].properties.is_empty());
}

#[test]
fn unknown_tags_and_value_kinds_are_skipped() {
    let mut props = Vec::new();
    // Unknown tag 0x7777 with 5 bytes of payload.
    props.extend_from_slice(&0x7777u16.to_le_bytes());
    props.extend_from_slice(&5u32.to_le_bytes());
    props.extend_from_slice(&[9, 9, 9, 9, 9]);
    // A NAMED property with unknown value kind 99.
    props.extend_from_slice(&1u16.to_le_bytes());
    props.extend_from_slice(&7u32.to_le_bytes());
    props.extend_from_slice(&2u32.to_le_bytes());
    props.extend_from_slice(&[99, 1, 2]);
    // A known property after them is still read.
    props.extend_from_slice(&1u16.to_le_bytes());
    props.extend_from_slice(&6u32.to_le_bytes());
    props.extend_from_slice(&2u32.to_le_bytes());
    props.extend_from_slice(&[0, 1]);
    let file = hand_built(&props, 1).unwrap();
    assert_eq!(
        file.objects[0].properties,
        vec![Property::named("text", FormValue::Bool(true))]
    );
}

#[test]
fn newer_version_is_rejected_and_older_accepted() {
    assert_eq!(
        hand_built(&[], FORMAT_VERSION + 1).unwrap_err(),
        FormError::UnsupportedVersion {
            version: FORMAT_VERSION + 1
        }
    );
    assert!(hand_built(&[], FORMAT_VERSION).is_ok());
}

#[test]
fn bad_magic_is_rejected() {
    assert_eq!(
        FormFile::from_bytes(b"NOPE").unwrap_err(),
        FormError::BadMagic
    );
    assert_eq!(FormFile::from_bytes(&[]).unwrap_err(), FormError::BadMagic);
}

#[test]
fn every_truncation_is_an_error_not_a_panic() {
    let bytes = sample().to_bytes().unwrap();
    for len in 0..bytes.len() {
        assert!(FormFile::from_bytes(&bytes[..len]).is_err(), "length {len}");
    }
}

#[test]
fn every_flipped_byte_is_handled_without_panic() {
    let bytes = sample().to_bytes().unwrap();
    for i in 0..bytes.len() {
        let mut damaged = bytes.clone();
        damaged[i] ^= 0xFF;
        let _ = FormFile::from_bytes(&damaged);
    }
}

#[test]
fn writer_rejects_inconsistent_files() {
    let mut file = sample();
    file.forms[0].root = 99;
    assert!(matches!(file.to_bytes(), Err(FormError::Invalid { .. })));

    let mut file = sample();
    file.objects[1].parent = 2; // Not the object whose child list holds it.
    assert!(matches!(file.to_bytes(), Err(FormError::Invalid { .. })));

    let mut file = sample();
    file.objects[0]
        .properties
        .push(Property::named("icon", FormValue::Image(5)));
    assert!(matches!(file.to_bytes(), Err(FormError::Invalid { .. })));
}

#[test]
fn parent_cycle_is_rejected() {
    let mut file = FormFile::new();
    file.add_object(None, ObjectRecord::new("a"));
    file.add_object(Some(0), ObjectRecord::new("b"));
    file.objects[0].parent = 1; // 0 and 1 now name each other as parent.
    assert!(file.validate().is_err());
}

#[test]
fn strings_are_pooled_once() {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));
    for _ in 0..10 {
        let mut b = ObjectRecord::new("pushrod.button");
        b.properties.push(Property::named(
            "text",
            FormValue::Text("pooled-string".into()),
        ));
        file.add_object(Some(root), b);
    }
    let bytes = file.to_bytes().unwrap();
    let hits = bytes.windows(13).filter(|w| *w == b"pooled-string").count();
    assert_eq!(hits, 1);
}

#[test]
fn display_messages_are_readable() {
    assert!(FormError::BadMagic.to_string().contains("PRSC"));
    assert!(
        FormError::UnsupportedVersion { version: 9 }
            .to_string()
            .contains('9')
    );
}
