// Per-state appearance
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

//! # Per-state appearance
//!
//! A [`StateStyle`] is a set of optional look overrides (background, foreground, border color,
//! border width, font).  An [`Appearance`] holds one `StateStyle` for each visual state a `Widget`
//! can be in.  `Widget`s read the *effective* style for their current [`VisualState`] with
//! [`Widget::style`](crate::widget::Widget::style), so applications can say "this button turns blue
//! on hover" without any widget code.
//!
//! ```text
//! state change ──> effective style = appearance[state] ⟶ falls back to appearance.normal ⟶ built-in default
//!                   │
//!                   └─> repaint only if the effective style changed
//! ```
//!
//! # Examples
//!
//! ```rust
//! use pushrod::appearance::{Appearance, StateStyle};
//! use pushrod::color::Color;
//! use pushrod::widget::VisualState;
//!
//! let mut appearance = Appearance::default();
//!
//! appearance.set(
//!     VisualState::HOVERED,
//!     StateStyle {
//!         background: Some(Color::BLUE),
//!         ..StateStyle::default()
//!     },
//! );
//!
//! assert_eq!(appearance.effective(VisualState::HOVERED).background, Some(Color::BLUE));
//! assert_eq!(appearance.effective(VisualState::empty()).background, None);
//! ```

use crate::color::Color;
use crate::paint::TextStyle;
use crate::widget::VisualState;

/// Look overrides for one visual state.  A field left `None` is not overridden: it falls back to
/// the `normal` style, and then to the `Widget`'s built-in default.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateStyle {
    /// The background fill color.
    pub background: Option<Color>,
    /// The text (foreground) color.
    pub foreground: Option<Color>,
    /// The border color.
    pub border_color: Option<Color>,
    /// The border width in pixels.
    pub border_width: Option<u8>,
    /// The font, size, and alignment of the text.
    pub font: Option<TextStyle>,
}

impl StateStyle {
    /// Returns whether no field is overridden.
    ///
    /// # Returns
    ///
    /// `true` if every field is `None`.
    pub fn is_empty(&self) -> bool {
        *self == StateStyle::default()
    }

    /// Fills each field that is `None` from `fallback`.
    ///
    /// # Arguments
    ///
    /// * `fallback` - The style that supplies the missing fields.
    ///
    /// # Returns
    ///
    /// The merged style.
    pub fn or(self, fallback: &StateStyle) -> StateStyle {
        StateStyle {
            background: self.background.or(fallback.background),
            foreground: self.foreground.or(fallback.foreground),
            border_color: self.border_color.or(fallback.border_color),
            border_width: self.border_width.or(fallback.border_width),
            font: self.font.or_else(|| fallback.font.clone()),
        }
    }
}

/// The order in which active states are looked up: the first state that is set *and* has a style
/// wins.  `DISABLED` always beats the interaction states.
const PRIORITY: [VisualState; 6] = [
    VisualState::DISABLED,
    VisualState::DRAG_OVER,
    VisualState::PRESSED,
    VisualState::CHECKED,
    VisualState::HOVERED,
    VisualState::FOCUSED,
];

/// One [`StateStyle`] for each visual state of a `Widget`.
///
/// `INDETERMINATE` has no style of its own; it is drawn by the generic state overlay.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Appearance {
    /// The look with no state flags set; every other state falls back to it.
    pub normal: StateStyle,
    /// The mouse button is held down on the `Widget`.
    pub pressed: StateStyle,
    /// The pointer is inside the `Widget`.
    pub hovered: StateStyle,
    /// The `Widget` has keyboard focus.
    pub focused: StateStyle,
    /// The `Widget` is disabled.
    pub disabled: StateStyle,
    /// The `Widget` is checked or selected.
    pub checked: StateStyle,
    /// A drag is over the `Widget`.
    pub drag_over: StateStyle,
}

impl Appearance {
    /// Returns the style stored for one state.
    ///
    /// # Arguments
    ///
    /// * `state` - One visual state flag; an empty set selects `normal`.  Anything else without
    ///   its own style (such as `INDETERMINATE`) also selects `normal`.
    ///
    /// # Returns
    ///
    /// A reference to the stored style.
    pub fn get(&self, state: VisualState) -> &StateStyle {
        match state {
            s if s == VisualState::PRESSED => &self.pressed,
            s if s == VisualState::HOVERED => &self.hovered,
            s if s == VisualState::FOCUSED => &self.focused,
            s if s == VisualState::DISABLED => &self.disabled,
            s if s == VisualState::CHECKED => &self.checked,
            s if s == VisualState::DRAG_OVER => &self.drag_over,
            _ => &self.normal,
        }
    }

    /// Replaces the style stored for one state.
    ///
    /// # Arguments
    ///
    /// * `state` - One visual state flag; an empty set (or a flag with no style) sets `normal`.
    /// * `style` - The new style.
    pub fn set(&mut self, state: VisualState, style: StateStyle) {
        match state {
            s if s == VisualState::PRESSED => self.pressed = style,
            s if s == VisualState::HOVERED => self.hovered = style,
            s if s == VisualState::FOCUSED => self.focused = style,
            s if s == VisualState::DISABLED => self.disabled = style,
            s if s == VisualState::CHECKED => self.checked = style,
            s if s == VisualState::DRAG_OVER => self.drag_over = style,
            _ => self.normal = style,
        }
    }

    /// Resolves the style to draw for a set of state flags.
    ///
    /// The active states are checked in the order disabled, drag-over, pressed, checked, hovered,
    /// focused; the first one that has a style wins.  Fields it leaves unset come from `normal`.
    /// Fields still unset are `None`, meaning "use the `Widget`'s built-in default".
    ///
    /// # Arguments
    ///
    /// * `state` - The `Widget`'s current visual state flags.
    ///
    /// # Returns
    ///
    /// The effective style.
    pub fn effective(&self, state: VisualState) -> StateStyle {
        PRIORITY
            .iter()
            .filter(|flag| state.contains(**flag))
            .map(|flag| self.get(*flag))
            .find(|style| !style.is_empty())
            .cloned()
            .unwrap_or_default()
            .or(&self.normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn bg(color: Color) -> StateStyle {
        StateStyle {
            background: Some(color),
            ..StateStyle::default()
        }
    }

    #[test]
    fn empty_appearance_resolves_to_no_overrides() {
        let a = Appearance::default();

        assert!(a.effective(VisualState::empty()).is_empty());
        assert!(
            a.effective(VisualState::HOVERED | VisualState::PRESSED)
                .is_empty()
        );
    }

    #[test]
    fn unset_state_falls_back_to_normal() {
        let mut a = Appearance::default();

        a.set(VisualState::empty(), bg(Color::GRAY));

        assert_eq!(
            a.effective(VisualState::HOVERED).background,
            Some(Color::GRAY)
        );
    }

    #[test]
    fn unset_fields_fall_back_to_normal_field_by_field() {
        let mut a = Appearance::default();

        a.set(
            VisualState::empty(),
            StateStyle {
                background: Some(Color::WHITE),
                border_width: Some(1),
                ..StateStyle::default()
            },
        );
        a.set(VisualState::HOVERED, bg(Color::BLUE));

        let e = a.effective(VisualState::HOVERED);

        assert_eq!(e.background, Some(Color::BLUE));
        assert_eq!(e.border_width, Some(1));
    }

    #[test]
    fn priority_is_disabled_dragover_pressed_checked_hovered_focused() {
        let mut a = Appearance::default();

        a.set(VisualState::FOCUSED, bg(Color::GREEN));
        a.set(VisualState::HOVERED, bg(Color::CYAN));
        a.set(VisualState::CHECKED, bg(Color::YELLOW));
        a.set(VisualState::PRESSED, bg(Color::BLUE));
        a.set(VisualState::DRAG_OVER, bg(Color::MAGENTA));
        a.set(VisualState::DISABLED, bg(Color::GRAY));

        let all = VisualState::all();

        assert_eq!(a.effective(all).background, Some(Color::GRAY));
        assert_eq!(
            a.effective(all - VisualState::DISABLED).background,
            Some(Color::MAGENTA)
        );
        assert_eq!(
            a.effective(VisualState::PRESSED | VisualState::CHECKED | VisualState::HOVERED)
                .background,
            Some(Color::BLUE)
        );
        assert_eq!(
            a.effective(VisualState::CHECKED | VisualState::HOVERED)
                .background,
            Some(Color::YELLOW)
        );
        assert_eq!(
            a.effective(VisualState::HOVERED | VisualState::FOCUSED)
                .background,
            Some(Color::CYAN)
        );
    }

    #[test]
    fn an_active_state_without_a_style_is_skipped() {
        let mut a = Appearance::default();

        a.set(VisualState::HOVERED, bg(Color::CYAN));

        // PRESSED is active but has no style, so HOVERED still applies.
        assert_eq!(
            a.effective(VisualState::PRESSED | VisualState::HOVERED)
                .background,
            Some(Color::CYAN)
        );
    }

    #[test]
    fn set_and_get_round_trip_every_state() {
        let mut a = Appearance::default();
        let states = [
            VisualState::PRESSED,
            VisualState::HOVERED,
            VisualState::FOCUSED,
            VisualState::DISABLED,
            VisualState::CHECKED,
            VisualState::DRAG_OVER,
            VisualState::empty(),
        ];

        for (i, state) in states.iter().enumerate() {
            a.set(*state, bg(Color::RGB(i as u8, 0, 0)));
        }

        for (i, state) in states.iter().enumerate() {
            assert_eq!(a.get(*state).background, Some(Color::RGB(i as u8, 0, 0)));
        }
    }
}
