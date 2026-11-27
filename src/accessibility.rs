// Accessibility
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

//! Accessibility: what a screen reader can learn about the `Widget`s.
//!
//! Pushrod draws every control itself, so the operating system cannot see them.  Each `Widget`
//! describes itself with [`Widget::accessibility`], which
//! returns an [`AccessInfo`]: a [`Role`], a name, a value, and state flags.  The
//! [`AccessTree`] walks the `Widget` tree and keeps what it last published, so each
//! [`AccessTree::update`] returns an [`AccessUpdate`] with **only the nodes that changed** (the
//! first update, after activation, holds all of them).  Nodes are keyed by the `Widget`'s stable
//! [`WidgetId`].
//!
//! The tree costs nothing until assistive technology asks for it: the [`Engine`](crate::engine::Engine)
//! builds none until a backend reports [`InputEvent::AccessibilityActivated`](crate::input::InputEvent::AccessibilityActivated)
//! (or the application calls
//! [`Engine::set_accessibility_active`](crate::engine::Engine::set_accessibility_active)).
//!
//! Requests from assistive technology arrive as [`AccessAction`]s.  They map onto normal Pushrod
//! behavior, so the same callbacks run as for user input: [`AccessAction::Click`] sends
//! [`Event::Activated`](crate::event::Event::Activated), [`AccessAction::Focus`] moves keyboard
//! focus, and [`AccessAction::SetValue`] / [`AccessAction::SetText`] set the `value` / `text`
//! property and send [`Event::ValueChanged`](crate::event::Event::ValueChanged) /
//! [`Event::TextChanged`](crate::event::Event::TextChanged).
//!
//! With the `accesskit` cargo feature, [`AccessUpdate::to_accesskit`] converts an update to an
//! [AccessKit](https://github.com/AccessKit/accesskit) `TreeUpdate`, which a platform adapter
//! publishes to Windows, macOS, and Linux accessibility APIs.
//!
//! # Examples
//!
//! ```rust
//! use pushrod::accessibility::AccessTree;
//! use pushrod::engine::Engine;
//! use pushrod::geometry::size;
//!
//! let mut engine: Engine = Engine::new(size(200, 100), 30);
//!
//! // Nothing is built while no assistive technology is running.
//! assert!(engine.accessibility_update().is_none());
//!
//! engine.set_accessibility_active(true);
//!
//! let first = engine.accessibility_update().expect("full tree");
//! assert!(first.full);
//!
//! // Nothing changed, so there is nothing to publish.
//! assert!(engine.accessibility_update().is_none());
//!
//! let _ = AccessTree::new();
//! ```

use crate::cache::WidgetCache;
use crate::geometry::Rect;
use crate::widget::{Behavior, ROOT_WIDGET_ID, VisualState, Widget, WidgetId};
use bitflags::bitflags;
use std::collections::HashMap;

/// What kind of control a `Widget` is, as a screen reader announces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Role {
    /// The application window.
    Window,
    /// A `Widget` with no more specific meaning; the default.
    #[default]
    Generic,
    /// A container that groups related controls.
    Group,
    /// A dialog box.
    Dialog,
    /// Static text.
    Label,
    /// A picture.
    Image,
    /// A push button.
    Button,
    /// A check box.
    CheckBox,
    /// A radio button.
    RadioButton,
    /// An editable text field.
    TextInput,
    /// An editable password field.
    PasswordInput,
    /// A slider.
    Slider,
    /// A scroll bar.
    ScrollBar,
    /// A progress bar.
    ProgressIndicator,
    /// A line that divides controls.
    Separator,
}

/// The checked condition of a check box, radio button, or toggle button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Toggled {
    /// Not checked.
    False,
    /// Checked.
    True,
    /// Neither (a tri-state control).
    Mixed,
}

impl From<bool> for Toggled {
    /// `true` is [`Toggled::True`]; `false` is [`Toggled::False`].
    ///
    /// # Arguments
    ///
    /// * `checked` - Whether the control is checked.
    ///
    /// # Returns
    ///
    /// The matching condition.
    fn from(checked: bool) -> Self {
        if checked {
            Toggled::True
        } else {
            Toggled::False
        }
    }
}

bitflags! {
    /// The actions assistive technology may request of a `Widget`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct AccessActions: u8 {
        /// Press or toggle the `Widget`.
        const CLICK = 1 << 0;
        /// Move keyboard focus to the `Widget`.
        const FOCUS = 1 << 1;
        /// Set a numeric value (a slider or scroll bar).
        const SET_VALUE = 1 << 2;
        /// Replace the text (a text field).
        const SET_TEXT = 1 << 3;
    }
}

/// A request from assistive technology.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessAction {
    /// Press or toggle the `Widget`, as a click would.
    Click,
    /// Move keyboard focus to the `Widget`.
    Focus,
    /// Set the `Widget`'s numeric `value` property.
    SetValue(f64),
    /// Set the `Widget`'s `text` property.
    SetText(String),
}

/// Which kind of [`AccessAction`] was requested, without its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessActionKind {
    /// See [`AccessAction::Click`].
    Click,
    /// See [`AccessAction::Focus`].
    Focus,
    /// See [`AccessAction::SetValue`].
    SetValue,
    /// See [`AccessAction::SetText`].
    SetText,
}

impl AccessAction {
    /// Returns the kind of the action, without its data.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn kind(&self) -> AccessActionKind {
        match self {
            AccessAction::Click => AccessActionKind::Click,
            AccessAction::Focus => AccessActionKind::Focus,
            AccessAction::SetValue(_) => AccessActionKind::SetValue,
            AccessAction::SetText(_) => AccessActionKind::SetText,
        }
    }
}

/// A numeric value with its range, for a slider, scroll bar, or progress bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericValue {
    /// The current value.
    pub value: f64,
    /// The smallest value.
    pub min: f64,
    /// The largest value.
    pub max: f64,
    /// How much one step changes the value, or `None` if there is no step.
    pub step: Option<f64>,
}

/// Where the caret and selection are in a text field, as character (not byte) indexes into the
/// [`AccessInfo::value`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    /// The end of the selection that stays put; equal to `caret` if nothing is selected.
    pub anchor: usize,
    /// The end of the selection that moves, or the caret.
    pub caret: usize,
}

/// What a screen reader is told about one `Widget`.
///
/// Build one with [`AccessInfo::new`] and the `with_*` methods.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccessInfo {
    /// What kind of control it is.
    pub role: Role,
    /// The accessible name: the label a screen reader reads (without mnemonic underscores).
    pub name: String,
    /// Extra help text; usually the tooltip.
    pub description: String,
    /// The text value: a text field's content (masked for a password), or a readable form of a
    /// numeric value.
    pub value: Option<String>,
    /// The numeric value and its range.
    pub numeric: Option<NumericValue>,
    /// The checked condition, if the control can be checked.
    pub toggled: Option<Toggled>,
    /// The `Widget` is disabled.
    pub disabled: bool,
    /// The `Widget` is read only.
    pub read_only: bool,
    /// The caret and selection of a text field.
    pub selection: Option<TextSelection>,
    /// The actions the `Widget` accepts.
    pub actions: AccessActions,
}

impl AccessInfo {
    /// Creates the description of a `Widget` of a role, with no name, value, or actions.
    ///
    /// # Arguments
    ///
    /// * `role` - What kind of control it is.
    ///
    /// # Returns
    ///
    /// The description.
    pub fn new(role: Role) -> Self {
        AccessInfo {
            role,
            ..AccessInfo::default()
        }
    }

    /// Starts the description of a `Widget` with what every `Widget` shares: its tooltip as the
    /// description, the checked condition of a [`Behavior::TOGGLE`] `Widget`, and the
    /// [`AccessActions::FOCUS`] action when it is focusable.  Override
    /// [`Widget::accessibility`] by building on it.
    ///
    /// # Arguments
    ///
    /// * `widget` - The `Widget` described.
    /// * `role` - What kind of control it is.
    ///
    /// # Returns
    ///
    /// The description to build on.
    pub fn of<W: Widget + ?Sized>(widget: &W, role: Role) -> Self {
        let mut info = AccessInfo::new(role);

        info.description = widget.tooltip().to_string();

        if widget.behavior().contains(Behavior::TOGGLE) {
            let state = widget.state();

            info.toggled = Some(if state.contains(VisualState::INDETERMINATE) {
                Toggled::Mixed
            } else {
                Toggled::from(state.contains(VisualState::CHECKED))
            });
        }

        if widget.is_focusable() {
            info.actions |= AccessActions::FOCUS;
        }

        info
    }

    /// Sets the accessible name.
    ///
    /// # Arguments
    ///
    /// * `name` - The label to read; a mnemonic underscore (`"_Save"`) is removed.
    ///
    /// # Returns
    ///
    /// The description, changed.
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = strip_mnemonic(name);
        self
    }

    /// Sets the text value.
    ///
    /// # Arguments
    ///
    /// * `value` - The text.
    ///
    /// # Returns
    ///
    /// The description, changed.
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the numeric value and its range.
    ///
    /// # Arguments
    ///
    /// * `numeric` - The value, range, and optional step.
    ///
    /// # Returns
    ///
    /// The description, changed.
    #[must_use]
    pub fn with_numeric(mut self, numeric: NumericValue) -> Self {
        self.numeric = Some(numeric);
        self
    }

    /// Sets the checked condition.
    ///
    /// # Arguments
    ///
    /// * `toggled` - The condition.
    ///
    /// # Returns
    ///
    /// The description, changed.
    #[must_use]
    pub fn with_toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Sets the caret and selection.
    ///
    /// # Arguments
    ///
    /// * `selection` - The anchor and caret, in characters.
    ///
    /// # Returns
    ///
    /// The description, changed.
    #[must_use]
    pub fn with_selection(mut self, selection: TextSelection) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Sets the actions the `Widget` accepts.
    ///
    /// # Arguments
    ///
    /// * `actions` - The accepted actions.
    ///
    /// # Returns
    ///
    /// The description, changed.
    #[must_use]
    pub fn with_actions(mut self, actions: AccessActions) -> Self {
        self.actions = actions;
        self
    }
}

/// Removes the mnemonic marker from a label: `"_Save"` is read as `"Save"`, and `"a__b"` as
/// `"a_b"`.
///
/// # Arguments
///
/// * `label` - The label, which may contain a `_` before its shortcut letter.
///
/// # Returns
///
/// The label as it is read aloud.
pub fn strip_mnemonic(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut chars = label.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '_' {
            if chars.peek() == Some(&'_') {
                out.push('_');
                chars.next();
            }
        } else {
            out.push(c);
        }
    }

    out
}

/// One node of the accessibility tree: a `Widget`, as published.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessNode {
    /// The `Widget`'s ID, which is the node's stable key.
    pub id: WidgetId,
    /// What the `Widget` reports about itself.
    pub info: AccessInfo,
    /// Where the `Widget` is, in window coordinates.
    pub bounds: Rect,
    /// The IDs of the visible child `Widget`s, in order.
    pub children: Vec<WidgetId>,
    /// The `Widget` has the keyboard focus.
    pub focused: bool,
}

/// The changes to publish since the last update.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessUpdate {
    /// The nodes that are new or changed; every node on the first update.
    pub nodes: Vec<AccessNode>,
    /// The IDs of nodes that are no longer in the tree (removed, hidden, or under a hidden
    /// parent).
    pub removed: Vec<WidgetId>,
    /// The `Widget` with keyboard focus, or `None` if the window itself has it.
    pub focus: Option<WidgetId>,
    /// `true` if this update holds the whole tree.
    pub full: bool,
}

/// Remembers what was last published and computes incremental [`AccessUpdate`]s.
#[derive(Debug, Default)]
pub struct AccessTree {
    known: HashMap<WidgetId, AccessNode>,
    focus: Option<WidgetId>,
    started: bool,
}

impl AccessTree {
    /// Creates an empty tree; its first update is a full one.
    ///
    /// # Returns
    ///
    /// An empty tree.
    pub fn new() -> Self {
        AccessTree::default()
    }

    /// Forgets what was published, so the next update is a full one (an assistive technology
    /// that reconnects needs the whole tree again).
    pub fn reset(&mut self) {
        self.known.clear();
        self.focus = None;
        self.started = false;
    }

    /// Returns the number of nodes last published.
    ///
    /// # Returns
    ///
    /// The node count.
    pub fn len(&self) -> usize {
        self.known.len()
    }

    /// Returns whether no node was published yet.
    ///
    /// # Returns
    ///
    /// `true` if no node was published.
    pub fn is_empty(&self) -> bool {
        self.known.is_empty()
    }

    /// Compares the `Widget` tree with what was last published.
    ///
    /// Hidden `Widget`s, and everything under them, are left out.  A `Widget` is included only if
    /// its [`AccessInfo`], bounds, children, or focus changed.  A parent whose children changed
    /// is included too, which is how a removal is published.
    ///
    /// # Arguments
    ///
    /// * `cache` - The `Widget`s.
    ///
    /// # Returns
    ///
    /// The changes, or `None` if nothing changed.
    pub fn update(&mut self, cache: &WidgetCache) -> Option<AccessUpdate> {
        let mut fresh: HashMap<WidgetId, AccessNode> = HashMap::new();
        let focus = cache.focused();

        collect(cache, ROOT_WIDGET_ID, focus, &mut fresh);

        let nodes: Vec<AccessNode> = {
            let mut changed: Vec<AccessNode> = fresh
                .values()
                .filter(|n| self.known.get(&n.id) != Some(*n))
                .cloned()
                .collect();

            changed.sort_by_key(|n| n.id);
            changed
        };
        let mut removed: Vec<WidgetId> = self
            .known
            .keys()
            .filter(|id| !fresh.contains_key(id))
            .copied()
            .collect();

        removed.sort_unstable();

        let full = !self.started;

        if !full && nodes.is_empty() && removed.is_empty() && focus == self.focus {
            return None;
        }

        self.known = fresh;
        self.focus = focus;
        self.started = true;

        Some(AccessUpdate {
            nodes,
            removed,
            focus,
            full,
        })
    }
}

/// Adds a `Widget` and its visible descendants to `out`.
fn collect(
    cache: &WidgetCache,
    id: WidgetId,
    focus: Option<WidgetId>,
    out: &mut HashMap<WidgetId, AccessNode>,
) {
    let Some(widget) = cache.get(id) else {
        return;
    };

    if widget.is_hidden() {
        return;
    }

    let origin = cache.absolute_origin(id);
    let size = widget.get_size();
    let children: Vec<WidgetId> = cache
        .children_of(id)
        .iter()
        .copied()
        .filter(|&c| cache.get(c).is_some_and(|w| !w.is_hidden()))
        .collect();
    let mut info = widget.accessibility();

    info.disabled |= !widget.is_enabled();

    out.insert(
        id,
        AccessNode {
            id,
            info,
            bounds: Rect {
                x: origin.x,
                y: origin.y,
                w: size.w,
                h: size.h,
            },
            children: children.clone(),
            focused: focus == Some(id),
        },
    );

    for child in children {
        collect(cache, child, focus, out);
    }
}

#[cfg(feature = "accesskit")]
mod kit {
    //! The AccessKit conversion.

    use super::*;
    use accesskit as ak;

    /// The node ID of the text run that holds a text field's characters.  Widget IDs are 32 bits,
    /// so setting bit 32 never collides with one.
    const TEXT_RUN_BIT: u64 = 1 << 32;

    /// Returns the AccessKit node ID of a `Widget`.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`'s ID.
    ///
    /// # Returns
    ///
    /// The node ID.
    pub fn node_id(id: WidgetId) -> ak::NodeId {
        ak::NodeId(u64::from(id))
    }

    /// Returns the `Widget` an AccessKit node ID names, or `None` for a text run.
    ///
    /// # Arguments
    ///
    /// * `id` - The AccessKit node ID of a request's target.
    ///
    /// # Returns
    ///
    /// The `Widget` ID, or `None` if the ID is not a `Widget`'s.
    pub fn widget_id(id: ak::NodeId) -> Option<WidgetId> {
        let raw = id.0;

        // A text run belongs to the text field that owns it.
        WidgetId::try_from(raw & !TEXT_RUN_BIT).ok()
    }

    /// Converts a request from AccessKit to an [`AccessAction`].
    ///
    /// # Arguments
    ///
    /// * `request` - The request an AccessKit adapter received.
    ///
    /// # Returns
    ///
    /// The `Widget` and the action, or `None` if the request is not one Pushrod handles.
    pub fn from_request(request: &ak::ActionRequest) -> Option<(WidgetId, AccessAction)> {
        let id = widget_id(request.target_node)?;
        let action = match (request.action, &request.data) {
            (ak::Action::Click, _) => AccessAction::Click,
            (ak::Action::Focus, _) => AccessAction::Focus,
            (ak::Action::SetValue, Some(ak::ActionData::NumericValue(n))) => {
                AccessAction::SetValue(*n)
            }
            (ak::Action::SetValue, Some(ak::ActionData::Value(text))) => {
                AccessAction::SetText(text.to_string())
            }
            _ => return None,
        };

        Some((id, action))
    }

    fn role(role: Role) -> ak::Role {
        match role {
            Role::Window => ak::Role::Window,
            Role::Generic => ak::Role::GenericContainer,
            Role::Group => ak::Role::Group,
            Role::Dialog => ak::Role::Dialog,
            Role::Label => ak::Role::Label,
            Role::Image => ak::Role::Image,
            Role::Button => ak::Role::Button,
            Role::CheckBox => ak::Role::CheckBox,
            Role::RadioButton => ak::Role::RadioButton,
            Role::TextInput => ak::Role::TextInput,
            Role::PasswordInput => ak::Role::PasswordInput,
            Role::Slider => ak::Role::Slider,
            Role::ScrollBar => ak::Role::ScrollBar,
            Role::ProgressIndicator => ak::Role::ProgressIndicator,
            Role::Separator => ak::Role::Splitter,
        }
    }

    fn rect(r: Rect) -> ak::Rect {
        ak::Rect {
            x0: f64::from(r.x),
            y0: f64::from(r.y),
            x1: f64::from(r.x) + f64::from(r.w),
            y1: f64::from(r.y) + f64::from(r.h),
        }
    }

    /// Builds the AccessKit nodes of one `Widget`: its own, and a text run for a text field.
    fn nodes_of(n: &AccessNode) -> Vec<(ak::NodeId, ak::Node)> {
        let info = &n.info;
        let mut node = ak::Node::new(role(info.role));
        let mut out = Vec::new();

        node.set_bounds(rect(n.bounds));
        node.set_children(n.children.iter().map(|&c| node_id(c)).collect::<Vec<_>>());

        if !info.name.is_empty() {
            node.set_label(info.name.as_str());
        }

        if !info.description.is_empty() {
            node.set_description(info.description.as_str());
        }

        if info.disabled {
            node.set_disabled();
        }

        if info.read_only {
            node.set_read_only();
        }

        if let Some(toggled) = info.toggled {
            node.set_toggled(match toggled {
                Toggled::False => ak::Toggled::False,
                Toggled::True => ak::Toggled::True,
                Toggled::Mixed => ak::Toggled::Mixed,
            });
        }

        if let Some(v) = info.numeric {
            node.set_numeric_value(v.value);
            node.set_min_numeric_value(v.min);
            node.set_max_numeric_value(v.max);

            if let Some(step) = v.step {
                node.set_numeric_value_step(step);
            }
        }

        if info.actions.contains(AccessActions::CLICK) {
            node.add_action(ak::Action::Click);
        }

        if info.actions.contains(AccessActions::FOCUS) {
            node.add_action(ak::Action::Focus);
        }

        if info
            .actions
            .intersects(AccessActions::SET_VALUE | AccessActions::SET_TEXT)
        {
            node.add_action(ak::Action::SetValue);
        }

        match (&info.value, info.selection) {
            (Some(text), Some(selection)) => {
                // A text field exposes its characters through a text run, which carries the
                // per-character lengths the selection is measured in.
                let run_id = ak::NodeId(u64::from(n.id) | TEXT_RUN_BIT);
                let mut run = ak::Node::new(ak::Role::TextRun);
                let lengths: Vec<u8> = text.chars().map(|c| c.len_utf8() as u8).collect();
                let count = lengths.len();

                run.set_bounds(rect(n.bounds));
                run.set_value(text.as_str());
                run.set_character_lengths(lengths);

                let position = |i: usize| ak::TextPosition {
                    node: run_id,
                    character_index: i.min(count),
                };

                node.set_text_selection(ak::TextSelection {
                    anchor: position(selection.anchor),
                    focus: position(selection.caret),
                });
                node.push_child(run_id);
                out.push((run_id, run));
            }
            (Some(text), None) => node.set_value(text.as_str()),
            _ => {}
        }

        out.insert(0, (node_id(n.id), node));
        out
    }

    impl AccessUpdate {
        /// Converts the update to an AccessKit `TreeUpdate`.
        ///
        /// Only the changed nodes are converted.  The first update holds the window's root node,
        /// so the platform adapter has a root to attach the tree to.
        ///
        /// # Returns
        ///
        /// The update for an AccessKit platform adapter.
        pub fn to_accesskit(&self) -> ak::TreeUpdate {
            let mut nodes = Vec::new();

            for n in &self.nodes {
                nodes.extend(nodes_of(n));
            }

            ak::TreeUpdate {
                nodes,
                tree: self.full.then(|| {
                    let mut tree = ak::TreeInfo::new(node_id(ROOT_WIDGET_ID));

                    tree.toolkit_name = Some("Pushrod".to_string());
                    tree
                }),
                tree_id: ak::TreeId::ROOT,
                focus: node_id(self.focus.unwrap_or(ROOT_WIDGET_ID)),
            }
        }
    }
}

#[cfg(feature = "accesskit")]
pub use kit::{from_request, node_id, widget_id};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnemonic_markers_are_not_read_aloud() {
        assert_eq!(strip_mnemonic("_Save"), "Save");
        assert_eq!(strip_mnemonic("a__b"), "a_b");
        assert_eq!(strip_mnemonic("Plain"), "Plain");
    }

    #[test]
    fn builder_sets_fields() {
        let info = AccessInfo::new(Role::Button)
            .with_name("_OK")
            .with_actions(AccessActions::CLICK);

        assert_eq!(info.role, Role::Button);
        assert_eq!(info.name, "OK");
        assert!(info.actions.contains(AccessActions::CLICK));
    }
}
