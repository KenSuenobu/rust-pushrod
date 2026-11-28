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

//! Reorder a list by dragging: each row is both a drag source and a drop target, and dropping row
//! `a` on row `b` moves it to `b`'s place.  The example plays the mouse headlessly.  Run it with
//! `cargo run -p pushrod-widgets --example drag_and_drop`.

use pushrod::dnd::{DragPayload, DragSource, DropTarget};
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod_widgets::panel::Panel;

/// The height of one row, in pixels.
const ROW: i32 = 30;

/// The slot a row occupies in the list.
struct Slot(usize);

/// What the application remembers: the items, top to bottom.
struct List {
    items: Vec<&'static str>,
}

/// Plays one mouse gesture: press, move, release.
///
/// # Arguments
///
/// * `engine` - The engine receiving the input.
/// * `list` - The application state.
/// * `from` - Where the press happens.
/// * `to` - Where the pointer is released.
fn drag(engine: &mut Engine<List>, list: &mut List, from: (i32, i32), to: (i32, i32)) {
    let start = point(from.0, from.1);
    let end = point(to.0, to.1);

    engine.dispatch(InputEvent::MouseMove { position: start }, list);
    engine.dispatch(
        InputEvent::MouseDown {
            position: start,
            button: MouseButton::Left,
            clicks: 1,
        },
        list,
    );
    engine.dispatch(InputEvent::MouseMove { position: end }, list);
    engine.dispatch(
        InputEvent::MouseUp {
            position: end,
            button: MouseButton::Left,
            clicks: 1,
        },
        list,
    );
}

fn main() {
    let mut engine: Engine<List> = Engine::new(size(200, 200), 30);
    let mut list = List {
        items: vec!["apples", "bread", "cheese"],
    };

    for slot in 0..list.items.len() {
        let row = engine.add_widget(Panel::new().at(0, slot as i32 * ROW).size(200, ROW as u32));

        engine.set_drag_source(row, DragSource::new(move || DragPayload::new(Slot(slot))));
        engine.set_drop_target(row, DropTarget::new().accepts::<Slot>());
        engine.on_drop(row, move |list, _ui, payload| {
            if let Some(Slot(from)) = payload.get::<Slot>() {
                let item = list.items.remove(*from);

                list.items.insert(slot, item);
            }
        });
    }

    println!("Before: {:?}", list.items);
    drag(&mut engine, &mut list, (50, 5), (50, 2 * ROW + 5));
    println!("After dragging the first row to the last: {:?}", list.items);
    drag(&mut engine, &mut list, (50, 5), (500, 500));
    println!("Dropped on nothing: {:?}", list.items);
}
