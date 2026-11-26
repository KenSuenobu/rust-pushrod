// Input
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

//! # Input
//!
//! Pushrod's own input vocabulary: [`Key`], [`Modifiers`], [`MouseButton`], and the
//! [`InputEvent`]s a [`Backend`](crate::backend::Backend) delivers to the engine.  No windowing
//! library type appears here, so a widget library never compiles against one.

use crate::event::EventMask;
use crate::geometry::{Point, Size};
use bitflags::bitflags;

/// A keyboard key, by what it means rather than where it sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    /// A printable key, as its lowercase character: `Char('a')`, `Char('.')`, `Char(' ')`.
    Char(char),
    /// The Tab key.
    Tab,
    /// The Return (Enter) key.
    Return,
    /// The Enter key on the numeric keypad.
    KpEnter,
    /// The Escape key.
    Escape,
    /// The Backspace key.
    Backspace,
    /// The Delete key.
    Delete,
    /// The Home key.
    Home,
    /// The End key.
    End,
    /// The Page Up key.
    PageUp,
    /// The Page Down key.
    PageDown,
    /// The Up arrow.
    Up,
    /// The Down arrow.
    Down,
    /// The Left arrow.
    Left,
    /// The Right arrow.
    Right,
    /// A function key, `F(1)` through `F(12)`.
    F(u8),
    /// A key the backend could not name; the number is the backend's own code.
    Other(u32),
}

bitflags! {
    /// The modifier keys held during a key event.  Left and right keys are not distinguished.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Modifiers: u8 {
        /// A Shift key.
        const SHIFT = 1;
        /// A Control key.
        const CTRL = 1 << 1;
        /// An Alt (Option) key.
        const ALT = 1 << 2;
        /// A GUI key (Command on macOS, Windows key elsewhere).
        const GUI = 1 << 3;
    }
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MouseButton {
    /// The left (primary) button.
    Left,
    /// The middle button.
    Middle,
    /// The right (secondary) button.
    Right,
    /// An extra button; the number is the backend's own code.
    Other(u8),
}

/// A window-system event a backend delivers to the engine, in window coordinates.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum InputEvent {
    /// The user asked to close the window.
    Quit,
    /// The window needs repainting.
    Exposed,
    /// The window changed size, in logical pixels.
    Resized(Size),
    /// The window's scale factor changed (it moved to a display with a different pixel
    /// density): the new number of physical pixels per logical pixel.
    ScaleChanged(f32),
    /// The window gained or lost keyboard focus.
    FocusChanged(bool),
    /// A mouse button went down.
    MouseDown {
        /// The pointer position.
        position: Point,
        /// The button.
        button: MouseButton,
        /// The click count (1 single, 2 double).
        clicks: u8,
    },
    /// A mouse button went up.
    MouseUp {
        /// The pointer position.
        position: Point,
        /// The button.
        button: MouseButton,
        /// The click count (1 single, 2 double).
        clicks: u8,
    },
    /// The pointer moved.
    MouseMove {
        /// The pointer position.
        position: Point,
    },
    /// The wheel or trackpad scrolled.
    Wheel {
        /// How far it scrolled, horizontally (`x`) and vertically (`y`).
        delta: Point,
    },
    /// A key went down (or repeated).
    KeyDown {
        /// The key.
        key: Key,
        /// The modifier keys held.
        modifiers: Modifiers,
        /// `true` if this is an auto-repeat.
        repeat: bool,
    },
    /// A key went up.
    KeyUp {
        /// The key.
        key: Key,
        /// The modifier keys held.
        modifiers: Modifiers,
    },
    /// Text was typed.
    Text(String),
    /// Another thread asked the loop to wake; see [`Backend::waker`](crate::backend::Backend::waker).
    Wake,
}

impl InputEvent {
    /// The [`EventMask`] class this event belongs to.
    ///
    /// # Returns
    ///
    /// The class, or `None` for events that are never filtered: close requests, exposure and
    /// resizing (and scale changes) keep the window alive and up to date.
    pub fn class(&self) -> Option<EventMask> {
        match self {
            InputEvent::Quit
            | InputEvent::Exposed
            | InputEvent::Resized(_)
            | InputEvent::ScaleChanged(_) => None,
            InputEvent::MouseDown { .. } | InputEvent::MouseUp { .. } => {
                Some(EventMask::POINTER_BUTTONS)
            }
            InputEvent::MouseMove { .. } => Some(EventMask::POINTER_MOTION | EventMask::HOVER),
            InputEvent::Wheel { .. } => Some(EventMask::SCROLL),
            InputEvent::KeyDown { .. } | InputEvent::KeyUp { .. } => Some(EventMask::KEYBOARD),
            InputEvent::Text(_) => Some(EventMask::TEXT),
            InputEvent::FocusChanged(_) => Some(EventMask::WINDOW),
            InputEvent::Wake => Some(EventMask::CUSTOM),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_and_repaint_events_are_never_filtered() {
        assert!(InputEvent::Quit.class().is_none());
        assert!(InputEvent::Exposed.class().is_none());
        assert!(InputEvent::Resized(Size { w: 1, h: 1 }).class().is_none());
        assert!(InputEvent::ScaleChanged(2.0).class().is_none());
    }

    #[test]
    fn input_events_map_to_their_mask_classes() {
        let down = InputEvent::KeyDown {
            key: Key::Tab,
            modifiers: Modifiers::empty(),
            repeat: false,
        };

        assert_eq!(down.class(), Some(EventMask::KEYBOARD));
        assert_eq!(InputEvent::Text("a".into()).class(), Some(EventMask::TEXT));
        assert_eq!(InputEvent::Wake.class(), Some(EventMask::CUSTOM));
        assert_eq!(
            InputEvent::FocusChanged(true).class(),
            Some(EventMask::WINDOW)
        );
    }

    #[test]
    fn modifiers_combine() {
        let m = Modifiers::SHIFT | Modifiers::GUI;

        assert!(m.contains(Modifiers::SHIFT));
        assert!(!m.intersects(Modifiers::CTRL | Modifiers::ALT));
    }
}
