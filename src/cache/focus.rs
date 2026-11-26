// Focus
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

//! Keyboard focus for the [`WidgetCache`]: which `Widget` receives key presses, how Tab and the
//! arrow keys move it, and how Return and Escape activate the default and cancel `Widget`s.
//!
//! **Focus groups.** `Widget`s that are [`Behavior::EXCLUSIVE`] and share a parent form one group
//! (a radio group).  Tab and Shift-Tab visit each group once, at its checked member (or its
//! first member if none is checked); the arrow keys move within the group.  Every other
//! focusable `Widget` is a group of one.  Order is by `WidgetId`, which is the order the form was
//! designed in.

use super::WidgetCache;
use crate::event::{EngineEvent, Event, EventCtx, EventPhase, Propagation};
use crate::input::{Key, Modifiers};
use crate::widget::{Behavior, ROOT_WIDGET_ID, VisualState, WidgetId};

/// The modifier keys that stop a Tab or arrow key from moving focus (Shift is allowed).
const COMMAND_MODS: Modifiers = Modifiers::CTRL.union(Modifiers::ALT).union(Modifiers::GUI);

impl WidgetCache {
    /// Returns the `Widget` that has the keyboard focus.
    ///
    /// # Returns
    ///
    /// The ID of the focused `Widget`, or `None` if nothing has focus.
    pub fn focused(&self) -> Option<WidgetId> {
        self.focused_widget_id
    }

    /// Gives the keyboard focus to a `Widget`.
    ///
    /// The `Widget` that had focus receives [`Event::FocusLost`], then the new one receives
    /// [`Event::FocusGained`] (both are also queued for the application), the two `Widget`s'
    /// [`VisualState::FOCUSED`] flags are updated, and [`EngineEvent::FocusChanged`] is queued.
    /// Nothing happens if the `Widget` already has focus.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` to focus.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` has focus afterward; `false` if it does not exist or cannot take
    /// focus (not [`Behavior::FOCUSABLE`], disabled, or hidden, itself or through a parent).
    pub fn set_focus(&mut self, id: WidgetId) -> bool {
        if !self.can_focus(id) {
            return false;
        }

        self.move_focus(Some(id));

        true
    }

    /// Removes the keyboard focus, so no `Widget` has it.
    pub fn clear_focus(&mut self) {
        self.move_focus(None);
    }

    /// Returns whether a `Widget` can take focus now.
    fn can_focus(&self, id: WidgetId) -> bool {
        self.get(id).is_some_and(|w| w.is_focusable()) && !self.hidden_above(id)
    }

    /// Returns whether any ancestor of a `Widget` is hidden.
    fn hidden_above(&self, id: WidgetId) -> bool {
        self.ancestors(id)
            .iter()
            .any(|&a| self.get(a).is_some_and(|w| w.is_hidden()))
    }

    /// Returns whether the `Widget` is a focusable member of a radio group.
    fn is_group_member(&self, id: WidgetId) -> bool {
        self.can_focus(id)
            && self
                .get(id)
                .is_some_and(|w| w.behavior().contains(Behavior::EXCLUSIVE))
    }

    /// Moves focus, delivering the focus events and updating the visual states.
    fn move_focus(&mut self, new: Option<WidgetId>) {
        let old = self.focused_widget_id;

        if old == new {
            return;
        }

        self.focused_widget_id = new;

        if let Some(id) = old {
            self.set_focused_state(id, false);
            self.notify(id, Event::FocusLost { id });
        }

        if let Some(id) = new {
            self.set_focused_state(id, true);
            self.notify(id, Event::FocusGained { id });
        }

        self.text_input_wanted = new
            .and_then(|id| self.get(id))
            .is_some_and(|w| w.accepts_text_input());

        self.queue_event(Event::Engine(EngineEvent::FocusChanged { old, new }));
    }

    /// Sets or clears [`VisualState::FOCUSED`] on a `Widget`.
    fn set_focused_state(&mut self, id: WidgetId, focused: bool) {
        if let Some(widget) = self.get_mut(id) {
            let mut state = widget.state();

            state.set(VisualState::FOCUSED, focused);
            widget.set_state(state);
        }
    }

    /// Delivers a focus event to its `Widget`, then queues it, and what the `Widget` generated,
    /// for the application.
    fn notify(&mut self, id: WidgetId, event: Event) {
        let mut ctx = EventCtx::default();

        self.send(id, &event, EventPhase::Target, &mut ctx);
        self.queue_event(event);

        for generated in self.finish(ctx) {
            self.queue_event(generated);
        }
    }

    /// Returns whether the engine should ask the OS for text input.
    ///
    /// # Returns
    ///
    /// `true` while the focused `Widget` accepts typed text.
    pub fn text_input_wanted(&self) -> bool {
        self.text_input_wanted
    }

    /// Focuses the nearest focusable `Widget` at or above the one that was clicked.
    ///
    /// A click on something that cannot take focus leaves the focus where it is.
    ///
    /// # Arguments
    ///
    /// * `target` - The `Widget` that was clicked.
    pub(super) fn focus_for_click(&mut self, target: WidgetId) {
        let mut candidates = vec![target];

        candidates.extend(self.ancestors(target));

        if let Some(&id) = candidates.iter().find(|&&id| self.can_focus(id)) {
            self.move_focus(Some(id));
        }
    }

    /// Returns the `Widget` that receives keys.
    ///
    /// # Returns
    ///
    /// The focused `Widget`, or the root with no focus.
    pub(super) fn key_target(&self) -> WidgetId {
        self.focused_widget_id
            .filter(|&id| self.can_focus(id))
            .unwrap_or(ROOT_WIDGET_ID)
    }

    /// The members of the radio group a `Widget` belongs to, in ID order.
    fn group_of(&self, id: WidgetId) -> Vec<WidgetId> {
        let Some(widget) = self.get(id) else {
            return Vec::new();
        };

        if !self.is_group_member(id) {
            return vec![id];
        }

        self.children_of(widget.get_parent())
            .iter()
            .copied()
            .filter(|&c| self.is_group_member(c))
            .collect()
    }

    /// The one `Widget` Tab visits for a group: its checked member, or its first.
    fn group_stop(&self, id: WidgetId) -> WidgetId {
        let group = self.group_of(id);

        group
            .iter()
            .copied()
            .find(|&m| {
                self.get(m)
                    .is_some_and(|w| w.state().contains(VisualState::CHECKED))
            })
            .or_else(|| group.first().copied())
            .unwrap_or(id)
    }

    /// Every `Widget` Tab visits, in ID order.
    fn tab_stops(&self) -> Vec<WidgetId> {
        (0..self.cache.len() as WidgetId)
            .filter(|&id| self.can_focus(id) && self.group_stop(id) == id)
            .collect()
    }

    /// Moves focus to the next (or previous) group, wrapping at the ends.
    ///
    /// # Arguments
    ///
    /// * `backward` - `true` for Shift-Tab.
    ///
    /// # Returns
    ///
    /// `true` if a `Widget` has focus afterward.
    pub fn focus_next(&mut self, backward: bool) -> bool {
        let stops = self.tab_stops();

        if stops.is_empty() {
            return false;
        }

        let current = self
            .focused_widget_id
            .filter(|&id| self.can_focus(id))
            .map(|id| self.group_stop(id))
            .and_then(|stop| stops.iter().position(|&s| s == stop));
        let last = stops.len() - 1;
        let index = match (current, backward) {
            (None, false) => 0,
            (None, true) => last,
            (Some(i), false) => (i + 1) % stops.len(),
            (Some(0), true) => last,
            (Some(i), true) => i - 1,
        };

        self.move_focus(Some(stops[index]));

        true
    }

    /// Moves focus within the focused `Widget`'s radio group, wrapping at the ends.
    ///
    /// # Arguments
    ///
    /// * `forward` - `true` for the next member (Right, Down), `false` for the previous.
    ///
    /// # Returns
    ///
    /// `true` if focus moved; `false` if the focused `Widget` is not in a group of two or more.
    pub fn focus_in_group(&mut self, forward: bool) -> bool {
        let Some(current) = self.focused_widget_id else {
            return false;
        };
        let group = self.group_of(current);
        let Some(index) = group.iter().position(|&m| m == current) else {
            return false;
        };

        if group.len() < 2 {
            return false;
        }

        let next = if forward {
            (index + 1) % group.len()
        } else {
            (index + group.len() - 1) % group.len()
        };

        self.move_focus(Some(group[next]));

        true
    }

    /// Moves focus within the radio group and activates the member that received it, so an
    /// arrow key selects as well as moves (a radio group's checked member follows the focus).
    ///
    /// # Arguments
    ///
    /// * `forward` - `true` for the next member, `false` for the previous.
    /// * `ctx` - The context collecting generated events.
    fn select_in_group(&mut self, forward: bool, ctx: &mut EventCtx) {
        if self.focus_in_group(forward)
            && let Some(id) = self.focused_widget_id
        {
            self.send(id, &Event::Activated { id }, EventPhase::Target, ctx);
        }
    }

    /// Returns the checked member of the radio group under `parent`.
    ///
    /// A group is the [`Behavior::EXCLUSIVE`] children of one parent.
    ///
    /// # Arguments
    ///
    /// * `parent` - The `Widget` the group's members are children of.
    ///
    /// # Returns
    ///
    /// The ID of the checked member, or `None` if none is checked.
    pub fn checked_in_group(&self, parent: WidgetId) -> Option<WidgetId> {
        self.children_of(parent).iter().copied().find(|&c| {
            self.get(c).is_some_and(|w| {
                w.behavior().contains(Behavior::EXCLUSIVE)
                    && w.state().contains(VisualState::CHECKED)
            })
        })
    }

    /// Keeps radio groups to one checked member.  For each `Toggled { checked: true }` from an
    /// [`Behavior::EXCLUSIVE`] `Widget`, every other checked exclusive sibling is cleared and a
    /// `Toggled { checked: false }` for it follows the event that caused it.
    ///
    /// # Arguments
    ///
    /// * `events` - The events `Widget`s generated.
    ///
    /// # Returns
    ///
    /// The same events, with the clearing `Toggled` events added.
    pub(super) fn apply_exclusive(&mut self, events: Vec<Event>) -> Vec<Event> {
        let mut out = Vec::with_capacity(events.len());

        for event in events {
            let checked_id = match &event {
                Event::Toggled { id, checked: true } => Some(*id),
                _ => None,
            };

            out.push(event);

            if let Some(id) = checked_id {
                out.extend(self.clear_siblings(id));
            }
        }

        out
    }

    /// Clears the checked exclusive siblings of `id`, returning a `Toggled(false)` for each.
    fn clear_siblings(&mut self, id: WidgetId) -> Vec<Event> {
        let Some(widget) = self.get(id) else {
            return Vec::new();
        };

        if !widget.behavior().contains(Behavior::EXCLUSIVE) {
            return Vec::new();
        }

        let parent = widget.get_parent();
        let siblings: Vec<WidgetId> = self
            .children_of(parent)
            .iter()
            .copied()
            .filter(|&c| c != id)
            .collect();
        let mut cleared = Vec::new();

        for sibling in siblings {
            let Some(w) = self.get_mut(sibling) else {
                continue;
            };

            if w.behavior().contains(Behavior::EXCLUSIVE)
                && w.state().contains(VisualState::CHECKED)
            {
                let mut state = w.state();

                state.set(VisualState::CHECKED, false);
                w.set_state(state);
                w.set_invalidated(true);
                cleared.push(Event::Toggled {
                    id: sibling,
                    checked: false,
                });
            }
        }

        cleared
    }

    /// Finds the `Widget` with `flag` that Return or Escape activates.
    ///
    /// The form is the nearest ancestor of the focused `Widget` (or the root) that contains an
    /// enabled, visible `Widget` with the flag; the first one in ID order wins.
    fn find_action(&self, flag: Behavior) -> Option<WidgetId> {
        let start = self.key_target();
        let mut forms = vec![start];

        forms.extend(self.ancestors(start));

        forms
            .into_iter()
            .find_map(|form| self.action_in(form, flag))
    }

    /// Searches the subtree under `form` for an actionable `Widget` with `flag`.
    fn action_in(&self, form: WidgetId, flag: Behavior) -> Option<WidgetId> {
        let mut stack = vec![form];
        let mut found: Option<WidgetId> = None;

        while let Some(id) = stack.pop() {
            let Some(widget) = self.get(id) else {
                continue;
            };

            if widget.is_hidden() {
                continue;
            }

            if widget.is_enabled()
                && widget.behavior().contains(flag)
                && found.is_none_or(|f| id < f)
            {
                found = Some(id);
            }

            stack.extend(self.children_of(id));
        }

        found
    }

    /// Queues the activation of the default or cancel `Widget`, if the form has one, and delivers
    /// [`Event::Activated`] to that `Widget` so it can act.
    fn trigger_action(&mut self, cancel: bool, ctx: &mut EventCtx) -> bool {
        let flag = if cancel {
            Behavior::CANCEL_ACTION
        } else {
            Behavior::DEFAULT_ACTION
        };

        let Some(id) = self.find_action(flag) else {
            return false;
        };

        // The application hears of the activation first; then the widget itself is told, so a
        // button can respond as if it had been clicked.
        ctx.emit(Event::Activated { id });
        self.send(id, &Event::Activated { id }, EventPhase::Target, ctx);
        self.queue_event(Event::Engine(EngineEvent::ActionTriggered { id, cancel }));

        true
    }

    /// Delivers a key press to the focused `Widget`, then, if nothing stopped it, applies the
    /// keyboard rules: Tab and Shift-Tab, the arrow keys within a group, Return for the default
    /// `Widget`, and Escape (or Cmd-. on macOS) for the cancel `Widget`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key pressed.
    /// * `modifiers` - The modifier keys held.
    /// * `repeat` - `true` if this is an auto-repeat.
    /// * `ctx` - The context collecting generated events.
    pub(super) fn handle_key_down(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        repeat: bool,
        ctx: &mut EventCtx,
    ) {
        let target = self.key_target();
        let event = Event::KeyPressed {
            id: target,
            key,
            modifiers,
            repeat,
        };

        if self.propagate_with(target, &event, ctx) == Propagation::Stop {
            return;
        }

        let command = modifiers.intersects(COMMAND_MODS);
        let shift = modifiers.intersects(Modifiers::SHIFT);

        match key {
            Key::Tab if !command => {
                self.focus_next(shift);
            }
            Key::Right | Key::Down if !command => {
                self.select_in_group(true, ctx);
            }
            Key::Left | Key::Up if !command => {
                self.select_in_group(false, ctx);
            }
            Key::Return | Key::KpEnter if !repeat && !command => {
                self.trigger_action(false, ctx);
            }
            Key::Escape if !repeat && !command => {
                self.trigger_action(true, ctx);
            }
            Key::Char('.')
                if cfg!(target_os = "macos") && !repeat && modifiers.intersects(Modifiers::GUI) =>
            {
                self.trigger_action(true, ctx);
            }
            _ => {}
        }
    }
}
