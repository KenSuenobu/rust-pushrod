// Event
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

//! # Events
//!
//! Events are objects that say what happened, in plain terms, with named fields.  They cover
//! the pointer, keyboard, focus, widget semantics, lifecycle, window, time, the engine, and
//! application-defined (custom) events.
//!
//! Events flow through Pushrod much like the event buffer in GEM on the Atari ST:
//!
//! ```text
//!  SDL2 Event ──> WidgetCache ──> Widget::handle_event(&mut self, event, ctx)
//!                                          │
//!                                    ctx.emit(Clicked { .. })
//!                                          │
//!                                          v
//!                                   event queue (Vec)
//!                                          │
//!            (after the Widget is no longer borrowed by the engine)
//!                                          v
//!                        Engine callbacks: on_click / on_event
//!                          FnMut(&mut AppState, &mut Ui)
//! ```
//!
//! Queueing the generated events, instead of calling application code from inside the `Widget`,
//! is what allows callbacks to receive mutable access to both the application state and every
//! `Widget` in the window without fighting the borrow checker.
//!
//! Every [`Event`] has a [`kind`](Event::kind) (used by masks and the callback registry) and an
//! optional [`widget`](Event::widget) it concerns.
//!
//! ## Event reference
//!
//! | Group | Event | Emitted when |
//! | --- | --- | --- |
//! | Pointer | `PointerEntered` | The pointer moves onto a widget. |
//! | Pointer | `PointerExited` | The pointer moves off a widget. |
//! | Pointer | `PointerMoved` | The pointer moves; sent to the capturing widget, or the one under the pointer. |
//! | Pointer | `PointerPressed` | A mouse button goes down over a widget. |
//! | Pointer | `PointerReleased` | A mouse button goes up; sent to the widget that captured the press. |
//! | Pointer | `Clicked` | A widget (such as a button) was pressed and released on itself. |
//! | Pointer | `Scrolled` | The wheel or trackpad scrolls over a widget. |
//! | Keyboard | `KeyPressed` | A key goes down (or repeats) while a widget has the keyboard. |
//! | Keyboard | `KeyReleased` | A key goes up. |
//! | Keyboard | `TextEntered` | Text is typed (after input-method composition). |
//! | Focus | `FocusGained` | A widget becomes the keyboard focus. |
//! | Focus | `FocusLost` | A widget stops being the keyboard focus. |
//! | Semantic | `Activated` | A widget is activated (button, menu item, Return on a default action). |
//! | Semantic | `Toggled` | A checkable widget changes state; `checked` is the new state. |
//! | Semantic | `ValueChanged` | A slider, spinner, or similar changes value. |
//! | Semantic | `SelectionChanged` | The selection in a list, table, or tab set changes. |
//! | Semantic | `TextChanged` | The text of an editor changes. |
//! | Semantic | `EditingFinished` | An editor loses focus or the user confirms its text. |
//! | Semantic | `Triggered` | A named action is triggered (menu, shortcut, toolbar). |
//! | Semantic | `ContextMenuRequested` | The user asks for a context menu at `position`. |
//! | Semantic | `PropertyChanged` | A named property of a widget changes. |
//! | Semantic | `ViewportChanged` | A scrolling widget's visible area moves to `offset`. |
//! | Lifecycle | `Shown` / `Hidden` | A widget becomes visible / hidden. |
//! | Lifecycle | `Resized` / `Moved` | A widget's size / origin changes. |
//! | Lifecycle | `Removed` | A widget is removed from the tree. |
//! | Window | `WindowResized` | The window size changes. |
//! | Window | `WindowFocusChanged` | The window gains or loses focus. |
//! | Window | `CloseRequested` | The user asks to close the window. |
//! | Time | `Tick` | A frame passes while a widget animates; `elapsed` is the time since the last tick. |
//! | Engine | `Engine` | Pushrod performs an internal action ([`EngineEvent`]). |
//! | Extension | `Custom` | An application or library defines its own event ([`CustomEvent`]). |
//! | Extension | `Raw` | An SDL2 event Pushrod has not mapped yet. |
//!
//! `BoundsChange` from the first release is now `PointerExited` followed by `PointerEntered`,
//! and `SystemEvent` is split into `PointerPressed`, `PointerReleased`, and `PointerMoved`
//! (or `Raw` for an unmapped SDL2 event).

use crate::geometry::{Point, Size};
use crate::widget::WidgetId;
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use std::time::Duration;

/// An application- or library-defined event, carried by [`Event::Custom`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomEvent {
    /// The name that identifies the kind of custom event.
    pub name: String,
}

/// An action Pushrod performs internally, carried by [`Event::Engine`].
///
/// No actions are defined yet; they are added as the engine gains them.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineEvent {}

/// The kind of an [`Event`], without its data.  Used by masks and the callback registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// See [`Event::PointerEntered`].
    PointerEntered,
    /// See [`Event::PointerExited`].
    PointerExited,
    /// See [`Event::PointerMoved`].
    PointerMoved,
    /// See [`Event::PointerPressed`].
    PointerPressed,
    /// See [`Event::PointerReleased`].
    PointerReleased,
    /// See [`Event::Clicked`].
    Clicked,
    /// See [`Event::Scrolled`].
    Scrolled,
    /// See [`Event::KeyPressed`].
    KeyPressed,
    /// See [`Event::KeyReleased`].
    KeyReleased,
    /// See [`Event::TextEntered`].
    TextEntered,
    /// See [`Event::FocusGained`].
    FocusGained,
    /// See [`Event::FocusLost`].
    FocusLost,
    /// See [`Event::Activated`].
    Activated,
    /// See [`Event::Toggled`].
    Toggled,
    /// See [`Event::ValueChanged`].
    ValueChanged,
    /// See [`Event::SelectionChanged`].
    SelectionChanged,
    /// See [`Event::TextChanged`].
    TextChanged,
    /// See [`Event::EditingFinished`].
    EditingFinished,
    /// See [`Event::Triggered`].
    Triggered,
    /// See [`Event::ContextMenuRequested`].
    ContextMenuRequested,
    /// See [`Event::PropertyChanged`].
    PropertyChanged,
    /// See [`Event::ViewportChanged`].
    ViewportChanged,
    /// See [`Event::Shown`].
    Shown,
    /// See [`Event::Hidden`].
    Hidden,
    /// See [`Event::Resized`].
    Resized,
    /// See [`Event::Moved`].
    Moved,
    /// See [`Event::Removed`].
    Removed,
    /// See [`Event::WindowResized`].
    WindowResized,
    /// See [`Event::WindowFocusChanged`].
    WindowFocusChanged,
    /// See [`Event::CloseRequested`].
    CloseRequested,
    /// See [`Event::Tick`].
    Tick,
    /// See [`Event::Engine`].
    Engine,
    /// See [`Event::Custom`].
    Custom,
    /// See [`Event::Raw`].
    Raw,
}

/// An event generated by Pushrod or by a `Widget`.
///
/// `Widget`s receive these in [`Widget::handle_event`](crate::widget::Widget::handle_event), and
/// applications receive the ones `Widget`s emit through callbacks or
/// [`Engine::next_event`](crate::engine::Engine::next_event).  Positions are in window
/// coordinates.
#[derive(Debug, Clone)]
pub enum Event {
    // ---- Pointer ----
    /// The pointer moved onto widget `id`.
    PointerEntered {
        /// The widget the pointer entered.
        id: WidgetId,
    },
    /// The pointer moved off widget `id`.
    PointerExited {
        /// The widget the pointer left.
        id: WidgetId,
    },
    /// The pointer moved.  Sent to the capturing widget, or the one under the pointer.
    PointerMoved {
        /// The widget receiving the motion.
        id: WidgetId,
        /// The pointer position.
        position: Point,
    },
    /// A mouse button went down over widget `id`.
    PointerPressed {
        /// The widget under the pointer.
        id: WidgetId,
        /// The button that went down.
        button: MouseButton,
        /// The pointer position.
        position: Point,
        /// The click count (1 single, 2 double).
        count: u8,
    },
    /// A mouse button went up.  Sent to the widget that captured the press.
    PointerReleased {
        /// The widget receiving the release.
        id: WidgetId,
        /// The button that went up.
        button: MouseButton,
        /// The pointer position.
        position: Point,
        /// The click count (1 single, 2 double).
        count: u8,
    },
    /// Widget `id` was clicked.
    Clicked {
        /// The widget that was clicked.
        id: WidgetId,
        /// The button used.
        button: MouseButton,
        /// The click count (1 single, 2 double).
        count: u8,
    },
    /// The wheel or trackpad scrolled over widget `id`.
    Scrolled {
        /// The widget under the pointer.
        id: WidgetId,
        /// How far it scrolled, horizontally (`x`) and vertically (`y`).
        delta: Point,
    },

    // ---- Keyboard ----
    /// A key went down (or repeated) while widget `id` had the keyboard.
    KeyPressed {
        /// The widget with the keyboard.
        id: WidgetId,
        /// The key.
        key: Keycode,
        /// The modifier keys held.
        modifiers: Mod,
        /// `true` if this is an auto-repeat.
        repeat: bool,
    },
    /// A key went up while widget `id` had the keyboard.
    KeyReleased {
        /// The widget with the keyboard.
        id: WidgetId,
        /// The key.
        key: Keycode,
        /// The modifier keys held.
        modifiers: Mod,
    },
    /// Text was typed into widget `id`.
    TextEntered {
        /// The widget with the keyboard.
        id: WidgetId,
        /// The text typed.
        text: String,
    },

    // ---- Focus ----
    /// Widget `id` became the keyboard focus.
    FocusGained {
        /// The widget that gained focus.
        id: WidgetId,
    },
    /// Widget `id` stopped being the keyboard focus.
    FocusLost {
        /// The widget that lost focus.
        id: WidgetId,
    },

    // ---- Semantic (emitted by widgets) ----
    /// Widget `id` was activated.
    Activated {
        /// The widget activated.
        id: WidgetId,
    },
    /// Widget `id` was toggled.
    Toggled {
        /// The widget toggled.
        id: WidgetId,
        /// The new state.
        checked: bool,
    },
    /// The value of widget `id` changed.
    ValueChanged {
        /// The widget whose value changed.
        id: WidgetId,
    },
    /// The selection of widget `id` changed.
    SelectionChanged {
        /// The widget whose selection changed.
        id: WidgetId,
    },
    /// The text of widget `id` changed.
    TextChanged {
        /// The widget whose text changed.
        id: WidgetId,
    },
    /// Editing in widget `id` finished.
    EditingFinished {
        /// The widget that was edited.
        id: WidgetId,
    },
    /// A named action was triggered by widget `id`.
    Triggered {
        /// The widget that triggered it.
        id: WidgetId,
        /// The action name.
        action: String,
    },
    /// A context menu was requested at `position` over widget `id`.
    ContextMenuRequested {
        /// The widget under the pointer.
        id: WidgetId,
        /// Where the menu was requested.
        position: Point,
    },
    /// A property of widget `id` changed.
    PropertyChanged {
        /// The widget whose property changed.
        id: WidgetId,
        /// The property name.
        name: String,
    },
    /// The visible area of scrolling widget `id` moved.
    ViewportChanged {
        /// The scrolling widget.
        id: WidgetId,
        /// The new scroll offset.
        offset: Point,
    },

    // ---- Lifecycle ----
    /// Widget `id` became visible.
    Shown {
        /// The widget shown.
        id: WidgetId,
    },
    /// Widget `id` became hidden.
    Hidden {
        /// The widget hidden.
        id: WidgetId,
    },
    /// Widget `id` changed size.
    Resized {
        /// The widget resized.
        id: WidgetId,
        /// Its new size.
        size: Size,
    },
    /// Widget `id` changed origin.
    Moved {
        /// The widget moved.
        id: WidgetId,
        /// Its new origin.
        origin: Point,
    },
    /// Widget `id` was removed from the tree.
    Removed {
        /// The widget removed.
        id: WidgetId,
    },

    // ---- Window ----
    /// The window changed size.
    WindowResized {
        /// The new window size.
        size: Size,
    },
    /// The window gained or lost focus.
    WindowFocusChanged {
        /// `true` if the window now has focus.
        focused: bool,
    },
    /// The user asked to close the window.
    CloseRequested,

    // ---- Time ----
    /// A frame passed while a widget animates.
    Tick {
        /// The time since the previous tick.
        elapsed: Duration,
    },

    // ---- Engine / extension ----
    /// An action Pushrod performed internally.
    Engine(EngineEvent),
    /// An application- or library-defined event.
    Custom(CustomEvent),
    /// An SDL2 event Pushrod has not mapped yet.
    Raw(SdlEvent),
}

impl Event {
    /// Returns the kind of this event, without its data.
    ///
    /// # Returns
    ///
    /// The matching [`EventKind`].
    pub fn kind(&self) -> EventKind {
        match self {
            Event::PointerEntered { .. } => EventKind::PointerEntered,
            Event::PointerExited { .. } => EventKind::PointerExited,
            Event::PointerMoved { .. } => EventKind::PointerMoved,
            Event::PointerPressed { .. } => EventKind::PointerPressed,
            Event::PointerReleased { .. } => EventKind::PointerReleased,
            Event::Clicked { .. } => EventKind::Clicked,
            Event::Scrolled { .. } => EventKind::Scrolled,
            Event::KeyPressed { .. } => EventKind::KeyPressed,
            Event::KeyReleased { .. } => EventKind::KeyReleased,
            Event::TextEntered { .. } => EventKind::TextEntered,
            Event::FocusGained { .. } => EventKind::FocusGained,
            Event::FocusLost { .. } => EventKind::FocusLost,
            Event::Activated { .. } => EventKind::Activated,
            Event::Toggled { .. } => EventKind::Toggled,
            Event::ValueChanged { .. } => EventKind::ValueChanged,
            Event::SelectionChanged { .. } => EventKind::SelectionChanged,
            Event::TextChanged { .. } => EventKind::TextChanged,
            Event::EditingFinished { .. } => EventKind::EditingFinished,
            Event::Triggered { .. } => EventKind::Triggered,
            Event::ContextMenuRequested { .. } => EventKind::ContextMenuRequested,
            Event::PropertyChanged { .. } => EventKind::PropertyChanged,
            Event::ViewportChanged { .. } => EventKind::ViewportChanged,
            Event::Shown { .. } => EventKind::Shown,
            Event::Hidden { .. } => EventKind::Hidden,
            Event::Resized { .. } => EventKind::Resized,
            Event::Moved { .. } => EventKind::Moved,
            Event::Removed { .. } => EventKind::Removed,
            Event::WindowResized { .. } => EventKind::WindowResized,
            Event::WindowFocusChanged { .. } => EventKind::WindowFocusChanged,
            Event::CloseRequested => EventKind::CloseRequested,
            Event::Tick { .. } => EventKind::Tick,
            Event::Engine(_) => EventKind::Engine,
            Event::Custom(_) => EventKind::Custom,
            Event::Raw(_) => EventKind::Raw,
        }
    }

    /// Returns the widget this event concerns, if any.
    ///
    /// Window, time, engine, custom, and raw events concern no single widget.
    ///
    /// # Returns
    ///
    /// The `WidgetId`, or `None`.
    pub fn widget(&self) -> Option<WidgetId> {
        match self {
            Event::PointerEntered { id }
            | Event::PointerExited { id }
            | Event::PointerMoved { id, .. }
            | Event::PointerPressed { id, .. }
            | Event::PointerReleased { id, .. }
            | Event::Clicked { id, .. }
            | Event::Scrolled { id, .. }
            | Event::KeyPressed { id, .. }
            | Event::KeyReleased { id, .. }
            | Event::TextEntered { id, .. }
            | Event::FocusGained { id }
            | Event::FocusLost { id }
            | Event::Activated { id }
            | Event::Toggled { id, .. }
            | Event::ValueChanged { id }
            | Event::SelectionChanged { id }
            | Event::TextChanged { id }
            | Event::EditingFinished { id }
            | Event::Triggered { id, .. }
            | Event::ContextMenuRequested { id, .. }
            | Event::PropertyChanged { id, .. }
            | Event::ViewportChanged { id, .. }
            | Event::Shown { id }
            | Event::Hidden { id }
            | Event::Resized { id, .. }
            | Event::Moved { id, .. }
            | Event::Removed { id } => Some(*id),
            Event::WindowResized { .. }
            | Event::WindowFocusChanged { .. }
            | Event::CloseRequested
            | Event::Tick { .. }
            | Event::Engine(_)
            | Event::Custom(_)
            | Event::Raw(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::point;

    #[test]
    fn kind_matches_variant() {
        assert_eq!(
            Event::PointerEntered { id: 1 }.kind(),
            EventKind::PointerEntered
        );
        assert_eq!(Event::CloseRequested.kind(), EventKind::CloseRequested);
        assert_eq!(
            Event::Custom(CustomEvent { name: "x".into() }).kind(),
            EventKind::Custom
        );
        assert_eq!(
            Event::Tick {
                elapsed: Duration::from_millis(16)
            }
            .kind(),
            EventKind::Tick
        );
    }

    #[test]
    fn widget_returns_the_target_or_none() {
        let clicked = Event::Clicked {
            id: 7,
            button: MouseButton::Left,
            count: 2,
        };
        let moved = Event::PointerMoved {
            id: 3,
            position: point(1, 2),
        };

        assert_eq!(clicked.widget(), Some(7));
        assert_eq!(moved.widget(), Some(3));
        assert_eq!(Event::CloseRequested.widget(), None);
        assert_eq!(Event::WindowFocusChanged { focused: true }.widget(), None);
        assert_eq!(
            Event::Custom(CustomEvent { name: "x".into() }).widget(),
            None
        );
    }

    #[test]
    fn ctx_returns_events_in_order() {
        let mut ctx = EventCtx::default();

        ctx.emit(Event::FocusLost { id: 1 });
        ctx.emit(Event::FocusGained { id: 2 });

        let kinds: Vec<_> = ctx.into_events().iter().map(Event::kind).collect();

        assert_eq!(kinds, vec![EventKind::FocusLost, EventKind::FocusGained]);
    }
}

/// The context passed to [`Widget::handle_event`](crate::widget::Widget::handle_event).
///
/// `Widget`s use it to queue the events they generate.  Queueing, instead of calling application
/// code from inside the `Widget`, lets callbacks later receive mutable access to both the
/// application state and every `Widget`.
#[derive(Default)]
pub struct EventCtx {
    events: Vec<Event>,
}

impl EventCtx {
    /// Queues an event generated by a `Widget`.
    ///
    /// Queued events are delivered to the application in the order in which they were emitted.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to queue, such as `Event::Clicked { id, button, count }`.
    pub fn emit(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Consumes the context and returns the queued events.
    ///
    /// # Returns
    ///
    /// The queued events, in the order emitted.
    pub fn into_events(self) -> Vec<Event> {
        self.events
    }
}
