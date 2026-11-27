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
use pushrod::registry::{Properties, WidgetRegistry};
use pushrod::widget::Widget;

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
    registry.register("pushrod.button", |p| done(Button::new(text(p, "text")?)));
    registry.register("pushrod.check_box", |p| {
        let mut w = CheckBox::new(text(p, "text")?);

        w.set_checked(p.get_bool("checked").unwrap_or(false));
        done(w)
    });
    registry.register("pushrod.radio_button", |p| {
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
    registry.register("pushrod.glyph_button", |p| {
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
    registry.register("pushrod.slider", |p| {
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
    registry.register("pushrod.text_field", |_| done(TextField::new()));
    registry.register("pushrod.frame", |_| done(Frame::new()));
    registry.register("pushrod.panel", |_| done(Panel::new()));
}
