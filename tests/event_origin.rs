// Event origin tests
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

//! `EventCtx::origin` is the window position of the widget handling a pointer event.

use pushrod::cache::WidgetCache;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Point, point, size};
use pushrod::input::InputEvent;
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetCore};
use std::cell::Cell;
use std::rc::Rc;

/// A widget that records the origin it was told about on each event.
struct Recorder {
    core: WidgetCore,
    seen: Rc<Cell<Option<Point>>>,
}

impl Widget for Recorder {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn handle_event(&mut self, _event: &Event, ctx: &mut EventCtx) -> Propagation {
        self.seen.set(Some(ctx.origin()));
        Propagation::Continue
    }
}

#[test]
fn a_nested_widget_is_told_its_window_origin() {
    let mut cache = WidgetCache::new();
    let seen = Rc::new(Cell::new(None));

    cache.add(Box::new(RootWidget::new(size(200, 200))));

    let parent_core = WidgetCore::new(point(30, 40), size(100, 100));
    let parent = cache.add_child(
        0,
        Box::new(Recorder {
            core: parent_core,
            seen: Rc::new(Cell::new(None)),
        }),
    );
    let mut core = WidgetCore::new(point(5, 6), size(20, 20));

    core.set_event_mask(EventMask::ALL);
    cache.add_child(
        parent,
        Box::new(Recorder {
            core,
            seen: seen.clone(),
        }),
    );

    cache.handle_event(InputEvent::MouseMove {
        position: point(40, 50),
    });
    assert_eq!(seen.get(), Some(point(35, 46)));
}

#[test]
fn a_default_context_has_a_zero_origin() {
    assert_eq!(EventCtx::default().origin(), point(0, 0));
}
