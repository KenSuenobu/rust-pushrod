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

#[derive(Debug, Clone)]
pub enum PushrodEvent {
    /// Indicates a change in object bounds, exiting of one bound and entering another.  First
    /// argument is the ID of the `Widget` that lost bounds, second argument is the ID of
    /// the `Widget` that gained bounds.
    BoundsChange(u32, u32),

    /// Indicates an SDL-based Event occurred.
    SystemEvent(Event),
}

pub trait EventHandler {
    fn process_event(&self, events: Vec<&PushrodEvent>);
}
