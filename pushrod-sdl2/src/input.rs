// Input conversion
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

//! Converts `SDL2` input events, keys, modifiers and buttons to Pushrod's own types, and works
//! out which `SDL2` event types an event mask lets the backend disable.

use pushrod::event::EventMask;
use pushrod::geometry::{Point, Size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use sdl2::event::{Event, EventType, WindowEvent};
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::mouse::MouseButton as SdlButton;
use std::path::PathBuf;

/// The `SDL2` user event the run loop wakes on when another thread posts; it carries no data.
pub(crate) struct Wake;

/// Converts an `SDL2` keycode to a Pushrod [`Key`].
///
/// # Arguments
///
/// * `code` - The `SDL2` keycode.
///
/// # Returns
///
/// The key: a named key, `Char` for printable ASCII (lowercase), or `Other` with the raw code.
pub(crate) fn to_key(code: Keycode) -> Key {
    match code {
        Keycode::Tab => Key::Tab,
        Keycode::Return => Key::Return,
        Keycode::KpEnter => Key::KpEnter,
        Keycode::Escape => Key::Escape,
        Keycode::Backspace => Key::Backspace,
        Keycode::Delete => Key::Delete,
        Keycode::Home => Key::Home,
        Keycode::End => Key::End,
        Keycode::PageUp => Key::PageUp,
        Keycode::PageDown => Key::PageDown,
        Keycode::Up => Key::Up,
        Keycode::Down => Key::Down,
        Keycode::Left => Key::Left,
        Keycode::Right => Key::Right,
        Keycode::F1 => Key::F(1),
        Keycode::F2 => Key::F(2),
        Keycode::F3 => Key::F(3),
        Keycode::F4 => Key::F(4),
        Keycode::F5 => Key::F(5),
        Keycode::F6 => Key::F(6),
        Keycode::F7 => Key::F(7),
        Keycode::F8 => Key::F(8),
        Keycode::F9 => Key::F(9),
        Keycode::F10 => Key::F(10),
        Keycode::F11 => Key::F(11),
        Keycode::F12 => Key::F(12),
        other => match other.into_i32() {
            code @ 0x20..=0x7e => Key::Char((code as u8 as char).to_ascii_lowercase()),
            code => Key::Other(code as u32),
        },
    }
}

/// Converts `SDL2` modifier flags to Pushrod [`Modifiers`], merging left and right keys.
///
/// # Arguments
///
/// * `keymod` - The `SDL2` modifier flags.
///
/// # Returns
///
/// The modifiers.
pub(crate) fn to_modifiers(keymod: Mod) -> Modifiers {
    let mut out = Modifiers::empty();

    if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        out |= Modifiers::SHIFT;
    }

    if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
        out |= Modifiers::CTRL;
    }

    if keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) {
        out |= Modifiers::ALT;
    }

    if keymod.intersects(Mod::LGUIMOD | Mod::RGUIMOD) {
        out |= Modifiers::GUI;
    }

    out
}

/// Converts an `SDL2` mouse button to a Pushrod [`MouseButton`].
///
/// # Arguments
///
/// * `button` - The `SDL2` button.
///
/// # Returns
///
/// The button; `X1` and `X2` become `Other(1)` and `Other(2)`, an unknown button `Other(0)`.
pub(crate) fn to_button(button: SdlButton) -> MouseButton {
    match button {
        SdlButton::Left => MouseButton::Left,
        SdlButton::Middle => MouseButton::Middle,
        SdlButton::Right => MouseButton::Right,
        SdlButton::X1 => MouseButton::Other(1),
        SdlButton::X2 => MouseButton::Other(2),
        SdlButton::Unknown => MouseButton::Other(0),
    }
}

/// Converts an `SDL2` event to an [`InputEvent`].
///
/// # Arguments
///
/// * `event` - The `SDL2` event.
///
/// # Returns
///
/// The input event, or `None` for an event Pushrod does not use.
pub(crate) fn to_input_event(event: Event) -> Option<InputEvent> {
    Some(match event {
        Event::Quit { .. } => InputEvent::Quit,
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::Exposed => InputEvent::Exposed,
            WindowEvent::SizeChanged(w, h) => {
                InputEvent::Resized(Size::new(w.max(0) as u32, h.max(0) as u32))
            }
            WindowEvent::FocusGained => InputEvent::FocusChanged(true),
            WindowEvent::FocusLost => InputEvent::FocusChanged(false),
            _ => return None,
        },
        Event::MouseButtonDown {
            x,
            y,
            mouse_btn,
            clicks,
            ..
        } => InputEvent::MouseDown {
            position: Point::new(x, y),
            button: to_button(mouse_btn),
            clicks,
        },
        Event::MouseButtonUp {
            x,
            y,
            mouse_btn,
            clicks,
            ..
        } => InputEvent::MouseUp {
            position: Point::new(x, y),
            button: to_button(mouse_btn),
            clicks,
        },
        Event::MouseMotion { x, y, .. } => InputEvent::MouseMove {
            position: Point::new(x, y),
        },
        Event::MouseWheel { x, y, .. } => InputEvent::Wheel {
            delta: Point::new(x, y),
        },
        Event::KeyDown {
            keycode: Some(key),
            keymod,
            repeat,
            ..
        } => InputEvent::KeyDown {
            key: to_key(key),
            modifiers: to_modifiers(keymod),
            repeat,
        },
        Event::KeyUp {
            keycode: Some(key),
            keymod,
            ..
        } => InputEvent::KeyUp {
            key: to_key(key),
            modifiers: to_modifiers(keymod),
        },
        Event::TextInput { text, .. } => InputEvent::Text(text),
        Event::TextEditing {
            text,
            start,
            length,
            ..
        } => InputEvent::Composition {
            text,
            start,
            length,
        },
        other if other.as_user_event_type::<Wake>().is_some() => InputEvent::Wake,
        _ => return None,
    })
}

/// Converts a batch of `SDL2` events, gathering the files of each drop into one
/// [`InputEvent::FilesDropped`].
///
/// `SDL2` reports a drop as `DropBegin`, one `DropFile` per file, then `DropComplete`, without a
/// position, so the pointer position is supplied by the caller.
///
/// # Arguments
///
/// * `events` - The `SDL2` events, in order.
/// * `pointer` - Where the pointer is now, used as the drop position.
/// * `pending` - The files of a drop still in progress; kept between calls because a drop can
///   span two polls.
///
/// # Returns
///
/// The input events, with a `FilesDropped` at each `DropComplete`.
pub(crate) fn to_input_events(
    events: impl IntoIterator<Item = Event>,
    pointer: Point,
    pending: &mut Vec<PathBuf>,
) -> Vec<InputEvent> {
    let mut out = Vec::new();

    for event in events {
        match event {
            Event::DropBegin { .. } => pending.clear(),
            Event::DropFile { filename, .. } => pending.push(PathBuf::from(filename)),
            Event::DropComplete { .. } => {
                if !pending.is_empty() {
                    out.push(InputEvent::FilesDropped {
                        position: pointer,
                        paths: std::mem::take(pending),
                    });
                }
            }
            other => out.extend(to_input_event(other)),
        }
    }

    out
}

/// Works out the modifier keys held from the state of individual keys.
///
/// # Arguments
///
/// * `pressed` - Tells whether a key is held.
///
/// # Returns
///
/// The modifiers held (left and right keys are not distinguished).
pub(crate) fn scancode_modifiers(pressed: impl Fn(Scancode) -> bool) -> Modifiers {
    let any = |a: Scancode, b: Scancode| pressed(a) || pressed(b);
    let mut held = Modifiers::empty();

    held.set(Modifiers::SHIFT, any(Scancode::LShift, Scancode::RShift));
    held.set(Modifiers::CTRL, any(Scancode::LCtrl, Scancode::RCtrl));
    held.set(Modifiers::ALT, any(Scancode::LAlt, Scancode::RAlt));
    held.set(Modifiers::GUI, any(Scancode::LGui, Scancode::RGui));
    held
}

/// Puts an [`InputEvent::ModifiersChanged`] before each mouse press.
///
/// # Arguments
///
/// * `events` - The input events, in order.
/// * `held` - The modifier keys held.
///
/// # Returns
///
/// The events with the modifiers reported ahead of every `MouseDown`.
pub(crate) fn with_modifiers(events: Vec<InputEvent>, held: Modifiers) -> Vec<InputEvent> {
    let mut out = Vec::with_capacity(events.len() + 1);

    for event in events {
        if matches!(event, InputEvent::MouseDown { .. }) {
            out.push(InputEvent::ModifiersChanged(held));
        }

        out.push(event);
    }

    out
}

/// Returns the `SDL2` event types that feed only classes outside `mask`, and so can be disabled
/// in `SDL2` itself.
///
/// `Quit` and window events are never disabled: close requests are always delivered, and window
/// exposure and resizing keep the display up to date.
///
/// # Arguments
///
/// * `mask` - The run loop's event mask.
///
/// # Returns
///
/// The event types to disable.
pub(crate) fn types_to_disable(mask: EventMask) -> Vec<EventType> {
    let mut types = Vec::new();

    if !mask.contains(EventMask::POINTER_BUTTONS) {
        types.extend([EventType::MouseButtonDown, EventType::MouseButtonUp]);
    }

    // Hover is derived from motion, so motion is needed while either class is wanted.
    if !mask.intersects(EventMask::POINTER_MOTION | EventMask::HOVER) {
        types.push(EventType::MouseMotion);
    }

    if !mask.contains(EventMask::SCROLL) {
        types.push(EventType::MouseWheel);
    }

    if !mask.contains(EventMask::KEYBOARD) {
        types.extend([EventType::KeyDown, EventType::KeyUp]);
    }

    if !mask.contains(EventMask::TEXT) {
        types.extend([EventType::TextInput, EventType::TextEditing]);
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_modifiers_are_read_from_individual_keys() {
        let held = scancode_modifiers(|code| matches!(code, Scancode::RShift | Scancode::LGui));

        assert_eq!(held, Modifiers::SHIFT | Modifiers::GUI);
        assert_eq!(scancode_modifiers(|_| false), Modifiers::empty());
    }

    #[test]
    fn modifiers_are_reported_before_each_press() {
        let down = InputEvent::MouseDown {
            position: Point { x: 1, y: 2 },
            button: MouseButton::Left,
            clicks: 1,
        };
        let out = with_modifiers(vec![InputEvent::Exposed, down.clone()], Modifiers::CTRL);

        assert_eq!(
            out,
            vec![
                InputEvent::Exposed,
                InputEvent::ModifiersChanged(Modifiers::CTRL),
                down
            ]
        );
    }

    #[test]
    fn all_classes_disable_no_sdl_types() {
        assert!(types_to_disable(EventMask::ALL).is_empty());
    }

    #[test]
    fn hover_keeps_motion_enabled_but_neither_disables_it() {
        let without_motion = EventMask::ALL & !EventMask::POINTER_MOTION;
        let without_either = without_motion & !EventMask::HOVER;

        assert!(!types_to_disable(without_motion).contains(&EventType::MouseMotion));
        assert!(types_to_disable(without_either).contains(&EventType::MouseMotion));
    }

    #[test]
    fn an_empty_mask_disables_input_but_never_quit_or_window_events() {
        let types = types_to_disable(EventMask::NONE);

        for expected in [
            EventType::MouseButtonDown,
            EventType::MouseButtonUp,
            EventType::MouseWheel,
            EventType::KeyDown,
            EventType::KeyUp,
            EventType::TextInput,
        ] {
            assert!(types.contains(&expected));
        }

        assert!(!types.contains(&EventType::Quit));
        assert!(!types.contains(&EventType::Window));
    }

    #[test]
    fn dropped_files_arrive_as_one_event_at_the_pointer() {
        let mut pending = Vec::new();
        let events = vec![
            Event::DropBegin {
                timestamp: 0,
                window_id: 0,
            },
            Event::DropFile {
                timestamp: 0,
                window_id: 0,
                filename: "/tmp/a.txt".to_string(),
            },
            Event::DropFile {
                timestamp: 0,
                window_id: 0,
                filename: "/tmp/b.txt".to_string(),
            },
            Event::DropComplete {
                timestamp: 0,
                window_id: 0,
            },
        ];
        let out = to_input_events(events, Point { x: 5, y: 6 }, &mut pending);

        assert_eq!(
            out,
            vec![InputEvent::FilesDropped {
                position: Point { x: 5, y: 6 },
                paths: vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")],
            }]
        );
        assert!(pending.is_empty());
    }

    #[test]
    fn a_drop_split_across_polls_is_joined_and_empty_drops_are_ignored() {
        let mut pending = Vec::new();
        let file = Event::DropFile {
            timestamp: 0,
            window_id: 0,
            filename: "x".to_string(),
        };
        let complete = || Event::DropComplete {
            timestamp: 0,
            window_id: 0,
        };

        assert!(to_input_events([file], Point { x: 0, y: 0 }, &mut pending).is_empty());
        assert_eq!(
            to_input_events([complete()], Point { x: 0, y: 0 }, &mut pending).len(),
            1
        );
        assert!(to_input_events([complete()], Point { x: 0, y: 0 }, &mut pending).is_empty());
    }

    #[test]
    fn input_method_events_convert() {
        let editing = Event::TextEditing {
            timestamp: 0,
            window_id: 0,
            text: "にほん".to_string(),
            start: 2,
            length: 1,
        };
        let commit = Event::TextInput {
            timestamp: 0,
            window_id: 0,
            text: "日本".to_string(),
        };

        assert_eq!(
            to_input_event(editing),
            Some(InputEvent::Composition {
                text: "にほん".to_string(),
                start: 2,
                length: 1
            })
        );
        assert_eq!(
            to_input_event(commit),
            Some(InputEvent::Text("日本".to_string()))
        );
    }

    #[test]
    fn keys_map_to_named_and_printable_keys() {
        assert_eq!(to_key(Keycode::Tab), Key::Tab);
        assert_eq!(to_key(Keycode::KpEnter), Key::KpEnter);
        assert_eq!(to_key(Keycode::A), Key::Char('a'));
        assert_eq!(to_key(Keycode::Period), Key::Char('.'));
        assert_eq!(to_key(Keycode::F5), Key::F(5));
    }

    #[test]
    fn modifiers_merge_left_and_right() {
        assert_eq!(to_modifiers(Mod::NOMOD), Modifiers::empty());
        assert_eq!(
            to_modifiers(Mod::RSHIFTMOD | Mod::LGUIMOD),
            Modifiers::SHIFT | Modifiers::GUI
        );
    }

    #[test]
    fn mouse_and_window_events_convert() {
        let down = Event::MouseButtonDown {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mouse_btn: SdlButton::Left,
            clicks: 2,
            x: 5,
            y: 6,
        };

        assert_eq!(
            to_input_event(down),
            Some(InputEvent::MouseDown {
                position: Point::new(5, 6),
                button: MouseButton::Left,
                clicks: 2
            })
        );
        assert_eq!(
            to_input_event(Event::Quit { timestamp: 0 }),
            Some(InputEvent::Quit)
        );
        assert_eq!(
            to_input_event(Event::Window {
                timestamp: 0,
                window_id: 0,
                win_event: WindowEvent::SizeChanged(640, 480)
            }),
            Some(InputEvent::Resized(Size::new(640, 480)))
        );
    }
}
