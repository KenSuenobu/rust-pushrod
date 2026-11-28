// Undo
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

//! # Undo
//!
//! An [`UndoStack`] holds the [`Command`]s the user has done, so they can be undone and redone.
//! The `Engine` owns one stack for the application.  It works like this:
//!
//! * A [`Command`] knows how to `apply` itself and how to `revert` it.  Push one with
//!   [`Ui::push_command`](crate::engine::Ui::push_command) (it is applied at once) or
//!   [`Ui::record_command`](crate::engine::Ui::record_command) (the change already happened).
//! * Pushing a command clears the redo history.
//! * A command may `merge` the one pushed after it, so a run of typing is one undo step.
//! * Text fields push their own edits ([`TextEdit`]); turn that off with
//!   [`Engine::set_text_undo`](crate::engine::Engine::set_text_undo).  Edits to a password field
//!   are never recorded.
//! * [`Engine::add_undo_actions`](crate::engine::Engine::add_undo_actions) adds the standard Undo
//!   and Redo [`Action`](crate::action::Action)s, which enable and disable themselves from
//!   [`UndoStack::can_undo`] and [`UndoStack::can_redo`].
//!
//! Every change to the stack queues [`EngineEvent::UndoStackChanged`](crate::event::EngineEvent),
//! and each undo or redo queues `CommandUndone` or `CommandRedone`.
//!
//! # Examples
//!
//! ```rust
//! use pushrod::engine::{Engine, Ui};
//! use pushrod::geometry::size;
//! use pushrod::undo::Command;
//!
//! /// Adds an amount to the counter.
//! struct Add(i32);
//!
//! impl Command<i32> for Add {
//!     fn description(&self) -> String {
//!         format!("Add {}", self.0)
//!     }
//!
//!     fn apply(&mut self, state: &mut i32, _ui: &mut Ui<i32>) {
//!         *state += self.0;
//!     }
//!
//!     fn revert(&mut self, state: &mut i32, _ui: &mut Ui<i32>) {
//!         *state -= self.0;
//!     }
//! }
//!
//! let mut engine: Engine<i32> = Engine::new(size(100, 100), 30);
//! let mut total = 0;
//!
//! engine.push_command(Add(5), &mut total);
//! assert_eq!(total, 5);
//!
//! engine.undo(&mut total);
//! assert_eq!(total, 0);
//!
//! engine.redo(&mut total);
//! assert_eq!(total, 5);
//! ```

use crate::action::ActionId;
use crate::engine::Ui;
use crate::widget::WidgetId;
use std::any::Any;
use std::collections::HashMap;

/// How many commands a stack keeps unless told otherwise.
pub const DEFAULT_UNDO_LIMIT: usize = 100;

/// One undoable change.
///
/// `S` is the application state type of the `Engine`.
pub trait Command<S> {
    /// Describes the change for a menu or a log ("Typing", "Delete row").
    ///
    /// # Returns
    ///
    /// The description.
    fn description(&self) -> String;

    /// Makes the change.  Called when the command is pushed with
    /// [`Ui::push_command`](crate::engine::Ui::push_command), and again by every redo.
    ///
    /// # Arguments
    ///
    /// * `state` - The application state.
    /// * `ui` - The handle to the window's `Widget`s.
    fn apply(&mut self, state: &mut S, ui: &mut Ui<S>);

    /// Takes the change back.
    ///
    /// # Arguments
    ///
    /// * `state` - The application state.
    /// * `ui` - The handle to the window's `Widget`s.
    fn revert(&mut self, state: &mut S, ui: &mut Ui<S>);

    /// Offers this command the one pushed after it.  If it can fold that change into itself, it
    /// does and returns `true`, and the next command is dropped; a run of typing is one step.
    ///
    /// # Arguments
    ///
    /// * `next` - The command just pushed; see [`Command::as_any`] to read its type.
    ///
    /// # Returns
    ///
    /// `true` if `next` was merged.  The default merges nothing.
    fn merge(&mut self, next: &dyn Command<S>) -> bool {
        let _ = next;
        false
    }

    /// Lets a command read the type of another in [`Command::merge`].
    ///
    /// # Returns
    ///
    /// `Some(self)` for a command that others may merge into; the default is `None`.
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

/// The undo and redo history.  See the [module documentation](self).
pub struct UndoStack<S> {
    undo: Vec<Box<dyn Command<S>>>,
    redo: Vec<Box<dyn Command<S>>>,
    limit: usize,
    /// Set by `seal`: the next push starts a new step instead of merging into the last.
    sealed: bool,
    /// Set while a command is being applied, reverted, or redone: pushes are ignored.
    replaying: bool,
    /// Set by any change; the engine clears it after it updates the Undo and Redo actions.
    dirty: bool,
    /// The text each text field had after its last recorded edit.
    baselines: HashMap<WidgetId, String>,
    /// Whether the text fields record their edits.
    text_undo: bool,
    /// The Undo and Redo actions this stack keeps enabled.
    actions: Option<(ActionId, ActionId)>,
}

impl<S> std::fmt::Debug for UndoStack<S> {
    /// Formats the value for debugging.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndoStack")
            .field("undo", &self.undo.len())
            .field("redo", &self.redo.len())
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}

impl<S> Default for UndoStack<S> {
    /// Creates an empty stack that keeps [`DEFAULT_UNDO_LIMIT`] commands.
    ///
    /// # Returns
    ///
    /// The stack.
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: DEFAULT_UNDO_LIMIT,
            sealed: false,
            replaying: false,
            dirty: false,
            baselines: HashMap::new(),
            text_undo: true,
            actions: None,
        }
    }
}

impl<S> UndoStack<S> {
    /// Returns whether there is a command to undo.
    ///
    /// # Returns
    ///
    /// `true` if [`Ui::undo`](crate::engine::Ui::undo) would do something.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Returns whether there is a command to redo.
    ///
    /// # Returns
    ///
    /// `true` if [`Ui::redo`](crate::engine::Ui::redo) would do something.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Describes the command an undo would take back.
    ///
    /// # Returns
    ///
    /// The description, or `None` if there is nothing to undo.
    pub fn undo_description(&self) -> Option<String> {
        self.undo.last().map(|c| c.description())
    }

    /// Describes the command a redo would make again.
    ///
    /// # Returns
    ///
    /// The description, or `None` if there is nothing to redo.
    pub fn redo_description(&self) -> Option<String> {
        self.redo.last().map(|c| c.description())
    }

    /// Returns how many commands can be undone.
    ///
    /// # Returns
    ///
    /// The number of undo steps.
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Returns how many commands can be redone.
    ///
    /// # Returns
    ///
    /// The number of redo steps.
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// Returns the most commands the stack keeps.
    ///
    /// # Returns
    ///
    /// The limit.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Sets the most commands the stack keeps; the oldest are forgotten first.
    ///
    /// # Arguments
    ///
    /// * `limit` - The most undo steps to keep; at least 1.
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit.max(1);
        self.trim();
    }

    /// Forgets all history, such as after a document is closed.
    pub fn clear(&mut self) {
        if self.undo.is_empty() && self.redo.is_empty() {
            return;
        }

        self.undo.clear();
        self.redo.clear();
        self.sealed = false;
        self.dirty = true;
    }

    /// Ends the current step: the next command pushed is never merged into the last one.  Text
    /// fields call this when they lose focus.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Records a command that has already happened.  If the last command merges it, it is folded
    /// in; otherwise it becomes a new step.  Pushing clears the redo history.  Ignored while a
    /// command is being applied or reverted.
    ///
    /// # Arguments
    ///
    /// * `command` - The command, already applied.
    pub(crate) fn record(&mut self, command: Box<dyn Command<S>>) {
        if self.replaying {
            return;
        }

        self.redo.clear();
        self.dirty = true;

        let merged = !self.sealed
            && self
                .undo
                .last_mut()
                .is_some_and(|last| last.merge(command.as_ref()));

        self.sealed = false;

        if !merged {
            self.undo.push(command);
            self.trim();
        }
    }

    /// Forgets the oldest steps beyond the limit.
    fn trim(&mut self) {
        if self.undo.len() > self.limit {
            let extra = self.undo.len() - self.limit;

            self.undo.drain(..extra);
            self.dirty = true;
        }
    }

    /// Takes the command to undo, and stops pushes until [`UndoStack::finish_undo`].
    ///
    /// # Returns
    ///
    /// The command, or `None` if there is nothing to undo.
    pub(crate) fn begin_undo(&mut self) -> Option<Box<dyn Command<S>>> {
        let command = self.undo.pop()?;

        self.replaying = true;
        Some(command)
    }

    /// Files a command that was just reverted as a redo step.
    ///
    /// # Arguments
    ///
    /// * `command` - The command from [`UndoStack::begin_undo`].
    pub(crate) fn finish_undo(&mut self, command: Box<dyn Command<S>>) {
        self.redo.push(command);
        self.replaying = false;
        self.sealed = true;
        self.dirty = true;
    }

    /// Takes the command to redo, and stops pushes until [`UndoStack::finish_redo`].
    ///
    /// # Returns
    ///
    /// The command, or `None` if there is nothing to redo.
    pub(crate) fn begin_redo(&mut self) -> Option<Box<dyn Command<S>>> {
        let command = self.redo.pop()?;

        self.replaying = true;
        Some(command)
    }

    /// Files a command that was just applied again as an undo step.
    ///
    /// # Arguments
    ///
    /// * `command` - The command from [`UndoStack::begin_redo`].
    pub(crate) fn finish_redo(&mut self, command: Box<dyn Command<S>>) {
        self.undo.push(command);
        self.replaying = false;
        self.sealed = true;
        self.dirty = true;
    }

    /// Starts or stops ignoring pushes, around applying a command that was just pushed.
    ///
    /// # Arguments
    ///
    /// * `replaying` - `true` to ignore pushes.
    pub(crate) fn set_replaying(&mut self, replaying: bool) {
        self.replaying = replaying;
    }

    /// Sets the Undo and Redo actions to keep enabled.
    ///
    /// # Arguments
    ///
    /// * `actions` - The Undo and Redo actions.
    pub(crate) fn bind_actions(&mut self, actions: (ActionId, ActionId)) {
        self.actions = Some(actions);
        self.dirty = true;
    }

    /// Returns the Undo and Redo actions the stack keeps enabled.
    ///
    /// # Returns
    ///
    /// The Undo and Redo actions, or `None` if none are bound.
    pub(crate) fn bound_actions(&self) -> Option<(ActionId, ActionId)> {
        self.actions
    }

    /// Returns whether a command is being applied, reverted, or redone.
    ///
    /// # Returns
    ///
    /// `true` while pushes are ignored.
    pub(crate) fn is_replaying(&self) -> bool {
        self.replaying
    }

    /// Returns and clears the changed flag.
    ///
    /// # Returns
    ///
    /// `true` if the stack changed since the last call.
    pub(crate) fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Returns whether text fields record their edits.
    ///
    /// # Returns
    ///
    /// `true` if they do; the default.
    pub fn text_undo(&self) -> bool {
        self.text_undo
    }

    /// Turns the recording of text field edits on or off.
    ///
    /// # Arguments
    ///
    /// * `on` - `true` to record them.
    pub fn set_text_undo(&mut self, on: bool) {
        self.text_undo = on;

        if !on {
            self.baselines.clear();
        }
    }

    /// Returns the text a field had after its last recorded edit.
    ///
    /// # Arguments
    ///
    /// * `id` - The text field.
    ///
    /// # Returns
    ///
    /// The text, or `None` if the field has not been seen.
    pub(crate) fn baseline(&self, id: WidgetId) -> Option<&str> {
        self.baselines.get(&id).map(String::as_str)
    }

    /// Remembers the text a field has now.
    ///
    /// # Arguments
    ///
    /// * `id` - The text field.
    /// * `text` - Its text.
    pub(crate) fn set_baseline(&mut self, id: WidgetId, text: String) {
        self.baselines.insert(id, text);
    }

    /// Forgets a field's text, such as when it is removed.
    ///
    /// # Arguments
    ///
    /// * `id` - The text field.
    pub(crate) fn forget_baseline(&mut self, id: WidgetId) {
        self.baselines.remove(&id);
    }
}

/// A change to the text of one text field.  Text fields push these themselves.  Consecutive
/// edits of the same field that all add text, or all remove text, merge into one step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// The text field.
    pub widget: WidgetId,
    /// The text before the edit.
    pub before: String,
    /// The text after the edit.
    pub after: String,
}

impl TextEdit {
    /// Creates an edit.
    ///
    /// # Arguments
    ///
    /// * `widget` - The text field.
    /// * `before` - The text before the edit.
    /// * `after` - The text after the edit.
    ///
    /// # Returns
    ///
    /// The edit.
    pub fn new(widget: WidgetId, before: &str, after: &str) -> Self {
        Self {
            widget,
            before: before.to_string(),
            after: after.to_string(),
        }
    }

    /// Returns whether the edit adds text (typing, paste) rather than removes it.
    ///
    /// # Returns
    ///
    /// `true` if the text got longer.
    fn grows(&self) -> bool {
        self.after.chars().count() >= self.before.chars().count()
    }

    /// Sets the field's text and remembers it as the field's baseline.
    ///
    /// # Arguments
    ///
    /// * `ui` - The handle to the window's `Widget`s.
    /// * `text` - The text to set.
    fn set<S>(&self, ui: &mut Ui<S>, text: &str) {
        use crate::registry::PropertyValue;

        let _ = ui.set_widget_property(self.widget, "text", &PropertyValue::Text(text.to_string()));

        ui.undo_baseline_set(self.widget, text);
    }
}

impl<S> Command<S> for TextEdit {
    /// Describes the edit.
    ///
    /// # Returns
    ///
    /// "Typing" for added text, "Delete" for removed text.
    fn description(&self) -> String {
        if self.grows() { "Typing" } else { "Delete" }.to_string()
    }

    /// Sets the text to what it was after the edit.
    ///
    /// # Arguments
    ///
    /// * `_state` - Unused.
    /// * `ui` - The handle to the window's `Widget`s.
    fn apply(&mut self, _state: &mut S, ui: &mut Ui<S>) {
        let after = self.after.clone();

        self.set(ui, &after);
    }

    /// Sets the text to what it was before the edit.
    ///
    /// # Arguments
    ///
    /// * `_state` - Unused.
    /// * `ui` - The handle to the window's `Widget`s.
    fn revert(&mut self, _state: &mut S, ui: &mut Ui<S>) {
        let before = self.before.clone();

        self.set(ui, &before);
    }

    /// Folds in the next edit of the same field when both add text or both remove text and the
    /// next one starts where this one ended.
    ///
    /// # Arguments
    ///
    /// * `next` - The command just pushed.
    ///
    /// # Returns
    ///
    /// `true` if merged.
    fn merge(&mut self, next: &dyn Command<S>) -> bool {
        let Some(next) = next.as_any().and_then(|a| a.downcast_ref::<TextEdit>()) else {
            return false;
        };

        if next.widget != self.widget || next.before != self.after || next.grows() != self.grows() {
            return false;
        }

        self.after = next.after.clone();
        true
    }

    /// Lets another [`TextEdit`] read this one.
    ///
    /// # Returns
    ///
    /// This command.
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}
