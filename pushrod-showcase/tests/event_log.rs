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

//! The event log: events from the page appear with their widget ID and payload; engine events,
//! filter, clear, pause, and clicking a line.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::geometry::{Point, point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::widget::WidgetId;
use pushrod_showcase::event_log::{FILTER_ENGINE, FILTERS};
use pushrod_showcase::shell::{self, ShowcaseState, WINDOW_H, WINDOW_W};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::text_field::TextField;
use std::time::Instant;

type Harness = (Engine<ShowcaseState>, ShowcaseState);

/// Builds the shell headlessly, opens the named page, and settles the layout.
/// A window tall enough to show every page button without scrolling the list.
const TALL_H: u32 = WINDOW_H * 2;

fn start(page: &str) -> Harness {
    let mut engine: Engine<ShowcaseState> = Engine::new(size(WINDOW_W, TALL_H), 30);
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
    let mut backend = HeadlessBackend::new(size(WINDOW_W, TALL_H));

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

/// Returns the first check box on the page.
fn first_check_box(engine: &mut Engine<ShowcaseState>, state: &ShowcaseState) -> WidgetId {
    let root = state.page_root.unwrap();

    engine
        .widgets()
        .children_of(root)
        .to_vec()
        .into_iter()
        .find(|id| engine.widgets().get_as::<CheckBox>(*id).is_some())
        .unwrap()
}

/// Returns the text of every entry now in the log.
fn texts(state: &ShowcaseState) -> Vec<String> {
    state.log.entries.iter().map(|e| e.text()).collect()
}

#[test]
fn the_log_starts_empty_with_engine_events_off() {
    let (_engine, state) = start("Button");

    assert!(state.log.entries.is_empty());
    assert!(!state.log.engine_on && !state.log.paused);
    assert_eq!(state.log.filter_name(), "All");
}

#[test]
fn a_click_appears_with_its_widget_id_and_payload() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);

    let entry = state
        .log
        .entries
        .iter()
        .find(|e| e.kind == "Clicked")
        .unwrap();

    assert_eq!(entry.source, Some(normal));
    assert!(entry.payload.contains("button: Left"), "{}", entry.payload);
    assert!(entry.text().contains(&format!("widget {normal}")));
    assert_eq!(state.log.lines.len(), state.log.entries.len());
}

#[test]
fn a_toggle_shows_its_checked_state_and_newest_is_last() {
    let (mut engine, mut state) = start("Check Box");
    let check = first_check_box(&mut engine, &state);

    click(&mut engine, &mut state, check);
    click(&mut engine, &mut state, check);

    let toggles: Vec<_> = state
        .log
        .entries
        .iter()
        .filter(|e| e.kind == "Toggled")
        .collect();

    assert_eq!(toggles.len(), 2);
    assert_ne!(toggles[0].payload, toggles[1].payload);
    assert!(toggles[0].seq < toggles[1].seq);
}

#[test]
fn the_shells_own_controls_are_not_logged() {
    let (mut engine, mut state) = start("Button");
    let clear = state.log.clear;

    click(&mut engine, &mut state, clear);
    let search = state.shell.search;

    click(&mut engine, &mut state, search);

    assert!(state.log.entries.is_empty(), "{:?}", texts(&state));
}

#[test]
fn engine_events_can_be_switched_on() {
    let (mut engine, mut state) = start("Text Field");
    let root = state.page_root.unwrap();
    let field = engine
        .widgets()
        .children_of(root)
        .to_vec()
        .into_iter()
        .find(|id| engine.widgets().get_as::<TextField>(*id).is_some())
        .unwrap();
    let engine_box = state.log.engine;

    // Off: focusing the field is logged as a widget event only.
    click(&mut engine, &mut state, field);
    assert!(state.log.entries.iter().all(|e| !e.engine));

    click(&mut engine, &mut state, engine_box);
    assert!(state.log.engine_on);

    // Focus moves to the search field and back, which the engine reports.
    let search = state.shell.search;

    click(&mut engine, &mut state, search);
    click(&mut engine, &mut state, field);

    let engine_entries: Vec<_> = state.log.entries.iter().filter(|e| e.engine).collect();

    assert!(!engine_entries.is_empty(), "{:?}", texts(&state));
    // The log's own lines do not feed it: only the offered kinds appear.
    assert!(
        engine_entries
            .iter()
            .all(|e| e.kind == "FocusChanged" || e.kind == "PropertyChanged")
    );
}

#[test]
fn the_kind_button_filters_and_cycles_back_to_all() {
    let (mut engine, mut state) = start("Check Box");
    let check = first_check_box(&mut engine, &state);

    click(&mut engine, &mut state, check);

    let total = state.log.entries.len();

    assert!(total >= 1);

    let kind = state.log.kind;

    // Step to "Toggled": only toggles remain shown, while every event is kept.
    while state.log.filter_name() != "Toggled" {
        click(&mut engine, &mut state, kind);
    }

    assert_eq!(state.log.entries.len(), total);
    assert!(!state.log.lines.is_empty());
    assert!(state.log.lines.len() <= total);

    let shown: Vec<u64> = state.log.lines.iter().map(|(_, seq)| *seq).collect();

    assert!(
        state
            .log
            .entries
            .iter()
            .filter(|e| shown.contains(&e.seq))
            .all(|e| e.kind == "Toggled")
    );

    for _ in 0..FILTERS.len() {
        click(&mut engine, &mut state, kind);
    }

    assert_eq!(state.log.filter_name(), "Toggled");
    assert!(FILTERS.contains(&FILTER_ENGINE));
}

#[test]
fn clear_empties_the_log() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    click(&mut engine, &mut state, normal);
    assert!(!state.log.entries.is_empty());

    let clear = state.log.clear;

    click(&mut engine, &mut state, clear);

    assert!(state.log.entries.is_empty());
    assert!(state.log.lines.is_empty());
    assert!(engine.widgets().children_of(state.log.list).is_empty());
}

#[test]
fn pause_stops_recording_until_resumed() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");
    let pause = state.log.pause;

    click(&mut engine, &mut state, pause);
    assert!(state.log.paused);

    click(&mut engine, &mut state, normal);
    assert!(state.log.entries.is_empty());

    click(&mut engine, &mut state, pause);
    assert!(!state.log.paused);

    click(&mut engine, &mut state, normal);
    assert!(!state.log.entries.is_empty());
}

#[test]
fn clicking_a_line_outlines_the_widget_that_emitted_it() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");
    let other = page_button(&mut engine, &state, "Disabled");

    click(&mut engine, &mut state, normal);
    click(&mut engine, &mut state, other);

    assert_eq!(state.panel.selected, Some(other));

    let (line, seq) = state.log.lines[0];
    let emitter = state
        .log
        .entries
        .iter()
        .find(|e| e.seq == seq)
        .and_then(|e| e.source)
        .unwrap();

    assert_eq!(emitter, normal);

    click(&mut engine, &mut state, line);

    assert_eq!(state.panel.selected, Some(normal));
    assert_eq!(state.panel.outline.len(), 4);
}

#[test]
fn the_log_keeps_only_the_newest_events() {
    let (mut engine, mut state) = start("Button");
    let normal = page_button(&mut engine, &state, "Normal");

    for _ in 0..(pushrod_showcase::event_log::MAX_ENTRIES + 20) {
        click(&mut engine, &mut state, normal);
    }

    assert_eq!(
        state.log.entries.len(),
        pushrod_showcase::event_log::MAX_ENTRIES
    );
    assert_eq!(state.log.lines.len(), state.log.entries.len());
    assert!(state.log.entries.front().unwrap().seq > 1);
}

#[test]
fn the_log_sits_under_the_page() {
    let (mut engine, state) = start("Button");
    let at = |engine: &mut Engine<ShowcaseState>, id: WidgetId| {
        (
            engine.widgets().absolute_origin(id),
            engine.widgets().get(id).unwrap().get_size(),
        )
    };
    let (page, page_size) = at(&mut engine, state.shell.page_area);
    let (kind, _) = at(&mut engine, state.log.kind);
    let (list, _) = at(&mut engine, state.log.list);

    assert!(kind.y >= page.y + page_size.h as i32, "header under page");
    assert!(list.y > kind.y);
    assert!(list.x >= page.x);
}

/// Returns the visible widgets that have not been painted at their size.
fn unpainted(engine: &mut Engine<ShowcaseState>, backend: &HeadlessBackend) -> Vec<WidgetId> {
    let mut missing = Vec::new();
    let mut stack = vec![pushrod::widget::ROOT_WIDGET_ID];

    while let Some(id) = stack.pop() {
        let Some(widget) = engine.widgets().get(id) else {
            continue;
        };

        if widget.is_hidden() {
            continue;
        }

        let extent = widget.get_size();

        if extent.w > 0 && extent.h > 0 && backend.texture_sizes.get(&id) != Some(&extent) {
            missing.push(id);
        }

        stack.extend(engine.widgets().children_of(id).to_vec());
    }

    missing
}

#[test]
fn scroll_bars_are_painted_when_they_first_appear() {
    // One backend for the whole session, as in the real window: a widget painted into an
    // earlier backend would hide the bug.
    let mut engine: Engine<ShowcaseState> = Engine::new(size(WINDOW_W, TALL_H), 30);
    let mut state = shell::build(&mut engine, None);
    let mut backend = HeadlessBackend::new(size(WINDOW_W, TALL_H));

    engine.run_idle(&mut state);

    for _ in 0..3 {
        engine.widgets().draw_loop(&mut backend);
    }

    let mut step = |engine: &mut Engine<ShowcaseState>, state: &mut ShowcaseState, id: WidgetId| {
        let position = center(engine, id);

        for event in [
            InputEvent::MouseDown {
                position,
                button: MouseButton::Left,
                clicks: 1,
            },
            InputEvent::MouseUp {
                position,
                button: MouseButton::Left,
                clicks: 1,
            },
        ] {
            engine.dispatch(event, state);
        }

        engine.advance(Instant::now(), state);
        engine.run_idle(state);

        for _ in 0..3 {
            engine.widgets().draw_loop(&mut backend);
        }

        unpainted(engine, &backend)
    };
    let index = state.registry.find("Button").unwrap();
    let (list_button, _) = state
        .buttons
        .iter()
        .copied()
        .find(|(_, i)| *i == index)
        .unwrap();

    assert_eq!(
        step(&mut engine, &mut state, list_button),
        Vec::<WidgetId>::new()
    );

    let normal = page_button(&mut engine, &state, "Normal");

    // Selecting a widget fills the property list, so its scroll bar appears.
    for _ in 0..3 {
        assert_eq!(
            step(&mut engine, &mut state, normal),
            Vec::<WidgetId>::new()
        );
    }
}
