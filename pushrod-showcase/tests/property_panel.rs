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

//! The property panel: selecting a page widget, editing its properties live, and Reset.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::geometry::{Point, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::WidgetId;
use pushrod_showcase::property_panel::EditorKind;
use pushrod_showcase::shell::{self, ShowcaseState};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::text_field::TextField;
use std::time::Instant;

type Harness = (Engine<ShowcaseState>, ShowcaseState);

/// Builds the shell headlessly, opens the named page, and settles the layout.
fn start(page: &str) -> Harness {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(900, 640), 30);
    let mut state = shell::build(&mut engine, None);

    engine.run_idle(&mut state);
    settle(&mut engine);

    let index = state.registry.find(page).unwrap();
    let (button, _) = state
        .buttons
        .iter()
        .copied()
        .find(|(_, i)| *i == index)
        .unwrap();

    click(&mut engine, &mut state, button);
    (engine, state)
}

/// Draws frames until the layout settles.
fn settle(engine: &mut Engine<ShowcaseState>) {
    let mut backend = HeadlessBackend::new(size(900, 640));

    for _ in 0..3 {
        engine.widgets().draw_loop(&mut backend);
    }
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

/// Clicks a widget with the left mouse button, then lets the layout settle.
fn click(engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, id: WidgetId) {
    let position = center(engine, id);

    engine.dispatch(
        InputEvent::MouseDown {
            position,
            button: MouseButton::Left,
            clicks: 1,
        },
        state,
    );
    engine.dispatch(
        InputEvent::MouseUp {
            position,
            button: MouseButton::Left,
            clicks: 1,
        },
        state,
    );
    // Custom events, such as the page root's pick request, are delivered with the next round.
    engine.advance(Instant::now(), state);
    settle(engine);
}

/// Returns the page's button with the given label.
fn page_button(engine: &mut Engine<ShowcaseState>, state: &ShowcaseState, label: &str) -> WidgetId {
    let root = state.page_root.unwrap();

    engine
        .widgets()
        .children_of(root)
        .to_vec()
        .into_iter()
        .find(|id| {
            engine
                .widgets()
                .get_as::<Button>(*id)
                .is_some_and(|b| b.get_text() == label)
        })
        .unwrap()
}

/// Scrolls the property list until the editor's center is inside the list's frame, as a user
/// would with the wheel.
fn reveal(engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, editor: WidgetId) {
    let frame = engine.widgets().get(state.panel.list).unwrap().get_parent();
    let top = engine.widgets().absolute_origin(frame);
    let height = engine.widgets().get(frame).unwrap().get_size().h as i32;
    let over = point(top.x + 20, top.y + 20);

    engine.dispatch(InputEvent::MouseMove { position: over }, state);

    for _ in 0..200 {
        let middle = center(engine, editor);

        if middle.y >= top.y && middle.y < top.y + height {
            return;
        }

        let step = if middle.y >= top.y + height { -1 } else { 1 };

        engine.dispatch(
            InputEvent::Wheel {
                delta: point(0, step),
            },
            state,
        );
        settle(engine);
    }

    panic!("could not scroll the editor into view");
}

/// Returns the editor widget of a property of the selected widget.
fn editor(state: &ShowcaseState, name: &str) -> WidgetId {
    state
        .panel
        .editors
        .iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| panic!("no editor for {name}"))
        .id
}

#[test]
fn nothing_is_selected_at_first() {
    let (_, state) = start("Button");

    assert_eq!(state.panel.selected, None);
    assert!(state.panel.editors.is_empty());
    assert!(state.panel.outline.is_empty());
}

#[test]
fn clicking_a_widget_selects_and_outlines_it() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    assert_eq!(state.panel.selected, Some(normal));
    assert_eq!(state.panel.outline.len(), 4);

    // Every property the widget lists has an editor, whatever the widget is.
    let listed = engine.widgets().get(normal).unwrap().properties();

    assert_eq!(state.panel.editors.len(), listed.len());
    assert_eq!(
        state
            .panel
            .editors
            .iter()
            .find(|e| e.name == "enabled")
            .unwrap()
            .kind,
        EditorKind::Check
    );
    assert_eq!(
        state
            .panel
            .editors
            .iter()
            .find(|e| e.name == "align_horizontal")
            .unwrap()
            .kind,
        EditorKind::Cycle
    );
}

#[test]
fn clicking_the_outline_or_the_page_background_keeps_the_selection() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let strip = state.panel.outline[0];

    click(&mut engine, &mut state, strip);

    assert_eq!(state.panel.selected, Some(normal));

    // Press the page's top-left corner, where there is no widget.
    let root = state.page_root.unwrap();
    let corner = engine.widgets().absolute_origin(root);

    for down in [true, false] {
        let position = point(corner.x + 1, corner.y + 1);
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

        engine.dispatch(event, &mut state);
    }

    engine.advance(Instant::now(), &mut state);

    assert_eq!(state.panel.selected, Some(normal));
}

#[test]
fn a_check_box_edit_applies_at_once() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let enabled = editor(&state, "enabled");

    assert!(
        engine
            .widgets()
            .get_as::<CheckBox>(enabled)
            .unwrap()
            .is_checked()
    );

    reveal(&mut engine, &mut state, enabled);
    click(&mut engine, &mut state, enabled);

    assert!(!engine.widgets().get(normal).unwrap().is_enabled());
}

#[test]
fn a_number_edit_applies_and_a_bad_one_is_ignored() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let width = editor(&state, "width");

    click(&mut engine, &mut state, width);
    assert!(engine.widgets().get_as::<TextField>(width).is_some());

    // The field holds "120"; typing "5" makes "1205".
    engine.dispatch(InputEvent::Text("5".into()), &mut state);
    assert_eq!(engine.widgets().get(normal).unwrap().get_size().w, 1205);

    // Not a number: the widget keeps its width.
    engine.dispatch(InputEvent::Text("x".into()), &mut state);
    assert_eq!(engine.widgets().get(normal).unwrap().get_size().w, 1205);
}

#[test]
fn a_choice_button_cycles() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let align = editor(&state, "align_horizontal");

    reveal(&mut engine, &mut state, align);

    let before = engine.widgets().get_as::<Button>(align).unwrap().get_text();

    click(&mut engine, &mut state, align);

    let after = engine.widgets().get_as::<Button>(align).unwrap().get_text();

    assert_ne!(before, after);

    let value = engine
        .widgets()
        .get(normal)
        .unwrap()
        .properties()
        .into_iter()
        .find(|p| p.name == "align_horizontal")
        .unwrap()
        .value;

    assert_eq!(value, pushrod::registry::PropertyValue::Text(after));
}

#[test]
fn reset_restores_the_page() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let enabled = editor(&state, "enabled");

    reveal(&mut engine, &mut state, enabled);
    click(&mut engine, &mut state, enabled);
    assert!(!engine.widgets().get(normal).unwrap().is_enabled());

    let reset = state.panel.reset;

    click(&mut engine, &mut state, reset);

    assert_eq!(state.panel.selected, None);
    assert!(state.panel.editors.is_empty());

    let fresh = page_button(&mut engine, &state, "Normal");

    assert!(engine.widgets().get(fresh).unwrap().is_enabled());
}

#[test]
fn it_works_on_another_widget_type() {
    let (mut engine, mut state) = start("Slider");
    let root = state.page_root.unwrap();
    let children = engine.widgets().children_of(root).to_vec();
    let slider = children
        .into_iter()
        .find(|id| {
            let widgets = engine.widgets();

            widgets
                .get(*id)
                .unwrap()
                .properties()
                .iter()
                .any(|p| p.name == "value")
        })
        .unwrap();

    click(&mut engine, &mut state, slider);

    assert_eq!(state.panel.selected, Some(slider));
    assert!(state.panel.editors.iter().any(|e| e.name == "value"));
}

#[test]
fn the_panel_follows_the_mockup_layout() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let frame = engine.widgets().get(state.panel.list).unwrap().get_parent();
    let at = |engine: &mut Engine<ShowcaseState>, id: WidgetId| {
        let origin = engine.widgets().absolute_origin(id);
        let extent = engine.widgets().get(id).unwrap().get_size();

        (origin, extent)
    };
    let (page, page_size) = at(&mut engine, state.shell.page_area);
    let (list, list_size) = at(&mut engine, frame);
    let (status, _) = at(&mut engine, state.panel.status);
    let (reset, _) = at(&mut engine, state.panel.reset);

    // A third column to the right of the page, not a strip under it.
    assert!(list.x >= page.x + page_size.w as i32);
    // The list fills the column; status and Reset follow it, Reset last.
    assert!(status.y >= list.y + list_size.h as i32);
    assert!(reset.y > status.y);
    assert_eq!(reset.x, list.x);

    // Every editor sits at the right edge of its row, after the name.
    let right_edge = list.x + list_size.w as i32;

    for editor in &state.panel.editors {
        let (origin, extent) = at(&mut engine, editor.id);

        assert!(origin.x >= list.x + 100, "{} too far left", editor.name);
        assert!(origin.x + extent.w as i32 <= right_edge, "{}", editor.name);
    }
}
