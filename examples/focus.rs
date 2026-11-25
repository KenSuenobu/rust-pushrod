// Focus Example
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

//! A form with two fields and OK and Cancel buttons, driven by keystrokes without opening a
//! window: Tab moves between the fields and buttons, Return presses OK, Escape presses Cancel.
//!
//! Run it with `cargo run --example focus`.

use pushrod::cache::WidgetCache;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::{self, Painter};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::{Keycode, Mod, Scancode};

/// A form control that draws a focus outline (through `state_overlay`) and does nothing else.
struct Control {
    core: WidgetCore,
}

impl Widget for Control {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws the control, with the dotted outline while it has focus.
    fn paint(&self, p: &mut dyn Painter) {
        paint::state_overlay(p, self.get_size(), self.state());
    }

    /// Ignores events; the cache handles Tab, Return, and Escape.
    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventCtx) -> Propagation {
        Propagation::Continue
    }
}

/// Adds a focusable control and returns its ID.
fn control(cache: &mut WidgetCache, x: i32, behavior: Behavior) -> WidgetId {
    let mut core = WidgetCore::new(point(x, 0), size(40, 20));

    core.set_event_mask(EventMask::ALL);

    let id = cache.add_child(0, Box::new(Control { core }));

    cache
        .get_mut(id)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE | behavior);

    id
}

/// Presses a key and prints what the application would hear.
fn press(cache: &mut WidgetCache, name: &str, key: Keycode, modifiers: Mod) {
    let events = cache.handle_event(SdlEvent::KeyDown {
        timestamp: 0,
        window_id: 0,
        keycode: Some(key),
        scancode: Some(Scancode::A),
        keymod: modifiers,
        repeat: false,
    });

    println!(
        "{name:<10} focus: {:?}, events: {events:?}",
        cache.focused()
    );
}

fn main() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(300, 100))));

    let first = control(&mut cache, 0, Behavior::empty());
    let second = control(&mut cache, 50, Behavior::empty());
    let ok = control(&mut cache, 100, Behavior::DEFAULT_ACTION);
    let cancel = control(&mut cache, 150, Behavior::CANCEL_ACTION);

    println!("fields {first} and {second}, OK {ok}, Cancel {cancel}");

    press(&mut cache, "Tab", Keycode::Tab, Mod::NOMOD);
    press(&mut cache, "Tab", Keycode::Tab, Mod::NOMOD);
    press(&mut cache, "Shift-Tab", Keycode::Tab, Mod::LSHIFTMOD);
    press(&mut cache, "Return", Keycode::Return, Mod::NOMOD);
    press(&mut cache, "Escape", Keycode::Escape, Mod::NOMOD);
}
