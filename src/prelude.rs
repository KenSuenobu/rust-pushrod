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

//! What an application normally imports from the core: `use pushrod::prelude::*;`.
//!
//! The prelude holds the engine, the event and input types, geometry helpers, colors, fonts, the
//! `Widget` trait and its supporting types, and the core error type.  Less common items (the
//! `Painter` trait, controllers, timers, backends) stay in their own modules.
//!
//! ```rust
//! use pushrod::prelude::*;
//!
//! let engine: Engine = Engine::new(size(400, 300), 30);
//! let _ = (engine, point(0, 0), Color::BLACK);
//! ```

pub use crate::color::{Color, FontStyle};
pub use crate::composite::{Composite, PartList, PartName, PartsBuilder, TypedParts};
pub use crate::engine::{Engine, Ui};
pub use crate::error::Error;
pub use crate::event::{Event, EventMask};
pub use crate::font::Font;
pub use crate::geometry::{Point, Rect, Size, point, rect, size};
pub use crate::input::{InputEvent, Key, MouseButton};
pub use crate::paint::{TextJustify, Truncate, Wrap};
pub use crate::widget::{Behavior, VisualState, Widget, WidgetCore, WidgetId};
