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

//! A settings dialog with three tabs, driven headlessly: it clicks a tab, switches pages with
//! Ctrl+Tab, adds and removes a page, and prints what the tab view does.  Run it with
//! `cargo run -p pushrod-widgets --example tab_view`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::tab_view::TabView;

#[derive(Default)]
struct Log(Vec<String>);

fn main() {
    let mut engine: Engine<Log> = Engine::new(size(400, 300), 30);
    let mut backend = HeadlessBackend::new(size(400, 300));
    let mut log = Log::default();
    let tabs = engine.add_widget(
        TabView::new()
            .at(10, 10)
            .size(300, 200)
            .with_page("General")
            .with_page("Advanced")
            .with_page("About"),
    );

    engine.on_selection_change(tabs, move |log, ui| {
        let page = ui.get::<TabView>(tabs).and_then(TabView::selected_page);

        log.0.push(format!("shown page: {page:?}"));
    });

    let mut frame = |engine: &mut Engine<Log>, log: &mut Log| {
        engine.widgets().draw_loop(&mut backend);
        engine.run_idle(log);
    };

    frame(&mut engine, &mut log);

    // Controls go in a page like in any widget; hiding the page keeps them.
    let general = engine
        .widgets()
        .get_as::<TabView>(tabs)
        .map(|t| t.pages()[0])
        .unwrap_or_default();
    let updates =
        engine.add_child_widget(general, CheckBox::new("Updates").at(10, 10).size(120, 20));

    frame(&mut engine, &mut log);

    // Click the second tab.
    let second = engine
        .widgets()
        .get_as::<TabView>(tabs)
        .map(|t| t.tabs()[1])
        .unwrap_or_default();
    let (origin, tab_size) = (
        engine.widgets().absolute_origin(second),
        engine
            .widgets()
            .get(second)
            .map(|t| t.get_size())
            .unwrap_or_default(),
    );
    let at = point(origin.x + tab_size.w as i32 / 2, origin.y + 10);

    for event in [
        InputEvent::MouseDown {
            position: at,
            button: MouseButton::Left,
            clicks: 1,
        },
        InputEvent::MouseUp {
            position: at,
            button: MouseButton::Left,
            clicks: 1,
        },
    ] {
        engine.dispatch(event, &mut log);
        frame(&mut engine, &mut log);
    }

    // Ctrl+Tab goes to the next page, Ctrl+Shift+Tab back.
    for modifiers in [Modifiers::CTRL, Modifiers::CTRL | Modifiers::SHIFT] {
        engine.dispatch(
            InputEvent::KeyDown {
                key: Key::Tab,
                modifiers,
                repeat: false,
            },
            &mut log,
        );
        frame(&mut engine, &mut log);
    }

    println!(
        "the checkbox on the hidden first page still exists: {}",
        engine.widgets().get(updates).is_some()
    );

    // Add a page, then remove the first.
    if let Ok((tab, page)) = TabView::add_page(engine.widgets(), tabs, "Network") {
        println!("added tab {tab} and page {page}");
    }

    let _ = TabView::remove_page(engine.widgets(), tabs, 0);

    frame(&mut engine, &mut log);

    println!(
        "pages now: {:?}",
        engine
            .widgets()
            .get_as::<TabView>(tabs)
            .map(TabView::page_count)
    );
    println!("events:");

    for line in &log.0 {
        println!("  {line}");
    }
}
