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

//! The core feature pages: appearance, focus, dialogs, form layout, and resource forms.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::geometry::{Point, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::resource::{FormId, ImageId, StringId};
use pushrod::widget::WidgetId;
use pushrod_showcase::resources;
use pushrod_showcase::shell::{self, ShowcaseState};
use pushrod_widgets::alert::Alert;
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// Builds the shell headlessly and runs the first idle pass.
fn start() -> (Engine<ShowcaseState>, ShowcaseState) {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(1100, 1600), 30);
    let mut state = shell::build(&mut engine, None);

    engine.run_idle(&mut state);
    settle(&mut engine);
    (engine, state)
}

/// Draws frames until the layout settles.
fn settle(engine: &mut Engine<ShowcaseState>) {
    let mut backend = HeadlessBackend::new(size(1100, 1600));

    for _ in 0..3 {
        engine.widgets().draw_loop(&mut backend);
    }
}

/// Opens a page by name.
fn open(engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, name: &str) -> WidgetId {
    let (button, _) = state
        .buttons
        .iter()
        .copied()
        .find(|(_, index)| state.registry.get(*index).unwrap().name() == name)
        .unwrap_or_else(|| panic!("no page named {name}"));

    click_widget(engine, state, button);
    state.page_root.unwrap()
}

/// Returns the center of a widget, in window coordinates.
fn center(engine: &mut Engine<ShowcaseState>, id: WidgetId) -> Point {
    let origin = engine.widgets().absolute_origin(id);
    let extent = engine.widgets().get(id).unwrap().get_size();

    point(
        origin.x + extent.w as i32 / 2,
        origin.y + extent.h as i32 / 2,
    )
}

/// Clicks the center of a widget.
fn click_widget(engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, id: WidgetId) {
    let position = center(engine, id);

    click_at(engine, state, position);
}

/// Clicks a point with the left mouse button.
fn click_at(engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, position: Point) {
    for down in [true, false] {
        let event = if down {
            InputEvent::MouseDown {
                position,
                button: MouseButton::Left,
                clicks: 1,
            }
        } else {
            InputEvent::MouseUp {
                position,
                button: MouseButton::Left,
                clicks: 1,
            }
        };

        engine.dispatch(event, state);
    }

    settle(engine);
}

/// Returns the children of a widget that are of one type.
fn children_of_type<T: pushrod::widget::Widget>(
    engine: &mut Engine<ShowcaseState>,
    parent: WidgetId,
) -> Vec<WidgetId> {
    let cache = engine.widgets();

    cache
        .children_of(parent)
        .iter()
        .copied()
        .filter(|id| cache.get_as::<T>(*id).is_some())
        .collect()
}

#[test]
fn the_resource_file_on_disk_matches_the_builder() {
    assert_eq!(resources::FILE, resources::build().as_slice());
}

#[test]
fn typed_ids_match_the_names_in_the_file() {
    let set = resources::load();

    assert_eq!(
        set.find::<StringId>("FORM_TITLE"),
        Some(resources::FORM_TITLE)
    );
    assert_eq!(set.find::<StringId>("GREETING"), Some(resources::GREETING));
    assert_eq!(set.find::<StringId>("GOODBYE"), Some(resources::GOODBYE));
    assert_eq!(set.find::<ImageId>("LOGO"), Some(resources::LOGO));
    assert_eq!(
        set.find::<FormId>(resources::CONTACT_FORM_NAME),
        Some(resources::CONTACT_FORM)
    );
    assert_eq!(set.string(resources::GREETING), Some("Hello, Jane Doe"));
}

#[test]
fn the_resource_form_page_shows_the_loaded_form_every_visit() {
    let (mut engine, mut state) = start();

    for _ in 0..2 {
        let root = open(&mut engine, &mut state, "Form from a Resource File");
        let forms = children_of_type::<Panel>(&mut engine, root);

        assert_eq!(forms.len(), 1, "the form is on the page");
        assert_eq!(
            children_of_type::<TextField>(&mut engine, forms[0]).len(),
            2
        );
        assert_eq!(children_of_type::<Button>(&mut engine, forms[0]).len(), 2);

        open(&mut engine, &mut state, "Text");
    }
}

#[test]
fn the_resources_page_reads_strings_by_id() {
    let (mut engine, mut state) = start();
    let root = open(&mut engine, &mut state, "Strings and Images by ID");
    let texts: Vec<String> = children_of_type::<Text>(&mut engine, root)
        .into_iter()
        .map(|id| engine.widgets().get_as::<Text>(id).unwrap().get_text())
        .collect();

    assert!(
        texts
            .iter()
            .any(|t| t.ends_with("GREETING: Hello, Jane Doe"))
    );
    assert!(
        texts
            .iter()
            .any(|t| t.starts_with("ImageId(1) LOGO: 67 bytes"))
    );
}

#[test]
fn the_form_layout_reflows_when_the_form_is_narrowed() {
    let (mut engine, mut state) = start();
    let root = open(&mut engine, &mut state, "Form Layout");
    let slider = children_of_type::<Slider>(&mut engine, root)[0];
    let form = children_of_type::<Panel>(&mut engine, root)[0];
    let field = children_of_type::<TextField>(&mut engine, form)[0];
    let wide_x = engine.widgets().get(field).unwrap().get_origin().x;

    // Press the far left of the slider's track: the knob jumps to the narrowest width.
    let origin = engine.widgets().absolute_origin(slider);
    let target = point(origin.x + 1, origin.y + 12);

    click_at(&mut engine, &mut state, target);
    settle(&mut engine);

    assert!(engine.widgets().get(form).unwrap().get_size().w < 440);
    assert!(
        engine.widgets().get(field).unwrap().get_origin().x < wide_x,
        "the field moves under its label when the form is narrow"
    );
}

#[test]
fn a_dialog_page_button_opens_a_modal_alert_and_reports_the_choice() {
    let (mut engine, mut state) = start();
    let root = open(&mut engine, &mut state, "Dialogs and Alerts");
    let buttons = children_of_type::<Button>(&mut engine, root);

    click_widget(&mut engine, &mut state, buttons[1]);

    let modal = engine.widgets().top_modal().expect("the alert is open");
    let cancel = *alert_parts(&mut engine, modal).last().unwrap();

    assert!(
        engine
            .widgets()
            .get_as::<Alert>(modal)
            .unwrap()
            .button_index(cancel)
            .is_some()
    );

    click_widget(&mut engine, &mut state, cancel);

    assert!(engine.widgets().top_modal().is_none());

    let result = children_of_type::<Text>(&mut engine, root)
        .into_iter()
        .map(|id| engine.widgets().get_as::<Text>(id).unwrap().get_text())
        .collect::<Vec<_>>();

    assert!(
        result.iter().any(|t| t == "Cancel (button 1)"),
        "{result:?}"
    );
}

/// Returns an alert's button IDs, left to right.
fn alert_parts(engine: &mut Engine<ShowcaseState>, alert: WidgetId) -> Vec<WidgetId> {
    engine
        .widgets()
        .get_as::<Alert>(alert)
        .unwrap()
        .typed_parts()
        .buttons
}
