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

use sdl2::event::Event;

#[derive(Debug)]
pub enum PushrodEvent {
    /// Indicates a mouse entered the bounds of an object.  The ID of the object is returned.
    EnteredBounds(u32),

    /// Indicates a mouse exited the bounds of an object.  The ID of the object is returned.
    ExitedBounds(u32),

    /// Indicates an object has been toggled.  The ID of the object is returned along with its
    /// selection state.
    Toggled(u32, bool),

    /// Indicates an object was clicked.  The ID of the object is returned along with the number
    /// of clicks within the object.
    Clicked(u32, u32),

    /// Indicates an SDL-based Event occurred.
    SystemEvent(Event),
}

/// The `EventHandler` is a class used to handle events generate from the `Engine::run` loop.
/// These are `PushrodEvent` objects, which are events generated from `Widget`s that intercept
/// normal events from `SDL2`.
pub trait EventHandler {
    /// Handles processing of events.  The `events` passed in are references to `PushrodEvent` objects.
    /// This function will only be called if the event list is non-empty.
    fn process_event(&self, events: Vec<&PushrodEvent>);
}
