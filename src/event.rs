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
use std::any::Any;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

/// Where a [`CustomEvent`] is delivered.
///
/// Whatever the target, the application's `on_custom` and `on_event` callbacks always receive the
/// event; the target only chooses which `Widget`s see it in `handle_event`.  A `Widget` receives it
/// only if [`EventMask::CUSTOM`] is in its mask (and in the run loop's mask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTarget {
    /// One `Widget`.
    Widget(WidgetId),
    /// A `Widget` and all of its descendants, parents before children.
    Subtree(WidgetId),
    /// Every visible `Widget`, in drawing order.
    Broadcast,
    /// No `Widget`: only the application's callbacks receive it.
    ApplicationOnly,
}

/// An application- or library-defined event, carried by [`Event::Custom`].
///
/// The payload is any `'static` value; receivers read it back with [`CustomEvent::payload`], which
/// returns `None` for any other type.  Pushrod never looks inside a custom event, and it has no
/// side effects on focus, mouse capture, selection, or redrawing.
///
/// # Examples
///
/// ```rust
/// use pushrod::event::{CustomEvent, EventTarget};
/// use std::rc::Rc;
///
/// struct Saved(u32);
///
/// let event = CustomEvent {
///     source: None,
///     target: EventTarget::Broadcast,
///     payload: Rc::new(Saved(3)),
/// };
///
/// assert_eq!(event.payload::<Saved>().unwrap().0, 3);
/// assert!(event.payload::<String>().is_none());
/// ```
#[derive(Clone)]
pub struct CustomEvent {
    /// The `Widget` that posted the event, or `None` if the application or another thread did.
    pub source: Option<WidgetId>,
    /// Which `Widget`s receive the event.
    pub target: EventTarget,
    /// The application's data.
    pub payload: Rc<dyn Any>,
}

impl CustomEvent {
    /// Reads the payload as a `T`.
    ///
    /// # Returns
    ///
    /// The payload, or `None` if it is not a `T`.
    pub fn payload<T: Any>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }
}

impl fmt::Debug for CustomEvent {
    /// Formats the source and target; the payload is opaque, so it is not shown.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomEvent")
            .field("source", &self.source)
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

/// A set of event classes, used to filter events for the run loop and for each `Widget`.
///
/// An event is handled only when its class is in both the run loop's mask
/// ([`Engine::set_event_mask`](crate::engine::Engine::set_event_mask)) and the receiving
/// `Widget`'s mask ([`Widget::event_mask`](crate::widget::Widget::event_mask)).  Combine classes
/// with `|`.
///
/// # Examples
///
/// ```rust
/// use pushrod::event::EventMask;
///
/// let mask = EventMask::POINTER_BUTTONS | EventMask::KEYBOARD;
///
/// assert!(mask.contains(EventMask::KEYBOARD));
/// assert!(!mask.contains(EventMask::SCROLL));
/// assert!(EventMask::ALL.contains(mask));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EventMask(u16);

impl EventMask {
    /// No classes: nothing is delivered.
    pub const NONE: EventMask = EventMask(0);
    /// Mouse button presses and releases ([`Event::PointerPressed`], [`Event::PointerReleased`]).
    pub const POINTER_BUTTONS: EventMask = EventMask(1);
    /// Pointer movement ([`Event::PointerMoved`]).
    pub const POINTER_MOTION: EventMask = EventMask(1 << 1);
    /// The pointer entering or leaving a `Widget` ([`Event::PointerEntered`], [`Event::PointerExited`]).
    pub const HOVER: EventMask = EventMask(1 << 2);
    /// Scroll wheel and trackpad scrolling ([`Event::Scrolled`]).
    pub const SCROLL: EventMask = EventMask(1 << 3);
    /// Key presses and releases ([`Event::KeyPressed`], [`Event::KeyReleased`]).
    pub const KEYBOARD: EventMask = EventMask(1 << 4);
    /// Text input ([`Event::TextEntered`]).
    pub const TEXT: EventMask = EventMask(1 << 5);
    /// Keyboard focus changes ([`Event::FocusGained`], [`Event::FocusLost`]).
    pub const FOCUS: EventMask = EventMask(1 << 6);
    /// Window events ([`Event::WindowResized`], [`Event::WindowFocusChanged`]).
    pub const WINDOW: EventMask = EventMask(1 << 7);
    /// `Widget` lifecycle and layout changes ([`Event::Shown`], [`Event::Resized`], [`Event::Removed`], ...).
    pub const LIFECYCLE: EventMask = EventMask(1 << 8);
    /// Timer ticks ([`Event::Tick`]).  Without it the run loop blocks while idle.
    pub const TIMER: EventMask = EventMask(1 << 9);
    /// Custom and unmapped events ([`Event::Custom`], [`Event::Raw`]).
    pub const CUSTOM: EventMask = EventMask(1 << 10);
    /// Engine events ([`Event::Engine`]).  A `Widget` receives them only with this class in its mask.
    pub const ENGINE: EventMask = EventMask(1 << 11);
    /// Every class.
    pub const ALL: EventMask = EventMask((1 << 12) - 1);

    /// Returns whether every class in `other` is in this mask.
    ///
    /// # Arguments
    ///
    /// * `other` - The classes to look for.
    ///
    /// # Returns
    ///
    /// `true` if all of `other`'s classes are present.
    pub fn contains(self, other: EventMask) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns whether this mask and `other` share at least one class.
    ///
    /// # Arguments
    ///
    /// * `other` - The classes to compare with.
    ///
    /// # Returns
    ///
    /// `true` if any class is in both.
    pub fn intersects(self, other: EventMask) -> bool {
        self.0 & other.0 != 0
    }

    /// Returns whether the mask holds no classes.
    ///
    /// # Returns
    ///
    /// `true` for [`EventMask::NONE`].
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for EventMask {
    type Output = EventMask;

    /// Returns the union of two masks.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The classes to add.
    ///
    /// # Returns
    ///
    /// A mask with the classes of both.
    fn bitor(self, rhs: EventMask) -> EventMask {
        EventMask(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for EventMask {
    type Output = EventMask;

    /// Returns the classes present in both masks.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The classes to keep.
    ///
    /// # Returns
    ///
    /// A mask with only the classes in both.
    fn bitand(self, rhs: EventMask) -> EventMask {
        EventMask(self.0 & rhs.0)
    }
}

impl std::ops::Not for EventMask {
    type Output = EventMask;

    /// Returns every class not in the mask.
    ///
    /// # Returns
    ///
    /// The complement of the mask within [`EventMask::ALL`].
    fn not(self) -> EventMask {
        EventMask(!self.0 & EventMask::ALL.0)
    }
}

/// What an event mask belongs to, in [`EngineEvent::EventMaskChanged`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskScope {
    /// The run loop's mask.
    RunLoop,
    /// The mask of one `Widget`.
    Widget(WidgetId),
}

/// The kind of an [`EngineEvent`], without its data.  Used to filter which engine events a
/// `Widget` or callback receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EngineEventKind {
    /// See [`EngineEvent::EventMaskChanged`].
    EventMaskChanged,
    /// See [`EngineEvent::WidgetAdded`].
    WidgetAdded,
    /// See [`EngineEvent::EventBroadcast`].
    EventBroadcast,
    /// See [`EngineEvent::CustomEventDropped`].
    CustomEventDropped,
}

/// An action Pushrod performs internally, carried by [`Event::Engine`].
///
/// Engine events are broadcast after the action has completed, to `Widget`s whose mask includes
/// [`EventMask::ENGINE`], then to `Engine::on_engine_event` callbacks and `Engine::on_event`
/// callbacks.  They never cause further internal actions by themselves, so they cannot loop.
///
/// **Rule for contributors:** any change that adds an internal action must add a variant here and
/// emit it.  More variants are added as the engine gains the actions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineEvent {
    /// A `Widget` was added to the window.  Not emitted for the root `Widget`.
    WidgetAdded {
        /// The ID of the new `Widget`.
        id: WidgetId,
        /// The ID of its parent.
        parent: WidgetId,
    },
    /// An event mask changed.
    EventMaskChanged {
        /// Whose mask changed.
        scope: MaskScope,
        /// The mask before the change.
        old: EventMask,
        /// The mask after the change.
        new: EventMask,
    },
    /// An event was broadcast to every interested `Widget`.
    EventBroadcast {
        /// The kind of the broadcast event.
        kind: EventKind,
        /// How many `Widget`s received it.
        recipients: usize,
    },
    /// A custom event was discarded because the per-frame limit on custom events was reached,
    /// which stops callbacks that re-post forever.
    CustomEventDropped {
        /// The target of the discarded event.
        target: EventTarget,
    },
}

impl EngineEvent {
    /// Returns the kind of this engine event, for filtering.
    ///
    /// # Returns
    ///
    /// The [`EngineEventKind`] matching the variant.
    pub fn kind(&self) -> EngineEventKind {
        match self {
            EngineEvent::EventMaskChanged { .. } => EngineEventKind::EventMaskChanged,
            EngineEvent::WidgetAdded { .. } => EngineEventKind::WidgetAdded,
            EngineEvent::EventBroadcast { .. } => EngineEventKind::EventBroadcast,
            EngineEvent::CustomEventDropped { .. } => EngineEventKind::CustomEventDropped,
        }
    }
}

/// Returns whether an engine event passes an optional kind filter.
///
/// # Arguments
///
/// * `filter` - The accepted kinds, or `None` to accept every kind.
/// * `event` - The engine event.
///
/// # Returns
///
/// `true` if there is no filter or the filter lists the event's kind.
pub fn engine_filter_accepts(filter: Option<&[EngineEventKind]>, event: &EngineEvent) -> bool {
    filter.is_none_or(|kinds| kinds.contains(&event.kind()))
}

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

impl EventKind {
    /// Returns the mask class this kind of event is filtered by.
    ///
    /// # Returns
    ///
    /// The class, or `None` when the event is never filtered: close requests, engine events, and
    /// the events `Widget`s generate for the application (`Clicked`, `Activated`, ...).
    pub fn mask_class(self) -> Option<EventMask> {
        use EventKind as K;

        match self {
            K::PointerPressed | K::PointerReleased => Some(EventMask::POINTER_BUTTONS),
            K::PointerMoved => Some(EventMask::POINTER_MOTION),
            K::PointerEntered | K::PointerExited => Some(EventMask::HOVER),
            K::Scrolled => Some(EventMask::SCROLL),
            K::KeyPressed | K::KeyReleased => Some(EventMask::KEYBOARD),
            K::TextEntered => Some(EventMask::TEXT),
            K::FocusGained | K::FocusLost => Some(EventMask::FOCUS),
            K::WindowResized | K::WindowFocusChanged => Some(EventMask::WINDOW),
            K::PropertyChanged
            | K::ViewportChanged
            | K::Shown
            | K::Hidden
            | K::Resized
            | K::Moved
            | K::Removed => Some(EventMask::LIFECYCLE),
            K::Tick => Some(EventMask::TIMER),
            K::Custom | K::Raw => Some(EventMask::CUSTOM),
            K::Clicked
            | K::Activated
            | K::Toggled
            | K::ValueChanged
            | K::SelectionChanged
            | K::TextChanged
            | K::EditingFinished
            | K::Triggered
            | K::ContextMenuRequested
            | K::CloseRequested
            | K::Engine => None,
        }
    }
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
    /// Reads the payload of a custom event as a `T`.
    ///
    /// # Returns
    ///
    /// The payload, or `None` if this is not [`Event::Custom`] or the payload is not a `T`.
    pub fn payload<T: Any>(&self) -> Option<&T> {
        match self {
            Event::Custom(custom) => custom.payload::<T>(),
            _ => None,
        }
    }

    /// The widget a widget-addressed event is for.
    ///
    /// # Returns
    ///
    /// The `WidgetId`, or `None` for window-level, engine, custom and raw events.
    pub fn widget_id(&self) -> Option<WidgetId> {
        match self {
            Event::PointerEntered { id }
            | Event::PointerExited { id }
            | Event::PointerMoved { id, .. }
            | Event::PointerPressed { id, .. }
            | Event::PointerReleased { id, .. }
            | Event::Clicked { id, .. }
            | Event::KeyPressed { id, .. }
            | Event::FocusGained { id }
            | Event::FocusLost { id }
            | Event::Activated { id }
            | Event::Toggled { id, .. }
            | Event::ValueChanged { id }
            | Event::SelectionChanged { id }
            | Event::TextChanged { id }
            | Event::EditingFinished { id }
            | Event::ViewportChanged { id, .. } => Some(*id),
            _ => None,
        }
    }

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

    fn sample_custom() -> CustomEvent {
        CustomEvent {
            source: None,
            target: EventTarget::ApplicationOnly,
            payload: Rc::new(7_u32),
        }
    }

    #[test]
    fn kind_matches_variant() {
        assert_eq!(
            Event::PointerEntered { id: 1 }.kind(),
            EventKind::PointerEntered
        );
        assert_eq!(Event::CloseRequested.kind(), EventKind::CloseRequested);
        assert_eq!(Event::Custom(sample_custom()).kind(), EventKind::Custom);
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
        assert_eq!(Event::Custom(sample_custom()).widget(), None);
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
    source: Option<WidgetId>,
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

    /// Posts a custom event, with the `Widget` currently handling an event as its source.
    ///
    /// The event is queued behind the events already waiting and delivered later, in order, never
    /// from inside this call.  See [`CustomEvent`].
    ///
    /// # Arguments
    ///
    /// * `target` - Which `Widget`s receive the event.
    /// * `payload` - The application's data; receivers read it with [`Event::payload`].
    pub fn post<T: Any>(&mut self, target: EventTarget, payload: T) {
        self.events.push(Event::Custom(CustomEvent {
            source: self.source,
            target,
            payload: Rc::new(payload),
        }));
    }

    /// Records which `Widget` is handling an event, so [`EventCtx::post`] can name it as the source.
    ///
    /// # Arguments
    ///
    /// * `source` - The `Widget` about to receive an event.
    pub(crate) fn set_source(&mut self, source: Option<WidgetId>) {
        self.source = source;
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

#[cfg(test)]
mod mask_tests {
    use super::*;

    #[test]
    fn masks_combine_and_compare() {
        let mask = EventMask::HOVER | EventMask::SCROLL;

        assert!(mask.contains(EventMask::HOVER));
        assert!(!mask.contains(EventMask::HOVER | EventMask::KEYBOARD));
        assert!(mask.intersects(EventMask::HOVER | EventMask::KEYBOARD));
        assert!((mask & EventMask::HOVER) == EventMask::HOVER);
        assert!(EventMask::NONE.is_empty());
        assert_eq!(!EventMask::NONE, EventMask::ALL);
        assert_eq!(!EventMask::ALL, EventMask::NONE);
        assert_eq!(EventMask::default(), EventMask::NONE);
    }

    #[test]
    fn input_kinds_have_a_class_and_outputs_and_close_have_none() {
        assert_eq!(
            EventKind::PointerMoved.mask_class(),
            Some(EventMask::POINTER_MOTION)
        );
        assert_eq!(EventKind::Tick.mask_class(), Some(EventMask::TIMER));
        assert_eq!(EventKind::Raw.mask_class(), Some(EventMask::CUSTOM));
        assert_eq!(EventKind::CloseRequested.mask_class(), None);
        assert_eq!(EventKind::Engine.mask_class(), None);
        assert_eq!(EventKind::Clicked.mask_class(), None);
    }

    #[test]
    fn engine_class_is_part_of_all() {
        assert!(EventMask::ALL.contains(EventMask::ENGINE));
        assert!(!(EventMask::ALL & !EventMask::ENGINE).intersects(EventMask::ENGINE));
    }

    #[test]
    fn engine_events_report_kind_and_filter() {
        let added = EngineEvent::WidgetAdded { id: 1, parent: 0 };

        assert_eq!(added.kind(), EngineEventKind::WidgetAdded);
        assert!(engine_filter_accepts(None, &added));
        assert!(engine_filter_accepts(
            Some(&[EngineEventKind::WidgetAdded]),
            &added
        ));
        assert!(!engine_filter_accepts(
            Some(&[EngineEventKind::EventMaskChanged]),
            &added
        ));
    }
}
