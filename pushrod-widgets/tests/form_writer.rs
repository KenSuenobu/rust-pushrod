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

//! Tests for saving a widget subtree to a form file: the round trip through the loader keeps
//! properties and painted output, composite parts are not written, and pools are deduplicated.

use pushrod::backend::headless::{HeadlessBackend, PaintCall};
use pushrod::engine::Engine;
use pushrod::error::Error;
use pushrod::form::{FormFile, FormWriter, Property, SymbolKind};
use pushrod::geometry::{point, size};
use pushrod::layout::{Align, Column, Margin};
use pushrod::widget::Widget;
use pushrod_widgets::prelude::*;

/// Makes an engine whose registry holds the standard widgets.
fn engine() -> Engine {
    let mut engine: Engine = Engine::new(size(320, 240), 30);

    pushrod_widgets::register(engine.widget_registry());

    engine
}

/// Makes a writer that knows the standard widgets.
fn writer() -> FormWriter {
    let mut writer = FormWriter::new();

    pushrod_widgets::register_form_types(&mut writer);

    writer
}

/// Draws one frame and returns what each widget painted.
fn paint(engine: &mut Engine) -> Vec<Vec<PaintCall>> {
    let mut backend = HeadlessBackend::new(size(320, 240));

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);

    backend
        .painted
        .into_iter()
        .map(|(_, calls)| calls)
        .collect()
}

/// Builds a code-made form: a column panel holding a label, a checked check box, and a stretched
/// button with a margin.  Returns the panel's ID.
fn build(engine: &mut Engine) -> u32 {
    let mut panel = Panel::new();

    panel.set_origin(point(5, 6));
    panel.set_size(size(200, 120));
    panel.set_layout(Box::new(Column::new().spacing(4).margins(Margin::all(3))));
    let panel = engine.add_widget(panel);

    let mut label = Text::new("Name");

    label.set_size(size(100, 20));
    label.set_tooltip("Your name");
    engine.add_child_widget(panel, label);

    let mut check = CheckBox::new("Remember me");

    check.set_size(size(120, 20));
    check.set_checked(true);
    engine.add_child_widget(panel, check);

    let mut ok = Button::new("OK");

    ok.set_size(size(60, 24));
    ok.set_layout_params(pushrod::layout::LayoutParams {
        stretch: 2,
        alignment: pushrod::layout::Alignment::new(Align::Center, Align::End),
        margin: Margin::all(2),
    });
    engine.add_child_widget(panel, ok);

    panel
}

#[test]
fn round_trip_keeps_properties_and_pixels() {
    let mut built = engine();
    let panel = build(&mut built);
    let mut writer = writer();
    let numbers = writer.add_form("main", built.widgets(), panel).unwrap();

    assert_eq!(numbers.len(), 4);

    // Through bytes, as a file on disk would be.
    let file = FormFile::from_bytes(&writer.to_bytes().unwrap()).unwrap();
    let mut loaded = engine();
    let form = loaded.load_form(&file, "main").unwrap();

    assert_eq!(form.len(), 4);

    let kids = loaded.widgets().children_of(form.root()).to_vec();

    assert!(
        loaded
            .widgets()
            .get_as::<CheckBox>(kids[1])
            .unwrap()
            .is_checked()
    );
    assert_eq!(
        loaded.widgets().get(kids[0]).unwrap().tooltip(),
        "Your name"
    );

    let golden = paint(&mut built);

    assert!(golden.len() >= 4);
    assert_eq!(paint(&mut loaded), golden);
}

#[test]
fn writing_to_a_path_saves_a_loadable_file() {
    let mut built = engine();
    let panel = build(&mut built);
    let mut writer = writer();

    writer.add_form("main", built.widgets(), panel).unwrap();

    let path = std::env::temp_dir().join(format!("pushrod-writer-{}.form", std::process::id()));

    writer.write(&path).unwrap();

    let bytes = std::fs::read(&path).unwrap();

    std::fs::remove_file(&path).unwrap();

    let file = FormFile::from_bytes(&bytes).unwrap();

    assert_eq!(&file, writer.file());
    assert!(writer.write("/nonexistent-dir/x.form").is_err());
}

#[test]
fn composite_parts_are_not_written() {
    let mut engine = engine();
    let container = engine.add_widget(Container::new(point(0, 0), size(100, 100), size(300, 300)));
    let parts = engine
        .widgets()
        .get(container)
        .unwrap()
        .as_composite()
        .unwrap()
        .parts();

    assert!(parts.len() >= 3);

    let mut writer = writer();

    writer.register_type::<Container>("pushrod.container");

    let numbers = writer.add_form("c", engine.widgets(), container).unwrap();

    assert_eq!(numbers.len(), 1);
    assert_eq!(writer.file().objects.len(), 1);
}

#[test]
fn images_and_strings_are_stored_once() {
    let mut writer = writer();

    assert_eq!(writer.add_image("logo", &[1, 2, 3]), 0);
    assert_eq!(writer.add_image("logo", &[1, 2, 3]), 0);
    assert_eq!(writer.add_image("logo", &[9]), 1);

    let mut engine = engine();
    let mut panel = Panel::new();

    panel.set_size(size(100, 100));
    let panel = engine.add_widget(panel);

    for _ in 0..3 {
        let mut image = ImageWidget::new("logo");

        image.set_size(size(10, 10));
        engine.add_child_widget(panel, image);
    }

    writer.add_form("main", engine.widgets(), panel).unwrap();

    let images = writer
        .file()
        .objects
        .iter()
        .filter(|o| o.type_name == "pushrod.image")
        .count();

    assert_eq!(images, 3);
    assert_eq!(writer.file().images.len(), 2);

    // The image property is a reference to the pool, and the same bytes repeat one string.
    let bytes = writer.to_bytes().unwrap();
    let count = bytes.windows(10).filter(|w| *w == b"pushrod.im").count();

    assert_eq!(count, 1);
}

#[test]
fn symbols_use_the_returned_object_numbers() {
    let mut engine = engine();
    let panel = build(&mut engine);
    let ok = engine.widgets().children_of(panel)[2];
    let mut writer = writer();
    let numbers = writer.add_form("main", engine.widgets(), panel).unwrap();

    writer.add_symbol(SymbolKind::Id, numbers[&ok], "OK_BUTTON");

    let mut loaded = self::engine();
    let form = loaded.load_form(writer.file(), "main").unwrap();

    assert!(form.named("OK_BUTTON").is_some());
}

#[test]
fn child_layout_is_written_only_when_it_differs_from_the_default() {
    let mut engine = engine();
    let panel = build(&mut engine);
    let mut writer = writer();

    writer.add_form("main", engine.widgets(), panel).unwrap();

    let with_layout: Vec<bool> = writer
        .file()
        .objects
        .iter()
        .map(|o| {
            o.properties
                .iter()
                .any(|p| matches!(p, Property::ChildLayout(_)))
        })
        .collect();

    assert_eq!(with_layout, vec![false, false, false, true]);
}

#[test]
fn errors_name_the_problem_and_change_nothing() {
    let mut engine = engine();
    let panel = build(&mut engine);
    let mut empty = FormWriter::new();

    assert!(matches!(
        empty.add_form("main", engine.widgets(), panel),
        Err(Error::FormFile { .. })
    ));
    assert!(empty.file().objects.is_empty());
    assert!(empty.file().forms.is_empty());

    let mut writer = writer();

    assert!(matches!(
        writer.add_form("main", engine.widgets(), 9999),
        Err(Error::UnknownWidget { id: 9999 })
    ));

    writer.add_form("main", engine.widgets(), panel).unwrap();

    assert!(writer.add_form("main", engine.widgets(), panel).is_err());
    assert_eq!(writer.file().forms.len(), 1);
}

#[test]
fn a_layout_the_format_cannot_store_is_an_error() {
    let mut engine = engine();
    let mut panel = Panel::new();

    panel.set_size(size(50, 50));
    panel.set_layout(Box::new(Column::new().align(Align::Fill)));
    let panel = engine.add_widget(panel);

    assert!(matches!(
        writer().add_form("main", engine.widgets(), panel),
        Err(Error::FormFile { .. })
    ));
}

#[test]
fn anchors_are_stored_and_restored() {
    use pushrod::layout::Anchors;

    let mut built = engine();
    let mut panel = Panel::new();

    panel.set_size(size(200, 120));
    let panel = built.add_widget(panel);

    let mut ok = Button::new("OK");

    ok.set_size(size(60, 24));
    ok.set_anchors(Anchors::RIGHT | Anchors::BOTTOM);
    built.add_child_widget(panel, ok);

    let mut writer = writer();

    writer.add_form("main", built.widgets(), panel).unwrap();

    let file = FormFile::from_bytes(&writer.to_bytes().unwrap()).unwrap();
    let mut loaded = engine();
    let form = loaded.load_form(&file, "main").unwrap();
    let kids = loaded.widgets().children_of(form.root()).to_vec();

    assert_eq!(
        loaded.widgets().get(kids[0]).unwrap().anchors(),
        Anchors::RIGHT | Anchors::BOTTOM
    );
    assert_eq!(
        loaded.widgets().get(form.root()).unwrap().anchors(),
        Anchors::empty()
    );
}
