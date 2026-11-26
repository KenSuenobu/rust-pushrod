// Timers Example
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

//! A clock label updated every second by a repeating timer, a one-shot timer that stops the
//! clock after four seconds, and an idle callback, all driven without opening a window: the
//! example moves time forward by hand with `Engine::advance`.
//!
//! Run it with `cargo run --example timers`.

use pushrod::engine::Engine;
use pushrod::event::{Event, EventCtx, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::widget::{Widget, WidgetCore};
use std::time::{Duration, Instant};

/// A label that shows the elapsed seconds.  (The standard `TextWidget` is in `pushrod-widgets`.)
struct ClockLabel {
    core: WidgetCore,
    text: String,
}

impl Widget for ClockLabel {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing; this example only prints the text.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Ignores events.
    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventCtx) -> Propagation {
        Propagation::Continue
    }
}

/// The application state.
#[derive(Default)]
struct App {
    seconds: u32,
    idle_runs: u32,
}

fn main() {
    let mut engine: Engine<App> = Engine::new(size(200, 60), 60);
    let label = engine.add_widget(ClockLabel {
        core: WidgetCore::new(point(10, 10), size(180, 20)),
        text: String::from("0 s"),
    });

    // Every second, update the label.
    let clock = engine.on_timer(Duration::from_secs(1), true, move |app, ui| {
        app.seconds += 1;

        if let Some(widget) = ui.get_mut::<ClockLabel>(label) {
            widget.text = format!("{} s", app.seconds);
            println!("clock label: {}", widget.text);
        }
    });

    // After four seconds, stop the clock from a callback.
    engine.on_timer(Duration::from_secs(4), false, move |_app, ui| {
        ui.cancel_timer(clock);
        println!("clock stopped");
    });

    engine.on_idle(|app, _ui| app.idle_runs += 1);

    let mut app = App::default();
    let start = Instant::now();

    // Six simulated seconds, one tenth of a second at a time.
    for step in 1..=60 {
        engine.advance(start + Duration::from_millis(step * 100), &mut app);
        engine.run_idle(&mut app);
    }

    println!(
        "final: {} s, idle callbacks ran {} times",
        app.seconds, app.idle_runs
    );
}
