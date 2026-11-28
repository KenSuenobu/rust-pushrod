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

//! A list view of 10,000 fictional rows, driven headlessly: it clicks, extends a selection with
//! Shift, scrolls, types ahead, and changes the model, printing what the list does.  Run it
//! with `cargo run -p pushrod-widgets --example list_view`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod_widgets::list_view::{ListView, SelectionMode, VecModel};

/// What the list told the application.
#[derive(Default)]
struct Log(Vec<String>);

fn main() {
    let mut engine: Engine<Log> = Engine::new(size(300, 200), 30);
    let mut backend = HeadlessBackend::new(size(300, 200));
    let mut log = Log::default();
    let model = VecModel::new((1..=10_000).map(|n| format!("Item {n:05}")));
    let list = engine.add_widget(
        ListView::new(model)
            .at(10, 10)
            .size(200, 100)
            .selection_mode(SelectionMode::Multiple),
    );

    engine.on_selection_change(list, |log, _| log.0.push("selection changed".into()));
    engine.on_activate(list, |log, _| log.0.push("activated".into()));
    engine.on_viewport_change(list, |log, _, o| {
        log.0.push(format!("scrolled to row {}", o.y))
    });

    let mut frame = |engine: &mut Engine<Log>, log: &mut Log| {
        engine.widgets().draw_loop(&mut backend);
        engine.run_idle(log);
    };

    frame(&mut engine, &mut log);

    let rows = engine
        .widgets()
        .get_as::<ListView>(list)
        .map_or(0, |l| l.typed_parts().rows.len());

    println!("10,000 rows, {rows} row widgets");

    // Click the third row, then Shift-click the fifth.
    for (modifiers, y) in [(Modifiers::empty(), 55), (Modifiers::SHIFT, 95)] {
        engine.dispatch(InputEvent::ModifiersChanged(modifiers), &mut log);
        for event in [
            InputEvent::MouseDown {
                position: point(30, y),
                button: MouseButton::Left,
                clicks: 1,
            },
            InputEvent::MouseUp {
                position: point(30, y),
                button: MouseButton::Left,
                clicks: 1,
            },
        ] {
            engine.dispatch(event, &mut log);
        }
        frame(&mut engine, &mut log);
    }

    let selected = engine
        .widgets()
        .get_as::<ListView>(list)
        .map(ListView::selected_rows);

    println!("selected after click and Shift-click: {selected:?}");

    // The wheel and the End key scroll without creating widgets.
    engine.dispatch(
        InputEvent::Wheel {
            delta: point(0, -20),
        },
        &mut log,
    );
    frame(&mut engine, &mut log);
    engine.dispatch(
        InputEvent::KeyDown {
            key: Key::End,
            modifiers: Modifiers::empty(),
            repeat: false,
        },
        &mut log,
    );
    frame(&mut engine, &mut log);

    // Change the model: the list follows.
    if let Some(l) = engine.widgets().get_as_mut::<ListView>(list)
        && let Some(model) = l.model_mut::<VecModel>()
    {
        model.insert(0, "Item 00000");
    }

    frame(&mut engine, &mut log);

    let list_ref = engine.widgets().get_as::<ListView>(list);

    println!("rows now: {:?}", list_ref.map(ListView::row_count));
    println!("events:");

    for line in &log.0 {
        println!("  {line}");
    }
}
