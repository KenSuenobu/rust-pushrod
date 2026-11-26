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

//! A transparent container that passes its children's key presses on to the application.
//!
//! A `Button` handles the mouse but ignores keys, and does not emit the `KeyPressed` events it
//! receives, so `Engine::on_key_press` never runs for one.  Key presses bubble up to the
//! button's parent, though: this container re-emits each one it sees on the way up, unchanged
//! (its `id` is still the focused button's), so `on_key_press(button, ..)` runs and the
//! application can activate the button with Space.

use pushrod::event::{Event, EventCtx, EventMask, EventPhase, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Widget, WidgetCore};

/// A container that re-emits the key presses that bubble up from its children.
#[derive(Debug)]
pub struct KeyRelay {
    core: WidgetCore,
}

impl KeyRelay {
    /// Creates an empty relay, transparent until its color is set.  Give it a layout to arrange
    /// its children.
    ///
    /// # Returns
    ///
    /// The relay, with keyboard events added to its event mask.
    pub fn new() -> Self {
        let mut relay = Self {
            core: WidgetCore::new(point(0, 0), size(0, 0)),
        };
        let mask = relay.event_mask() | EventMask::KEYBOARD;

        relay.core.set_event_mask(mask);
        relay
    }
}

impl Widget for KeyRelay {
    /// Returns the relay's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the relay's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the relay with its color, transparent unless set, so the window shows through.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the relay.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.style().background.unwrap_or(self.get_color()));
    }

    /// Re-emits a key press that bubbled up from a child, so the application's `on_key_press`
    /// callback for that child runs.  Nothing is consumed.
    ///
    /// # Arguments
    ///
    /// * `event` - The event.
    /// * `ctx` - The context used to emit it again.
    ///
    /// # Returns
    ///
    /// Always [`Propagation::Continue`], so Return, Escape and Tab keep working.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if matches!(event, Event::KeyPressed { .. }) && ctx.phase() == EventPhase::Bubble {
            ctx.emit(event.clone());
        }

        Propagation::Continue
    }
}
