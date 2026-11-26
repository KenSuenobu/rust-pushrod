// Clipping Example
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

//! A panel with a child that sticks out of it, and a pop-up that is allowed to: prints each
//! widget's visible rectangle, the cropped blits, and what a click lands on, without opening a window.
//!
//! Run it with `cargo run --example clipping`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::geometry::{point, size};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetId};

/// Adds a child at a position with a size and returns its ID.
fn add(cache: &mut WidgetCache, parent: WidgetId, x: i32, y: i32, w: u32, h: u32) -> WidgetId {
    let mut widget = RootWidget::new(size(w, h));

    widget.set_origin(point(x, y));
    cache.add_child(parent, Box::new(widget))
}

fn main() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 100))));

    let panel = add(&mut cache, 0, 10, 10, 80, 60);
    let wide = add(&mut cache, panel, 50, 20, 80, 20);
    let popup = add(&mut cache, panel, 50, 45, 80, 20);

    cache
        .get_mut(popup)
        .unwrap()
        .set_behavior(Behavior::OVERLAY);

    println!("panel {panel}, wide child {wide}, overlay pop-up {popup}");

    for id in [panel, wide, popup] {
        println!("widget {id} visible: {:?}", cache.visible_rect(id));
    }

    // Over the part of the wide child that sticks out of the panel, then over the pop-up.
    println!("click (100, 40) -> {}", cache.widget_at(100, 40));
    println!("click (100, 65) -> {}", cache.widget_at(100, 65));

    let mut backend = HeadlessBackend::new(size(200, 100));

    cache.draw_loop(&mut backend);

    for ((id, source), (_, dest)) in backend.draw_sources.iter().zip(&backend.draws) {
        println!("blit widget {id}: copy {source:?} to {dest:?}");
    }
}
