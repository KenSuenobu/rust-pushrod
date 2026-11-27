// Widget Property Tests
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

//! Tests that every standard widget lists and sets its properties by name.

use pushrod::color::Color;
use pushrod::error::Error;
use pushrod::geometry::{point, size};
use pushrod::registry::PropertyValue;
use pushrod::widget::Widget;
use pushrod_widgets::prelude::*;

/// Builds one of every standard widget, with its name for failure messages.
fn every_widget() -> Vec<(&'static str, Box<dyn Widget>)> {
    vec![
        (
            "Alert",
            Box::new(Alert::new(AlertKind::Info, "T", "M", &["OK"], 0)),
        ),
        ("BoxedLabel", Box::new(BoxedLabel::new("Label"))),
        ("Button", Box::new(Button::new("Go"))),
        ("CheckBox", Box::new(CheckBox::new("Agree"))),
        (
            "Container",
            Box::new(Container::new(point(0, 0), size(50, 50), size(100, 100))),
        ),
        (
            "CustomPaint",
            Box::new(CustomPaint::new(point(0, 0), size(10, 10), |_, _, _| {})),
        ),
        ("Frame", Box::new(Frame::new())),
        ("GlyphButton", Box::new(GlyphButton::new('+'))),
        ("GroupBox", Box::new(GroupBox::new("Group"))),
        ("ImageWidget", Box::new(ImageWidget::new("missing.png"))),
        ("Panel", Box::new(Panel::new())),
        ("ProgressBar", Box::new(ProgressBar::new(100.0))),
        ("RadioButton", Box::new(RadioButton::new("One"))),
        ("ScrollBar", Box::new(ScrollBar::vertical())),
        ("Separator", Box::new(Separator::horizontal(50))),
        ("Slider", Box::new(Slider::new(0.0, 10.0))),
        ("Text", Box::new(Text::new("Hi"))),
        ("TextField", Box::new(TextField::new())),
    ]
}

#[test]
fn every_built_in_lists_the_core_properties() {
    for (name, widget) in every_widget() {
        let listed = widget.properties();

        for core in [
            "x", "y", "width", "height", "color", "hidden", "enabled", "tooltip",
        ] {
            assert!(listed.iter().any(|p| p.name == core), "{name}: {core}");
        }
    }
}

#[test]
fn every_listed_property_round_trips_without_a_change() {
    for (name, mut widget) in every_widget() {
        for property in widget.properties() {
            let result = widget.set_property(&property.name, &property.value);

            match result {
                Ok(changed) => assert!(!changed, "{name}.{}", property.name),
                Err(Error::Property { message, .. }) => {
                    assert!(message.contains("read only"), "{name}.{}", property.name)
                }
                Err(other) => panic!("{name}.{}: {other}", property.name),
            }
        }
    }
}

#[test]
fn every_listed_property_keeps_its_kind_and_list() {
    for (name, widget) in every_widget() {
        let first = widget.properties();
        let second = widget.properties();

        assert_eq!(first, second, "{name}");

        for property in first {
            assert_eq!(
                pushrod::properties::PropertyKind::of(&property.value),
                property.kind,
                "{name}.{}",
                property.name
            );
        }
    }
}

#[test]
fn every_built_in_rejects_an_unknown_property() {
    for (name, mut widget) in every_widget() {
        assert!(
            widget
                .set_property("no_such", &PropertyValue::Int(1))
                .is_err(),
            "{name}"
        );
    }
}

/// Sets a property, asserts that it changed, and reads it back.
fn change(widget: &mut dyn Widget, name: &str, value: PropertyValue) {
    assert_eq!(widget.set_property(name, &value), Ok(true), "{name}");

    let listed = widget
        .properties()
        .into_iter()
        .find(|p| p.name == name)
        .unwrap();

    assert_eq!(listed.value, value, "{name}");
}

#[test]
fn text_widgets_set_their_text() {
    let mut button = Button::new("Go");
    let mut boxed = BoxedLabel::new("Label");
    let mut text = Text::new("Hi");
    let mut field = TextField::new();
    let mut group = GroupBox::new("Group");

    change(&mut button, "text", PropertyValue::Text("Stop".into()));
    change(&mut button, "checked", PropertyValue::Bool(true));
    change(&mut button, "border_width", PropertyValue::Int(3));
    change(&mut boxed, "text", PropertyValue::Text("New".into()));
    change(&mut boxed, "border_color", PropertyValue::Color(Color::RED));
    change(&mut text, "text", PropertyValue::Text("Bye".into()));
    change(&mut text, "font_color", PropertyValue::Color(Color::BLUE));
    change(&mut field, "text", PropertyValue::Text("Typed".into()));
    change(&mut group, "title", PropertyValue::Text("Other".into()));
    change(&mut group, "border_color", PropertyValue::Color(Color::RED));

    assert_eq!(button.get_text(), "Stop");
    assert!(button.is_checked());
    assert_eq!(boxed.get_text(), "New");
    assert_eq!(text.get_text(), "Bye");
    assert_eq!(field.text(), "Typed");
    assert_eq!(group.get_title(), "Other");
}

#[test]
fn toggle_widgets_set_their_state() {
    let mut check = CheckBox::new("Agree");
    let mut radio = RadioButton::new("One");

    change(&mut check, "text", PropertyValue::Text("Accept".into()));
    change(&mut check, "checked", PropertyValue::Bool(true));
    change(&mut check, "indeterminate", PropertyValue::Bool(true));
    change(&mut radio, "text", PropertyValue::Text("Two".into()));
    change(&mut radio, "checked", PropertyValue::Bool(true));

    assert!(check.is_checked() || check.is_indeterminate());
    assert_eq!(radio.get_text(), "Two");
    assert!(radio.is_checked());
}

#[test]
fn number_widgets_set_and_clamp_their_values() {
    let mut slider = Slider::new(0.0, 10.0);
    let mut bar = ProgressBar::new(100.0);
    let mut scroll = ScrollBar::vertical();

    change(&mut slider, "max", PropertyValue::Float(20.0));
    change(&mut slider, "step", PropertyValue::Float(2.0));
    change(&mut slider, "value", PropertyValue::Float(8.0));
    // Past the range is clamped, so setting 99 reports the clamped result.
    assert_eq!(
        slider.set_property("value", &PropertyValue::Float(99.0)),
        Ok(true)
    );
    assert_eq!(slider.get_value(), 20.0);
    assert_eq!(
        slider.set_property("value", &PropertyValue::Float(99.0)),
        Ok(false)
    );
    assert!(
        slider
            .set_property("vertical", &PropertyValue::Bool(true))
            .is_err()
    );

    change(&mut bar, "max", PropertyValue::Float(50.0));
    change(&mut bar, "value", PropertyValue::Float(25.0));
    change(&mut bar, "outline", PropertyValue::Bool(true));
    change(&mut bar, "indeterminate", PropertyValue::Bool(true));
    assert!(bar.is_indeterminate());
    assert_eq!(
        bar.set_property("indeterminate", &PropertyValue::Bool(false)),
        Ok(true)
    );
    assert_eq!(bar.get_value(), Some(0.0));

    change(&mut scroll, "page_size", PropertyValue::Float(0.4));
    change(&mut scroll, "step", PropertyValue::Float(0.3));
    change(&mut scroll, "value", PropertyValue::Float(0.5));
    assert!(
        scroll
            .set_property("vertical", &PropertyValue::Bool(false))
            .is_err()
    );
}

#[test]
fn glyph_image_frame_separator_and_container_set_theirs() {
    let mut glyph = GlyphButton::new('+');
    let mut image = ImageWidget::new("a.png");
    let mut frame = Frame::new();
    let mut rule = Separator::horizontal(50);
    let mut container = Container::new(point(0, 0), size(50, 50), size(100, 100));

    change(&mut glyph, "glyph", PropertyValue::Text("-".into()));
    assert!(
        glyph
            .set_property("glyph", &PropertyValue::Text("ab".into()))
            .is_err()
    );
    assert!(
        glyph
            .set_property("glyph", &PropertyValue::Text(String::new()))
            .is_err()
    );
    change(&mut image, "path", PropertyValue::Text("b.png".into()));
    change(&mut frame, "border_width", PropertyValue::Int(4));
    assert!(
        frame
            .set_property("border_width", &PropertyValue::Int(300))
            .is_err()
    );
    change(&mut rule, "etched", PropertyValue::Bool(true));
    change(&mut rule, "line_color", PropertyValue::Color(Color::RED));
    change(&mut container, "line_step", PropertyValue::Int(30));

    assert_eq!(glyph.get_glyph(), '-');
    assert_eq!(image.path(), "b.png");
    assert!(rule.is_etched());
    assert_eq!(container.get_line_step(), 30);
}
