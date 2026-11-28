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

//! The showcase shell: the page registry, search, selection, and the remembered page.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::geometry::{Point, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::WidgetId;
use pushrod_showcase::page::Group;
use pushrod_showcase::pages;
use pushrod_showcase::registry::Registry;
use pushrod_showcase::settings;
use pushrod_showcase::shell::{self, ShowcaseState};
use pushrod_showcase::source_view::Code;
use pushrod_widgets::custom_paint::CustomPaint;
use pushrod_widgets::text::Text;

/// Builds the shell headlessly and runs the first idle pass.
fn start(settings_path: Option<std::path::PathBuf>) -> (Engine<ShowcaseState>, ShowcaseState) {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(900, 640), 30);
    let mut state = shell::build(&mut engine, settings_path);

    engine.run_idle(&mut state);
    settle(&mut engine);
    (engine, state)
}

/// Draws frames until the layout settles, so widgets have their places.
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

/// Clicks a widget with the left mouse button.
fn click(engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, id: WidgetId) {
    let position = center(engine, id);

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

fn title(engine: &mut Engine<ShowcaseState>, state: &ShowcaseState) -> String {
    engine
        .widgets()
        .get_as::<Text>(state.shell.title)
        .unwrap()
        .get_text()
}

#[test]
fn registry_filters_by_name_and_description() {
    let mut registry = Registry::new();

    pages::register_all(&mut registry);

    let everything: usize = Group::ALL
        .iter()
        .map(|group| registry.matching(*group, "").len())
        .sum();

    assert_eq!(everything, registry.len());

    let by_name = registry.matching(Group::Standard, "  CHECK ");

    assert_eq!(by_name, vec![registry.find("Check Box").unwrap()]);

    let by_description = registry.matching(Group::Standard, "titled frame");

    assert_eq!(by_description, vec![registry.find("Group Box").unwrap()]);
    assert!(registry.matching(Group::Extra, "").is_empty());
    assert!(registry.matching(Group::Standard, "zzzz").is_empty());
}

#[test]
fn page_names_are_unique_and_pages_carry_source() {
    let mut registry = Registry::new();

    pages::register_all(&mut registry);

    for index in 0..registry.len() {
        let page = registry.get(index).unwrap();

        assert!(!page.description().is_empty());
        assert!(page.source().starts_with("fn build("), "{}", page.name());
    }
}

#[test]
fn first_page_opens_and_clicking_another_replaces_it() {
    let (mut engine, mut state) = start(None);

    assert_eq!(title(&mut engine, &state), "Text");

    let first_root = state.page_root.unwrap();
    let (button, index) = state
        .buttons
        .iter()
        .copied()
        .find(|(_, index)| state.registry.get(*index).unwrap().name() == "Button")
        .unwrap();

    click(&mut engine, &mut state, button);

    assert_eq!(state.selected, Some(index));
    assert_eq!(title(&mut engine, &state), "Button");
    assert!(engine.widgets().get(first_root).is_none());
    assert!(engine.widgets().get(state.page_root.unwrap()).is_some());
}

#[test]
fn typing_in_search_filters_the_list() {
    let (mut engine, mut state) = start(None);
    let all = state.buttons.len();

    assert_eq!(all, state.registry.len());

    // Focus the field by clicking it, then type: typing emits `TextChanged`.
    let search = state.shell.search;

    click(&mut engine, &mut state, search);
    engine.dispatch(InputEvent::Text("radio".into()), &mut state);
    settle(&mut engine);

    assert_eq!(state.buttons.len(), 1);

    let (_, index) = state.buttons[0];

    assert_eq!(state.registry.get(index).unwrap().name(), "Radio Button");

    // A search with no match shows no page buttons.
    engine.dispatch(InputEvent::Text("zzzz".into()), &mut state);

    assert!(state.buttons.is_empty());
}

#[test]
fn last_page_is_remembered() {
    let dir = std::env::temp_dir().join(format!("pushrod-showcase-test-{}", std::process::id()));
    let path = dir.join("last_page");

    assert_eq!(settings::load_last_page(&path), None);

    let (mut engine, mut state) = start(Some(path.clone()));
    let (button, _) = state
        .buttons
        .iter()
        .copied()
        .find(|(_, index)| state.registry.get(*index).unwrap().name() == "Frame")
        .unwrap();

    click(&mut engine, &mut state, button);

    assert_eq!(settings::load_last_page(&path).as_deref(), Some("Frame"));

    let (mut engine2, state2) = start(Some(path));

    assert_eq!(title(&mut engine2, &state2), "Frame");

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn every_page_opens_with_its_title() {
    let (mut engine, mut state) = start(None);
    let buttons = state.buttons.clone();

    assert_eq!(buttons.len(), state.registry.len());

    for (button, index) in buttons {
        click(&mut engine, &mut state, button);

        assert_eq!(state.selected, Some(index));
        assert_eq!(
            title(&mut engine, &state),
            state.registry.get(index).unwrap().name()
        );
    }
}

#[test]
fn every_page_source_is_its_own_build_function() {
    let mut registry = Registry::new();

    pages::register_all(&mut registry);

    for index in 0..registry.len() {
        let page = registry.get(index).unwrap();
        let source = page.source();

        assert!(source.starts_with("fn build("), "{}", page.name());
        assert!(source.ends_with('}'), "{}", page.name());
        assert!(!source.contains("impl Page for"), "{}", page.name());
    }
}

#[test]
fn the_source_toggle_swaps_the_page_for_its_code_and_back() {
    let (mut engine, mut state) = start(None);
    let page = state.page_root.unwrap();
    let code = state
        .registry
        .get(state.selected.unwrap())
        .unwrap()
        .source();

    assert!(engine.widgets().get(state.shell.copy).unwrap().is_hidden());

    {
        let id = state.shell.source;
        click(&mut engine, &mut state, id);
    }

    assert!(state.showing_source);
    assert!(engine.widgets().get(page).is_none());
    assert!(!engine.widgets().get(state.shell.copy).unwrap().is_hidden());

    let view = engine
        .widgets()
        .get_as::<CustomPaint<Code>>(state.page_root.unwrap())
        .unwrap();

    assert_eq!(view.data().text, code);

    // Choosing another page while the code is shown shows that page's code.
    let (button, index) = state
        .buttons
        .iter()
        .copied()
        .find(|(_, index)| state.registry.get(*index).unwrap().name() == "Button")
        .unwrap();

    click(&mut engine, &mut state, button);

    let button_code = state.registry.get(index).unwrap().source();
    let view = engine
        .widgets()
        .get_as::<CustomPaint<Code>>(state.page_root.unwrap())
        .unwrap();

    assert_eq!(view.data().text, button_code);

    {
        let id = state.shell.source;
        click(&mut engine, &mut state, id);
    }

    assert!(!state.showing_source);
    assert!(engine.widgets().get(state.shell.copy).unwrap().is_hidden());
    assert!(
        engine
            .widgets()
            .get_as::<CustomPaint<Code>>(state.page_root.unwrap())
            .is_none()
    );
}

#[test]
fn copy_places_the_shown_code_on_the_clipboard() {
    let (mut engine, mut state) = start(None);
    let code = state
        .registry
        .get(state.selected.unwrap())
        .unwrap()
        .source();

    {
        let id = state.shell.source;
        click(&mut engine, &mut state, id);
    }
    assert!(engine.widgets().take_clipboard_write().is_none());

    {
        let id = state.shell.copy;
        click(&mut engine, &mut state, id);
    }

    assert_eq!(
        engine.widgets().take_clipboard_write().as_deref(),
        Some(code)
    );
}
