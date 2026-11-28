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

//! The showcase's own resource file: a form, strings, and an image, each found by a typed ID.
//!
//! The file is `resources/showcase.prsc`, embedded with `include_bytes!`.  [`build`] writes the
//! same bytes; the `make_resources` example saves them to disk, and a test fails when the file
//! and [`build`] disagree, so the file cannot go stale.

use pushrod::engine::Engine;
use pushrod::form::{FormFile, FormValue, ObjectRecord, Property};
use pushrod::resource::{FormId, ImageId, Resources, StringId};
use pushrod::widget::{Behavior, ROOT_WIDGET_ID, WidgetId};

/// The embedded resource file.
pub const FILE: &[u8] = include_bytes!("../resources/showcase.prsc");

/// The name of the contact form in the file.
pub const CONTACT_FORM_NAME: &str = "ContactForm";

/// The contact form's ID: a form's ID is its 1-based position in the file.
pub const CONTACT_FORM: FormId = FormId(1);

/// The ID of the string `FORM_TITLE`.
pub const FORM_TITLE: StringId = StringId(1);

/// The ID of the string `GREETING`.
pub const GREETING: StringId = StringId(2);

/// The ID of the string `GOODBYE`.
pub const GOODBYE: StringId = StringId(3);

/// The ID of the image `LOGO`.
pub const LOGO: ImageId = ImageId(1);

/// A 1 by 1 pixel transparent PNG, the stand-in for a real logo.
const LOGO_PNG: [u8; 67] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// Adds one widget to the contact form.
///
/// # Arguments
///
/// * `file` - The file being built.
/// * `parent` - The parent's object number.
/// * `type_name` - The registered widget type.
/// * `bounds` - Left, top, width, and height.
/// * `text` - The `text` property, if the widget has one.
/// * `behavior` - Behavior flags, or empty to keep the widget's own.
///
/// # Returns
///
/// The object number.
fn add(
    file: &mut FormFile,
    parent: u32,
    type_name: &str,
    bounds: (i32, i32, u32, u32),
    text: Option<&str>,
    behavior: Behavior,
) -> u32 {
    let mut record = ObjectRecord::new(type_name);

    record.x = bounds.0;
    record.y = bounds.1;
    record.width = bounds.2;
    record.height = bounds.3;
    record.behavior = u32::from(behavior.bits());

    if let Some(text) = text {
        record
            .properties
            .push(Property::named("text", FormValue::Text(text.to_string())));
    }

    file.add_object(Some(parent), record)
}

/// Builds the resource file's bytes.
///
/// # Returns
///
/// The bytes of `resources/showcase.prsc`.
pub fn build() -> Vec<u8> {
    let mut file = FormFile::new();
    let root = file.add_object(None, ObjectRecord::new("pushrod.panel"));

    file.objects[root as usize].width = 360;
    file.objects[root as usize].height = 150;

    add(
        &mut file,
        root,
        "pushrod.text",
        (10, 14, 70, 18),
        Some("Name"),
        Behavior::empty(),
    );
    add(
        &mut file,
        root,
        "pushrod.text_field",
        (90, 10, 250, 24),
        None,
        Behavior::empty(),
    );
    add(
        &mut file,
        root,
        "pushrod.text",
        (10, 54, 70, 18),
        Some("Email"),
        Behavior::empty(),
    );
    add(
        &mut file,
        root,
        "pushrod.text_field",
        (90, 50, 250, 24),
        None,
        Behavior::empty(),
    );
    add(
        &mut file,
        root,
        "pushrod.button",
        (180, 106, 76, 26),
        Some("OK"),
        Behavior::FOCUSABLE | Behavior::DEFAULT_ACTION,
    );
    add(
        &mut file,
        root,
        "pushrod.button",
        (264, 106, 76, 26),
        Some("Cancel"),
        Behavior::FOCUSABLE | Behavior::CANCEL_ACTION,
    );

    file.add_form(CONTACT_FORM_NAME, root);

    // The order of these calls fixes the IDs above.
    let strings = &mut file.resources.strings;

    strings
        .add("FORM_TITLE", "Contact form".to_string())
        .unwrap();
    strings
        .add("GREETING", "Hello, Jane Doe".to_string())
        .unwrap();
    strings.add("GOODBYE", "See you soon".to_string()).unwrap();
    file.resources
        .images
        .add("LOGO", LOGO_PNG.to_vec())
        .unwrap();

    file.to_bytes()
        .expect("the showcase resource file is valid")
}

/// Loads the embedded resource file.
///
/// # Returns
///
/// The resources, searched by typed ID or by name.
pub fn load() -> Resources {
    Resources::from_bytes(FILE).expect("the embedded showcase resource file is valid")
}

/// Hands the engine the resources and loads the contact form once, hidden on a shelf.  The
/// Forms page borrows the loaded form for as long as it shows (a `Ui` cannot load forms).
///
/// # Arguments
///
/// * `engine` - The engine to add the resources and the form to.
///
/// # Returns
///
/// The shelf's ID, or `None` when the form cannot be loaded.
pub fn preload<S>(engine: &mut Engine<S>) -> Option<WidgetId> {
    // A form names its widgets by type, so the engine must know the standard ones.
    pushrod_widgets::register(engine.widget_registry());
    engine.set_resources(load());

    let shelf = engine.widgets().add_child(
        ROOT_WIDGET_ID,
        Box::new(crate::property_panel::PageRoot::new(
            pushrod::geometry::size(0, 0),
        )),
    );

    engine.widgets().get_mut(shelf)?.set_hidden(true);

    let form = engine.load_resource_form(CONTACT_FORM).ok()?;

    engine.widgets().reparent(form.root(), shelf).ok()?;
    Some(shelf)
}

/// Finds the shelf: the page root that is a direct child of the window root.
///
/// # Arguments
///
/// * `cache` - The widget tree.
///
/// # Returns
///
/// The shelf's ID, or `None` when [`preload`] did not run.
pub fn shelf(cache: &pushrod::cache::WidgetCache) -> Option<WidgetId> {
    cache
        .children_of(ROOT_WIDGET_ID)
        .iter()
        .copied()
        .find(|id| {
            cache
                .get_as::<crate::property_panel::PageRoot>(*id)
                .is_some()
        })
}
