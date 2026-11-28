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

//! Autoresizing: a form with a stretching text area, an OK button that keeps its distance from
//! the bottom right corner, and a title that stays put.  The example resizes the window
//! headlessly and prints where each widget ends up.  Run it with
//! `cargo run -p pushrod-widgets --example anchors`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::input::InputEvent;
use pushrod::layout::Anchors;
use pushrod::widget::Widget;
use pushrod_widgets::panel::Panel;

fn main() {
    let mut engine: Engine = Engine::new(size(300, 200), 30);
    let mut backend = HeadlessBackend::new(size(300, 200));

    let mut title = Panel::new().at(10, 10).size(280, 20);
    let mut notes = Panel::new().at(10, 40).size(280, 110);
    let mut ok = Panel::new().at(220, 160).size(70, 30);

    // The title keeps its place; the notes grow with the window; OK sticks to the corner.
    title.set_anchors(Anchors::LEFT | Anchors::TOP);
    notes.set_anchors(Anchors::LEFT | Anchors::RIGHT | Anchors::TOP | Anchors::BOTTOM);
    ok.set_anchors(Anchors::RIGHT | Anchors::BOTTOM);

    let ids = [
        ("title", engine.add_widget(title)),
        ("notes", engine.add_widget(notes)),
        ("ok", engine.add_widget(ok)),
    ];

    // The first frame records the window's starting size.
    engine.widgets().draw_loop(&mut backend);

    for window in [size(400, 300), size(250, 150)] {
        engine.dispatch(InputEvent::Resized(window), &mut ());
        engine.widgets().draw_loop(&mut backend);
        println!("Window {}x{}:", window.w, window.h);

        for (name, id) in ids {
            let widget = engine.widgets().get(id).unwrap();

            println!(
                "  {name}: at {:?}, size {:?}",
                widget.get_origin(),
                widget.get_size()
            );
        }
    }
}
