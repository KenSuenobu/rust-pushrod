// Drag Controller Example
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

//! A custom widget that uses a `DragController`.  It runs without a window: the events a mouse
//! would produce are fed to the cache by hand.
//!
//! Run it with `cargo run --example drag_controller`.

use pushrod::cache::WidgetCache;
use pushrod::controllers::{ControllerEvent, DragController};
use pushrod::event::{Event, EventCtx, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetCore};
use sdl2::mouse::MouseButton;

/// A handle that follows a drag and keeps its offset.
struct Handle {
    core: WidgetCore,
    value: i32,
}

impl Widget for Handle {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Moves `value` with the drag, and reports it with `ValueChanged`.
    fn on_controller_event(&mut self, event: &ControllerEvent, ctx: &mut EventCtx) -> Propagation {
        match event {
            ControllerEvent::DragStarted { .. } => println!("drag started"),
            ControllerEvent::DragUpdated { offset, .. } => {
                self.value = offset.x;
                println!("drag updated: value = {}", self.value);
                ctx.emit(Event::ValueChanged { id: self.get_id() });
            }
            ControllerEvent::DragEnded { offset, .. } => {
                println!("drag ended at {}, {}", offset.x, offset.y)
            }
            _ => return Propagation::Continue,
        }

        Propagation::Stop
    }
}

fn main() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 100))));

    let mut core = WidgetCore::new(point(0, 0), size(20, 20));

    // Adding the controller also adds the event classes it needs to the widget's mask.
    core.add_controller(DragController::new());

    let handle = cache.add_child(0, Box::new(Handle { core, value: 0 }));
    let start = point(10, 10);

    cache.propagate(
        handle,
        &Event::PointerPressed {
            id: handle,
            button: MouseButton::Left,
            position: start,
            count: 1,
        },
    );

    for x in [12, 25, 40] {
        cache.propagate(
            handle,
            &Event::PointerMoved {
                id: handle,
                position: point(x, 10),
            },
        );
    }

    cache.propagate(
        handle,
        &Event::PointerReleased {
            id: handle,
            button: MouseButton::Left,
            position: point(40, 10),
            count: 1,
        },
    );
}
