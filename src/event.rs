// Pushrod Events
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

//! # PushrodEvents
//!
//! Events are objects that contain messages that result from a user or system interaction.
//! They contain different types of messages, such as mouse clicks, coordinates, keyboard
//! presses, and so on.
//!
//! `PushrodEvents` are structured objects that contain these messages wrapped in a `Struct`,
//! which can be interpreted by other `Widget`s in the `Pushrod` library.

use sdl2::event::Event;

/// These are different types of events that the `Pushrod` library will generate.  Any custom
/// events should be added here.
#[derive(Debug, Clone)]
pub enum PushrodEvent {
    /// Indicates a change in object bounds, exiting of one bound and entering another.  First
    /// argument is the ID of the `Widget` that lost bounds, second argument is the ID of
    /// the `Widget` that gained bounds.
    BoundsChange(u32, u32),

    /// Indicates a `Widget` detected a click event inside its bounds.  The first argument is
    /// the ID of the `Widget` that was clicked, second argument is the number of clicks that
    /// the `Widget` received.
    Clicked(u32, u8),

    /// Indicates an SDL-based Event occurred.  The first argument is the ID of the widget that
    /// was found, and the second is the event that occurred.
    SystemEvent(u32, Event),
}

/// This is a trait that indicates an impl can process events.
pub trait EventHandler {
    /// Processes a list of events.
    fn process_event(&self, events: Vec<&PushrodEvent>);
}
