// Standard Widget Registration
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

//! # Registering the standard widgets
//!
//! [`register`] adds every standard widget that can be built from properties alone to a
//! [`WidgetRegistry`], under the `pushrod.` prefix, so form files can create them by type name.
//!
//! | Type name | Properties (besides `x`, `y`, `width`, `height`, `hidden`, `color`) |
//! | --- | --- |
//! | `pushrod.button` | `text` |
//! | `pushrod.check_box` | `text`, `checked` |
//! | `pushrod.radio_button` | `text`, `checked` |
//! | `pushrod.text` | `text` |
//! | `pushrod.boxed_label` | `text` |
//! | `pushrod.group_box` | `title` |
//! | `pushrod.glyph_button` | `glyph` (one character) |
//! | `pushrod.image` | `path` |
//! | `pushrod.progress_bar` | `max` (default 100), `value` |
//! | `pushrod.slider` | `min` (default 0), `max` (default 100), `step`, `value`, `vertical` |
//! | `pushrod.separator` | `vertical` |
//! | `pushrod.text_field` | none |
//! | `pushrod.frame` | none |
//! | `pushrod.panel` | none |
//!
//! Each type also declares its events (see [`WidgetRegistry::events`]): `Clicked` for `button` and
//! `glyph_button`; `Toggled` for `check_box` and `radio_button`; `ValueChanged` and
//! `EditingFinished` for `slider`; `TextChanged` and `EditingFinished` for `text_field`.  The
//! others (`image` is display only when built from properties) only receive the shared pointer, key, and focus events.
//!
//! [`register_form_types`] teaches a [`FormWriter`] the same names, so saved forms load again.
//!
//! Widgets that need code, not data (`CustomPaint`), or that are built from other widgets
//! (`Container`, `ScrollBar`, `Alert`) are not registered.
//!
//! ```rust
//! use pushrod::registry::{Properties, WidgetRegistry};
//!
//! let mut registry = WidgetRegistry::new();
//!
//! pushrod_widgets::register(&mut registry);
//!
//! let ok = registry
//!     .create("pushrod.button", &Properties::new().with_text("text", "OK").with_int("width", 80))
//!     .unwrap();
//!
//! assert_eq!(ok.get_size().w, 80);
//! ```

use crate::prelude::*;
use pushrod::error::{Error, Result};
use pushrod::event::EventKind;
use pushrod::form::FormWriter;
use pushrod::registry::{EventInfo, PayloadField, Properties, WidgetRegistry};
use pushrod::widget::Widget;

/// The `Clicked` event of a button-like widget.
const CLICKED: &[EventInfo] = &[EventInfo {
    name: "Clicked",
    description: "The widget was clicked.",
    kind: EventKind::Clicked,
    payload: &[
        PayloadField {
            name: "button",
            type_name: "MouseButton",
        },
        PayloadField {
            name: "count",
            type_name: "u8",
        },
    ],
    is_default: true,
    callback: Some("on_click"),
}];

/// The `Toggled` event of a check box or radio button.
const TOGGLED: &[EventInfo] = &[EventInfo {
    name: "Toggled",
    description: "The checked state changed.",
    kind: EventKind::Toggled,
    payload: &[PayloadField {
        name: "checked",
        type_name: "bool",
    }],
    is_default: true,
    callback: Some("on_toggle"),
}];

/// The events of a slider.
const SLIDER_EVENTS: &[EventInfo] = &[
    EventInfo {
        name: "ValueChanged",
        description: "The value changed.",
        kind: EventKind::ValueChanged,
        payload: &[],
        is_default: true,
        callback: Some("on_value_change"),
    },
    EventInfo {
        name: "EditingFinished",
        description: "The user let go of the slider or left it.",
        kind: EventKind::EditingFinished,
        payload: &[],
        is_default: false,
        callback: Some("on_editing_finished"),
    },
];

/// The events of a text field.
const TEXT_FIELD_EVENTS: &[EventInfo] = &[
    EventInfo {
        name: "TextChanged",
        description: "The text changed.",
        kind: EventKind::TextChanged,
        payload: &[],
        is_default: true,
        callback: Some("on_text_change"),
    },
    EventInfo {
        name: "EditingFinished",
        description: "Return was pressed or the field lost focus.",
        kind: EventKind::EditingFinished,
        payload: &[],
        is_default: false,
        callback: Some("on_editing_finished"),
    },
];

/// Returns a required text property.
///
/// # Arguments
///
/// * `props` - The properties.
/// * `name` - The property name.
///
/// # Returns
///
/// The text.
///
/// # Errors
///
/// Returns [`Error::Property`] if the property is missing or is not text.
fn text<'a>(props: &'a Properties, name: &str) -> Result<&'a str> {
    props
        .get_text(name)
        .ok_or_else(|| Error::property(name, "required text property is missing"))
}

/// Boxes a constructed widget.
///
/// # Arguments
///
/// * `widget` - The widget.
///
/// # Returns
///
/// The widget as a trait object, wrapped in `Ok`.
fn done<W: Widget>(widget: W) -> Result<Box<dyn Widget>> {
    Ok(Box::new(widget))
}

/// Registers the standard widgets under the `pushrod.` prefix.
///
/// Calling it again, or registering one of these names afterward, replaces the constructor, so
/// another library can restyle a standard widget for existing forms.
///
/// # Arguments
///
/// * `registry` - The registry to add the widgets to.
pub fn register(registry: &mut WidgetRegistry) {
    registry.register_with_events("pushrod.button", CLICKED, |p| {
        done(Button::new(text(p, "text")?))
    });
    registry.register_with_events("pushrod.check_box", TOGGLED, |p| {
        let mut w = CheckBox::new(text(p, "text")?);

        w.set_checked(p.get_bool("checked").unwrap_or(false));
        done(w)
    });
    registry.register_with_events("pushrod.radio_button", TOGGLED, |p| {
        let mut w = RadioButton::new(text(p, "text")?);

        w.set_checked(p.get_bool("checked").unwrap_or(false));
        done(w)
    });
    registry.register("pushrod.text", |p| done(Text::new(text(p, "text")?)));
    registry.register("pushrod.boxed_label", |p| {
        done(BoxedLabel::new(text(p, "text")?))
    });
    registry.register("pushrod.group_box", |p| {
        done(GroupBox::new(text(p, "title")?))
    });
    registry.register_with_events("pushrod.glyph_button", CLICKED, |p| {
        let mut chars = text(p, "glyph")?.chars();

        match (chars.next(), chars.next()) {
            (Some(glyph), None) => done(GlyphButton::new(glyph)),
            _ => Err(Error::property("glyph", "must be exactly one character")),
        }
    });
    registry.register("pushrod.image", |p| {
        done(ImageWidget::new(text(p, "path")?))
    });
    registry.register("pushrod.progress_bar", |p| {
        let mut w = ProgressBar::new(p.get_float("max").unwrap_or(100.0));

        if let Some(value) = p.get_float("value") {
            w.set_value(Some(value));
        }

        done(w)
    });
    registry.register_with_events("pushrod.slider", SLIDER_EVENTS, |p| {
        let min = p.get_float("min").unwrap_or(0.0);
        let max = p.get_float("max").unwrap_or(100.0);
        let mut w = Slider::new(min, max);

        if p.get_bool("vertical").unwrap_or(false) {
            w = w.vertical();
        }

        if let Some(step) = p.get_float("step") {
            w.set_step(step);
        }

        if let Some(value) = p.get_float("value") {
            w.set_value(value);
        }

        done(w)
    });
    registry.register("pushrod.separator", |p| {
        if p.get_bool("vertical").unwrap_or(false) {
            done(Separator::vertical(1))
        } else {
            done(Separator::horizontal(1))
        }
    });
    registry.register_with_events("pushrod.text_field", TEXT_FIELD_EVENTS, |_| {
        done(TextField::new())
    });
    registry.register("pushrod.frame", |_| done(Frame::new()));
    registry.register("pushrod.panel", |_| done(Panel::new()));
}

/// Tells a form writer which type name each standard widget is saved as, the same names
/// [`register`] uses, and that the `path` property of an image widget names an image.
///
/// # Arguments
///
/// * `writer` - The writer to teach the standard widget types.
pub fn register_form_types(writer: &mut FormWriter) {
    writer.register_type::<Button>("pushrod.button");
    writer.register_type::<CheckBox>("pushrod.check_box");
    writer.register_type::<RadioButton>("pushrod.radio_button");
    writer.register_type::<Text>("pushrod.text");
    writer.register_type::<BoxedLabel>("pushrod.boxed_label");
    writer.register_type::<GroupBox>("pushrod.group_box");
    writer.register_type::<GlyphButton>("pushrod.glyph_button");
    writer.register_type::<ImageWidget>("pushrod.image");
    writer.register_type::<ProgressBar>("pushrod.progress_bar");
    writer.register_type::<Slider>("pushrod.slider");
    writer.register_type::<Separator>("pushrod.separator");
    writer.register_type::<TextField>("pushrod.text_field");
    writer.register_type::<Frame>("pushrod.frame");
    writer.register_type::<Panel>("pushrod.panel");
    writer.image_property("path");
}
