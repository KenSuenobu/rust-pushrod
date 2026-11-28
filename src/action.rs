// Action
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

//! # Action
//!
//! An [`Action`] is one command (Save, Bold, Undo) that the application offers in several places:
//! a button, a menu item, a toolbar button, a keyboard shortcut.  The application describes the
//! command once, registers it with [`Engine::add_action`](crate::engine::Engine::add_action), and
//! binds controls to it with [`Engine::bind_widget`](crate::engine::Engine::bind_widget).  The
//! action then owns the state:
//!
//! * Changing `enabled`, `checked`, the label, or the tooltip updates every bound control.
//! * Triggering any bound control, or pressing the action's [`Shortcut`], toggles `checked` (for a
//!   checkable action), emits [`EngineEvent::ActionInvoked`],
//!   and runs the callback registered with [`Engine::on_action`](crate::engine::Engine::on_action).
//! * An [`ActionGroup`] makes checkable actions exclusive: checking one unchecks the others.
//!
//! Shortcuts are scoped with [`ShortcutScope`] to the whole application, to the window, or to a
//! `Widget` subtree.  [`StandardShortcut`] names the platform-standard keys (Save is Cmd+S on macOS
//! and Ctrl+S elsewhere).
//!
//! This module holds the data and rules only; the `Engine` applies them to the `Widget`s.
//!
//! # Examples
//!
//! ```rust
//! use pushrod::action::{Action, StandardShortcut};
//!
//! let save = Action::new("Save")
//!     .standard_shortcut(StandardShortcut::Save)
//!     .tooltip("Save the document")
//!     .status_text("Writes the document to disk");
//!
//! assert!(save.enabled);
//! assert!(!save.checkable);
//! assert!(save.shortcut.is_some());
//! ```

use crate::cache::WidgetCache;
use crate::event::{EngineEvent, Event};
use crate::input::{Key, Modifiers};
use crate::widget::WidgetId;

/// The identifier of a registered [`Action`].  IDs are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(pub u32);

/// The identifier of an [`ActionGroup`].  IDs are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionGroupId(pub u32);

/// A key together with the modifiers that must be held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// The key.  A [`Key::Char`] is compared in lowercase.
    pub key: Key,
    /// The modifiers that must be held, exactly.
    pub modifiers: Modifiers,
}

impl Shortcut {
    /// Creates a shortcut.
    ///
    /// # Arguments
    ///
    /// * `key` - The key.
    /// * `modifiers` - The modifiers that must be held.
    ///
    /// # Returns
    ///
    /// The shortcut, with a character key stored in lowercase.
    pub fn new(key: Key, modifiers: Modifiers) -> Self {
        let key = match key {
            Key::Char(c) => Key::Char(c.to_lowercase().next().unwrap_or(c)),
            other => other,
        };

        Self { key, modifiers }
    }

    /// Returns whether a key press is this shortcut.
    ///
    /// # Arguments
    ///
    /// * `key` - The key pressed.
    /// * `modifiers` - The modifiers held.
    ///
    /// # Returns
    ///
    /// `true` if the key and the modifiers both match exactly.
    pub fn matches(&self, key: Key, modifiers: Modifiers) -> bool {
        *self == Shortcut::new(key, modifiers)
    }
}

/// A platform-standard shortcut, resolved to the right keys for the platform.
///
/// The command modifier is Cmd on macOS and Ctrl elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StandardShortcut {
    /// New: command+N.
    New,
    /// Open: command+O.
    Open,
    /// Save: command+S.
    Save,
    /// Close: command+W.
    Close,
    /// Quit: command+Q.
    Quit,
    /// Undo: command+Z.
    Undo,
    /// Redo: command+Shift+Z on macOS, Ctrl+Y elsewhere.
    Redo,
    /// Cut: command+X.
    Cut,
    /// Copy: command+C.
    Copy,
    /// Paste: command+V.
    Paste,
    /// Select all: command+A.
    SelectAll,
    /// Find: command+F.
    Find,
    /// Print: command+P.
    Print,
}

impl StandardShortcut {
    /// Resolves the shortcut for the platform the program is running on.
    ///
    /// # Returns
    ///
    /// The [`Shortcut`] for macOS when built for macOS, otherwise for the other platforms.
    pub fn shortcut(self) -> Shortcut {
        self.shortcut_for(cfg!(target_os = "macos"))
    }

    /// Resolves the shortcut for a named platform family.
    ///
    /// # Arguments
    ///
    /// * `macos` - `true` for macOS (Cmd), `false` for Windows and Linux (Ctrl).
    ///
    /// # Returns
    ///
    /// The [`Shortcut`].
    pub fn shortcut_for(self, macos: bool) -> Shortcut {
        let command = if macos {
            Modifiers::GUI
        } else {
            Modifiers::CTRL
        };

        match self {
            StandardShortcut::New => Shortcut::new(Key::Char('n'), command),
            StandardShortcut::Open => Shortcut::new(Key::Char('o'), command),
            StandardShortcut::Save => Shortcut::new(Key::Char('s'), command),
            StandardShortcut::Close => Shortcut::new(Key::Char('w'), command),
            StandardShortcut::Quit => Shortcut::new(Key::Char('q'), command),
            StandardShortcut::Undo => Shortcut::new(Key::Char('z'), command),
            StandardShortcut::Redo if macos => {
                Shortcut::new(Key::Char('z'), command | Modifiers::SHIFT)
            }
            StandardShortcut::Redo => Shortcut::new(Key::Char('y'), command),
            StandardShortcut::Cut => Shortcut::new(Key::Char('x'), command),
            StandardShortcut::Copy => Shortcut::new(Key::Char('c'), command),
            StandardShortcut::Paste => Shortcut::new(Key::Char('v'), command),
            StandardShortcut::SelectAll => Shortcut::new(Key::Char('a'), command),
            StandardShortcut::Find => Shortcut::new(Key::Char('f'), command),
            StandardShortcut::Print => Shortcut::new(Key::Char('p'), command),
        }
    }
}

/// Where an action's shortcut works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShortcutScope {
    /// Always, even while a modal dialog is open.
    Application,
    /// Whenever the window is usable: not while a modal dialog is open.  This is the default.
    Window,
    /// Only while the keyboard focus is on this `Widget` or one of its descendants.
    Subtree(WidgetId),
}

/// One command, shared by every control that performs it.
///
/// Build it with [`Action::new`] and the `with`-style methods, then register it with
/// [`Engine::add_action`](crate::engine::Engine::add_action).  The fields are public so a
/// callback can read them; change them through the `Engine` or `Ui` so bound controls follow.
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    /// The ID assigned when the action was registered; `ActionId(0)` before that.
    pub id: ActionId,
    /// The text shown on bound controls.
    pub label: String,
    /// The name of the icon resource for bound controls, if any.
    pub icon: Option<String>,
    /// The keyboard shortcut, if any.
    pub shortcut: Option<Shortcut>,
    /// Where the shortcut works.
    pub scope: ShortcutScope,
    /// The tooltip text for bound controls.
    pub tooltip: String,
    /// The longer description for a status bar.
    pub status_text: String,
    /// Whether the action has an on/off state.
    pub checkable: bool,
    /// Whether the action is on.  Only meaningful when `checkable`.
    pub checked: bool,
    /// Whether the action can be triggered.
    pub enabled: bool,
    /// The exclusive group the action belongs to, if any.
    pub group: Option<ActionGroupId>,
}

impl Action {
    /// Creates an enabled, unchecked, non-checkable action with a label.
    ///
    /// # Arguments
    ///
    /// * `label` - The text for bound controls.
    ///
    /// # Returns
    ///
    /// The new action, with no icon, shortcut, tooltip, or status text.
    pub fn new(label: &str) -> Self {
        Self {
            id: ActionId(0),
            label: label.to_string(),
            icon: None,
            shortcut: None,
            scope: ShortcutScope::Window,
            tooltip: String::new(),
            status_text: String::new(),
            checkable: false,
            checked: false,
            enabled: true,
            group: None,
        }
    }

    /// Sets the icon resource name.
    ///
    /// # Arguments
    ///
    /// * `icon` - The icon name.
    ///
    /// # Returns
    ///
    /// The action with the icon set.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Sets the keyboard shortcut.
    ///
    /// # Arguments
    ///
    /// * `shortcut` - The key and modifiers.
    ///
    /// # Returns
    ///
    /// The action with the shortcut set.
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Sets the keyboard shortcut to a platform-standard one.
    ///
    /// # Arguments
    ///
    /// * `standard` - The standard shortcut, resolved for the running platform.
    ///
    /// # Returns
    ///
    /// The action with the shortcut set.
    pub fn standard_shortcut(self, standard: StandardShortcut) -> Self {
        self.shortcut(standard.shortcut())
    }

    /// Sets where the shortcut works.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope.
    ///
    /// # Returns
    ///
    /// The action with the scope set.
    pub fn scope(mut self, scope: ShortcutScope) -> Self {
        self.scope = scope;
        self
    }

    /// Sets the tooltip.
    ///
    /// # Arguments
    ///
    /// * `tooltip` - The tooltip text.
    ///
    /// # Returns
    ///
    /// The action with the tooltip set.
    pub fn tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = tooltip.to_string();
        self
    }

    /// Sets the status-bar text.
    ///
    /// # Arguments
    ///
    /// * `status_text` - The description.
    ///
    /// # Returns
    ///
    /// The action with the status text set.
    pub fn status_text(mut self, status_text: &str) -> Self {
        self.status_text = status_text.to_string();
        self
    }

    /// Makes the action checkable, starting in the given state.
    ///
    /// # Arguments
    ///
    /// * `checked` - The starting state.
    ///
    /// # Returns
    ///
    /// The action, now checkable.
    pub fn checkable(mut self, checked: bool) -> Self {
        self.checkable = true;
        self.checked = checked;
        self
    }

    /// Starts the action disabled or enabled.
    ///
    /// # Arguments
    ///
    /// * `enabled` - The starting state.
    ///
    /// # Returns
    ///
    /// The action with the enabled state set.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A set of checkable actions of which at most one, and once chosen exactly one, is checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionGroup {
    /// The group's ID.
    pub id: ActionGroupId,
    /// Whether checking one member unchecks the others.
    pub exclusive: bool,
}

/// The registered actions, the groups, and which `Widget`s are bound to which action.
#[derive(Debug, Default)]
pub struct Actions {
    actions: Vec<Action>,
    groups: Vec<ActionGroup>,
    bindings: Vec<(WidgetId, ActionId)>,
}

impl Actions {
    /// Creates a group.
    ///
    /// # Arguments
    ///
    /// * `exclusive` - `true` if checking one member unchecks the others.
    ///
    /// # Returns
    ///
    /// The ID of the new group.
    pub fn add_group(&mut self, exclusive: bool) -> ActionGroupId {
        let id = ActionGroupId(self.groups.len() as u32 + 1);

        self.groups.push(ActionGroup { id, exclusive });
        id
    }

    /// Registers an action.  Joining an exclusive group with other members checked leaves this
    /// one unchecked; the first checked member of a group stays checked.
    ///
    /// # Arguments
    ///
    /// * `action` - The action; its `id` is replaced.
    ///
    /// # Returns
    ///
    /// The ID assigned to the action.
    pub fn add(&mut self, mut action: Action) -> ActionId {
        let id = ActionId(self.actions.len() as u32 + 1);

        action.id = id;

        if let Some(group) = action.group
            && self.is_exclusive(group)
            && action.checked
            && self
                .actions
                .iter()
                .any(|a| a.group == Some(group) && a.checked)
        {
            action.checked = false;
        }

        self.actions.push(action);
        id
    }

    /// Returns an action.
    ///
    /// # Arguments
    ///
    /// * `id` - The action's ID.
    ///
    /// # Returns
    ///
    /// The action, or `None` if the ID is not registered.
    pub fn get(&self, id: ActionId) -> Option<&Action> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Returns how many actions are registered.
    ///
    /// # Returns
    ///
    /// The number of actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Returns whether no action is registered.
    ///
    /// # Returns
    ///
    /// `true` if there are no actions.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Returns the actions in a group.
    ///
    /// # Arguments
    ///
    /// * `group` - The group.
    ///
    /// # Returns
    ///
    /// The members' IDs, in registration order.
    pub fn members(&self, group: ActionGroupId) -> Vec<ActionId> {
        self.actions
            .iter()
            .filter(|a| a.group == Some(group))
            .map(|a| a.id)
            .collect()
    }

    /// Returns whether a group is exclusive.
    fn is_exclusive(&self, group: ActionGroupId) -> bool {
        self.groups.iter().any(|g| g.id == group && g.exclusive)
    }

    /// Returns the action a `Widget` is bound to.
    ///
    /// # Arguments
    ///
    /// * `widget` - The `Widget`.
    ///
    /// # Returns
    ///
    /// The action's ID, or `None` if the `Widget` is not bound.
    pub fn bound_action(&self, widget: WidgetId) -> Option<ActionId> {
        self.bindings
            .iter()
            .find(|(w, _)| *w == widget)
            .map(|(_, a)| *a)
    }

    /// Binds a `Widget` to an action and copies the action's state onto it.  A `Widget` is bound
    /// to one action; binding it again replaces the old binding.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    /// * `action` - The action.
    /// * `widget` - The `Widget`.
    ///
    /// # Returns
    ///
    /// `true` if bound; `false` if the action or the `Widget` does not exist.
    pub fn bind(&mut self, cache: &mut WidgetCache, action: ActionId, widget: WidgetId) -> bool {
        if self.get(action).is_none() || cache.get(widget).is_none() {
            return false;
        }

        self.bindings.retain(|(w, _)| *w != widget);
        self.bindings.push((widget, action));
        self.sync_widget(cache, action, widget);
        true
    }

    /// Forgets the bindings of a removed `Widget`.
    ///
    /// # Arguments
    ///
    /// * `widget` - The `Widget` that was removed.
    pub fn unbind_widget(&mut self, widget: WidgetId) {
        self.bindings.retain(|(w, _)| *w != widget);
    }

    /// Copies an action's state onto one bound `Widget`.  A `Widget` without a matching property
    /// (a button has no `checked`) simply keeps its own.
    fn sync_widget(&self, cache: &mut WidgetCache, action: ActionId, widget: WidgetId) {
        use crate::registry::PropertyValue;

        let Some(a) = self.get(action) else {
            return;
        };
        let Some(w) = cache.get_mut(widget) else {
            return;
        };

        let _ = w.set_property("text", &PropertyValue::Text(a.label.clone()));
        let _ = w.set_property("tooltip", &PropertyValue::Text(a.tooltip.clone()));
        let _ = w.set_property("enabled", &PropertyValue::Bool(a.enabled));

        if a.checkable {
            let _ = w.set_property("checked", &PropertyValue::Bool(a.checked));
        }
    }

    /// Copies an action's state onto every `Widget` bound to it and queues
    /// [`EngineEvent::ActionChanged`].
    fn changed(&self, cache: &mut WidgetCache, action: ActionId) {
        for (widget, _) in self.bindings.iter().filter(|(_, a)| *a == action) {
            self.sync_widget(cache, action, *widget);
        }

        cache.queue_event(Event::Engine(EngineEvent::ActionChanged { action }));
    }

    /// Enables or disables an action and every control bound to it.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    /// * `id` - The action.
    /// * `enabled` - The new state.
    ///
    /// # Returns
    ///
    /// `true` if the state changed; `false` if it was already set or the action is unknown.
    pub fn set_enabled(&mut self, cache: &mut WidgetCache, id: ActionId, enabled: bool) -> bool {
        match self.actions.iter_mut().find(|a| a.id == id) {
            Some(a) if a.enabled != enabled => a.enabled = enabled,
            _ => return false,
        }

        self.changed(cache, id);
        true
    }

    /// Checks or unchecks an action, and every control bound to it.  In an exclusive group,
    /// checking an action unchecks the others (each reports a change), and unchecking the
    /// checked one is refused.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    /// * `id` - The action.
    /// * `checked` - The new state.
    ///
    /// # Returns
    ///
    /// `true` if the state changed; `false` if the action is unknown or not checkable, already
    /// has that state, or the group refused.
    pub fn set_checked(&mut self, cache: &mut WidgetCache, id: ActionId, checked: bool) -> bool {
        let Some(action) = self.get(id) else {
            return false;
        };

        if !action.checkable || action.checked == checked {
            return false;
        }

        let group = action.group.filter(|g| self.is_exclusive(*g));

        if group.is_some() && !checked {
            // A radio-style group always keeps one member checked.
            self.changed(cache, id);
            return false;
        }

        let mut touched = vec![id];

        if let Some(group) = group {
            for other in self.actions.iter_mut() {
                if other.group == Some(group) && other.id != id && other.checked {
                    other.checked = false;
                    touched.push(other.id);
                }
            }
        }

        if let Some(a) = self.actions.iter_mut().find(|a| a.id == id) {
            a.checked = checked;
        }

        for changed in touched {
            self.changed(cache, changed);
        }

        true
    }

    /// Sets an action's label on every bound control.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    /// * `id` - The action.
    /// * `label` - The new label.
    ///
    /// # Returns
    ///
    /// `true` if the label changed.
    pub fn set_label(&mut self, cache: &mut WidgetCache, id: ActionId, label: &str) -> bool {
        match self.actions.iter_mut().find(|a| a.id == id) {
            Some(a) if a.label != label => a.label = label.to_string(),
            _ => return false,
        }

        self.changed(cache, id);
        true
    }

    /// Triggers an action: toggles it if checkable and not in an exclusive group, then queues
    /// [`EngineEvent::ActionInvoked`].  A disabled or unknown action does nothing.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    /// * `id` - The action.
    ///
    /// # Returns
    ///
    /// `true` if the action was triggered.
    pub fn trigger(&mut self, cache: &mut WidgetCache, id: ActionId) -> bool {
        let Some(action) = self.get(id) else {
            return false;
        };

        if !action.enabled {
            return false;
        }

        if action.checkable {
            let next = match action.group.filter(|g| self.is_exclusive(*g)) {
                Some(_) => true,
                None => !action.checked,
            };

            self.set_checked(cache, id, next);
        }

        cache.queue_event(Event::Engine(EngineEvent::ActionInvoked { action: id }));
        true
    }

    /// Handles a bound control that was clicked or toggled.  For a toggle, the action takes the
    /// control's new state first, so the control is never ahead of the action.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    /// * `widget` - The control.
    /// * `toggled` - `Some(new state)` if the control toggled, `None` if it was clicked.
    ///
    /// # Returns
    ///
    /// The action triggered, if the control is bound and the action accepted it.
    pub fn control_activated(
        &mut self,
        cache: &mut WidgetCache,
        widget: WidgetId,
        toggled: Option<bool>,
    ) -> Option<ActionId> {
        let id = self.bound_action(widget)?;
        let action = self.get(id)?;

        if !action.checkable {
            // The control toggled by itself; put it back in step with the action.
            if toggled.is_some() {
                self.sync_widget(cache, id, widget);
            }

            return self.trigger(cache, id).then_some(id);
        }

        if !action.enabled {
            self.sync_widget(cache, id, widget);
            return None;
        }

        match toggled {
            Some(state) => {
                self.set_checked(cache, id, state);
                self.sync_widget(cache, id, widget);
                cache.queue_event(Event::Engine(EngineEvent::ActionInvoked { action: id }));
                Some(id)
            }
            None => self.trigger(cache, id).then_some(id),
        }
    }

    /// Finds the enabled action whose shortcut matches a key press and whose scope allows it now.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s, for the focus and the modal state.
    /// * `key` - The key pressed.
    /// * `modifiers` - The modifiers held.
    ///
    /// # Returns
    ///
    /// The first matching action in registration order, or `None`.
    pub fn find_shortcut(
        &self,
        cache: &WidgetCache,
        key: Key,
        modifiers: Modifiers,
    ) -> Option<ActionId> {
        self.actions
            .iter()
            .filter(|a| a.enabled)
            .filter(|a| a.shortcut.is_some_and(|s| s.matches(key, modifiers)))
            .find(|a| scope_allows(cache, a.scope))
            .map(|a| a.id)
    }
}

/// Returns whether a shortcut scope is active right now.
///
/// # Arguments
///
/// * `cache` - The `Widget`s.
/// * `scope` - The scope to test.
///
/// # Returns
///
/// `true` if the shortcut may fire.
fn scope_allows(cache: &WidgetCache, scope: ShortcutScope) -> bool {
    match scope {
        ShortcutScope::Application => true,
        ShortcutScope::Window => cache.top_modal().is_none(),
        ShortcutScope::Subtree(root) => {
            let mut current = cache.focused();

            while let Some(id) = current {
                if id == root {
                    return true;
                }

                let parent = cache.get(id).map(|w| w.get_parent());

                current = parent.filter(|p| *p != id);
            }

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_shortcuts_follow_the_platform() {
        let mac = StandardShortcut::Save.shortcut_for(true);
        let other = StandardShortcut::Save.shortcut_for(false);

        assert_eq!(mac.modifiers, Modifiers::GUI);
        assert_eq!(other.modifiers, Modifiers::CTRL);
        assert_eq!(mac.key, Key::Char('s'));
        assert_eq!(
            StandardShortcut::Redo.shortcut_for(true).modifiers,
            Modifiers::GUI | Modifiers::SHIFT
        );
        assert_eq!(
            StandardShortcut::Redo.shortcut_for(false).key,
            Key::Char('y')
        );
    }

    #[test]
    fn shortcut_matches_ignoring_character_case() {
        let s = Shortcut::new(Key::Char('S'), Modifiers::CTRL);

        assert!(s.matches(Key::Char('s'), Modifiers::CTRL));
        assert!(!s.matches(Key::Char('s'), Modifiers::empty()));
        assert!(!s.matches(Key::Char('s'), Modifiers::CTRL | Modifiers::SHIFT));
    }

    #[test]
    fn exclusive_group_keeps_one_checked() {
        let mut cache = WidgetCache::default();
        let mut actions = Actions::default();
        let group = actions.add_group(true);
        let mut left = Action::new("Left").checkable(true);
        let mut right = Action::new("Right").checkable(false);

        left.group = Some(group);
        right.group = Some(group);

        let left = actions.add(left);
        let right = actions.add(right);

        assert!(actions.set_checked(&mut cache, right, true));
        assert!(!actions.get(left).unwrap().checked);
        assert!(!actions.set_checked(&mut cache, right, false));
        assert!(actions.get(right).unwrap().checked);
    }

    #[test]
    fn joining_a_group_checked_twice_keeps_the_first() {
        let mut actions = Actions::default();
        let group = actions.add_group(true);
        let mut a = Action::new("A").checkable(true);
        let mut b = Action::new("B").checkable(true);

        a.group = Some(group);
        b.group = Some(group);

        let a = actions.add(a);
        let b = actions.add(b);

        assert!(actions.get(a).unwrap().checked);
        assert!(!actions.get(b).unwrap().checked);
    }

    #[test]
    fn disabled_action_does_not_trigger() {
        let mut cache = WidgetCache::default();
        let mut actions = Actions::default();
        let id = actions.add(Action::new("Save").enabled(false));

        assert!(!actions.trigger(&mut cache, id));
        assert!(actions.set_enabled(&mut cache, id, true));
        assert!(actions.trigger(&mut cache, id));
    }

    #[test]
    fn triggering_toggles_a_checkable_action() {
        let mut cache = WidgetCache::default();
        let mut actions = Actions::default();
        let id = actions.add(Action::new("Bold").checkable(false));

        actions.trigger(&mut cache, id);
        assert!(actions.get(id).unwrap().checked);
        actions.trigger(&mut cache, id);
        assert!(!actions.get(id).unwrap().checked);
    }

    #[test]
    fn unchanged_state_reports_no_change() {
        let mut cache = WidgetCache::default();
        let mut actions = Actions::default();
        let id = actions.add(Action::new("Save"));

        assert!(!actions.set_enabled(&mut cache, id, true));
        assert!(!actions.set_checked(&mut cache, id, true));
        assert!(!actions.set_label(&mut cache, id, "Save"));
        assert!(actions.set_label(&mut cache, id, "Save As"));
    }

    #[test]
    fn unknown_ids_are_ignored() {
        let mut cache = WidgetCache::default();
        let mut actions = Actions::default();

        assert!(!actions.set_enabled(&mut cache, ActionId(9), false));
        assert!(!actions.trigger(&mut cache, ActionId(9)));
        assert!(!actions.bind(&mut cache, ActionId(9), 1));
        assert!(actions.get(ActionId(9)).is_none());
    }
}
