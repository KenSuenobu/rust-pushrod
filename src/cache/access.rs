// Accessibility actions
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

//! Performs the requests of assistive technology on the [`WidgetCache`], by the same paths user
//! input takes, so the same callbacks run.

use super::WidgetCache;
use crate::accessibility::AccessAction;
use crate::event::{EngineEvent, Event, EventCtx, EventPhase};
use crate::registry::PropertyValue;
use crate::widget::WidgetId;

impl WidgetCache {
    /// Performs an accessibility action on a `Widget`.
    ///
    /// [`AccessAction::Click`] sends [`Event::Activated`] (what Return or a mnemonic sends), so a
    /// button clicks and a check box toggles.  [`AccessAction::Focus`] is [`WidgetCache::set_focus`].
    /// [`AccessAction::SetValue`] and [`AccessAction::SetText`] set the `value` and `text`
    /// properties, then emit [`Event::ValueChanged`] or [`Event::TextChanged`] as an edit would.
    /// A disabled, hidden, or modal-blocked `Widget` ignores every action.
    /// [`EngineEvent::AccessActionPerformed`] is queued when the action was performed.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` the action is for.
    /// * `action` - What it is asked to do.
    ///
    /// # Returns
    ///
    /// The events the `Widget` generated, or `None` if the action was refused: there is no such
    /// `Widget`, it cannot act now, or it has no such property.
    pub fn perform_access_action(
        &mut self,
        id: WidgetId,
        action: &AccessAction,
    ) -> Option<Vec<Event>> {
        let usable = self
            .get(id)
            .is_some_and(|w| w.is_enabled() && !w.is_hidden())
            && !self.hidden_above(id)
            && !self.is_inert(id);

        if !usable {
            return None;
        }

        let mut ctx = EventCtx::default();

        match action {
            AccessAction::Click => {
                self.send(id, &Event::Activated { id }, EventPhase::Target, &mut ctx);
            }
            AccessAction::Focus => {
                if !self.set_focus(id) {
                    return None;
                }
            }
            AccessAction::SetValue(value) => {
                if !self.set_from_access(id, "value", PropertyValue::Float(*value)) {
                    return None;
                }

                ctx.emit(Event::ValueChanged { id });
            }
            AccessAction::SetText(text) => {
                if !self.set_from_access(id, "text", PropertyValue::Text(text.clone())) {
                    return None;
                }

                ctx.emit(Event::TextChanged { id });
            }
        }

        self.queue_event(Event::Engine(EngineEvent::AccessActionPerformed {
            id,
            action: action.kind(),
        }));

        Some(self.finish(ctx))
    }

    /// Sets a property for an accessibility action.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` has the property and accepted the value.
    fn set_from_access(&mut self, id: WidgetId, name: &str, value: PropertyValue) -> bool {
        self.set_property(id, name, &value).is_ok()
    }
}
