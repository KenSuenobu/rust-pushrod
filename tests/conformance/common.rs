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

//! Shared test widgets for the conformance suite.

use pushrod::cache::WidgetCache;
use pushrod::event::{Event, EventCtx, EventMask, EventPhase, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetCore, WidgetId};
use std::cell::RefCell;
use std::rc::Rc;

/// What a [`Probe`] saw: its name, and the phase it was delivered in.
pub type PhaseLog = Rc<RefCell<Vec<(&'static str, EventPhase)>>>;

/// A widget that logs each delivery and can capture or stop an event.
pub struct Probe {
    core: WidgetCore,
    name: &'static str,
    capture: bool,
    stops_in: Option<EventPhase>,
    log: PhaseLog,
}

impl Widget for Probe {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing.
    ///
    /// # Arguments
    ///
    /// * `_p` - The painter.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Whether this probe takes part in the capture phase.
    fn handles_capture(&self) -> bool {
        self.capture
    }

    /// Logs the delivery, and stops in the configured phase.
    ///
    /// # Arguments
    ///
    /// * `_event` - The event.
    /// * `ctx` - The context, which knows the phase.
    ///
    /// # Returns
    ///
    /// `Stop` in the configured phase, otherwise `Continue`.
    fn handle_event(&mut self, _event: &Event, ctx: &mut EventCtx) -> Propagation {
        self.log.borrow_mut().push((self.name, ctx.phase()));

        if self.stops_in == Some(ctx.phase()) {
            Propagation::Stop
        } else {
            Propagation::Continue
        }
    }
}

/// Creates a cache holding only the root widget.
pub fn cache() -> WidgetCache {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));
    cache
}

/// Adds a [`Probe`] that listens to every event class under `parent`.
///
/// # Arguments
///
/// * `cache` - The cache to add to.
/// * `parent` - The parent widget.
/// * `name` - The name the probe logs.
/// * `capture` - Whether it takes part in the capture phase.
/// * `stops_in` - The phase in which it returns `Stop`, if any.
/// * `log` - The log it writes to.
///
/// # Returns
///
/// The new widget's ID.
pub fn probe(
    cache: &mut WidgetCache,
    parent: WidgetId,
    name: &'static str,
    capture: bool,
    stops_in: Option<EventPhase>,
    log: &PhaseLog,
) -> WidgetId {
    probe_with_mask(cache, parent, name, capture, stops_in, log, EventMask::ALL)
}

/// Adds a [`Probe`] with the given event mask under `parent`.
///
/// # Arguments
///
/// * `cache` - The cache to add to.
/// * `parent` - The parent widget.
/// * `name` - The name the probe logs.
/// * `capture` - Whether it takes part in the capture phase.
/// * `stops_in` - The phase in which it returns `Stop`, if any.
/// * `log` - The log it writes to.
/// * `mask` - The probe's event mask.
///
/// # Returns
///
/// The new widget's ID.
pub fn probe_with_mask(
    cache: &mut WidgetCache,
    parent: WidgetId,
    name: &'static str,
    capture: bool,
    stops_in: Option<EventPhase>,
    log: &PhaseLog,
    mask: EventMask,
) -> WidgetId {
    let mut core = WidgetCore::new(point(0, 0), size(50, 50));

    core.set_event_mask(mask);
    cache.add_child(
        parent,
        Box::new(Probe {
            core,
            name,
            capture,
            stops_in,
            log: log.clone(),
        }),
    )
}
