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

//! A `Widget` invalidated while it is hidden (or below a hidden parent) is painted when it is
//! first shown, not left blank until something invalidates it again.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::geometry::{point, size};
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetCore};

/// A widget that paints nothing; the tests only count its paints.
#[derive(Debug)]
struct Blank {
    core: WidgetCore,
}

impl Widget for Blank {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn pushrod::paint::Painter) {}
}

fn blank() -> Box<Blank> {
    Box::new(Blank {
        core: WidgetCore::new(point(0, 0), size(20, 20)),
    })
}

#[test]
fn a_child_of_a_hidden_parent_is_painted_when_the_parent_is_shown() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(100, 100))));

    let parent = cache.add_child(0, blank());
    let child = cache.add_child(parent, blank());
    let mut backend = HeadlessBackend::new(size(100, 100));

    cache.get_mut(parent).unwrap().set_hidden(true);
    cache.draw_loop(&mut backend);
    cache.draw_loop(&mut backend);

    assert!(backend.painted.iter().all(|(id, _)| *id != child));

    // Showing the parent invalidates only the parent; the child must still be painted.
    cache.get_mut(parent).unwrap().set_hidden(false);
    cache.draw_loop(&mut backend);

    assert!(backend.painted.iter().any(|(id, _)| *id == child));
    assert!(backend.texture_sizes.contains_key(&child));
}
