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

//! # pushrod-widgets
//!
//! The standard widgets for [Pushrod](https://docs.rs/pushrod): simple, flat, and predictable.
//! Every widget here is drawn with the core's `paint` helpers (fills, one-pixel borders, and
//! text), following Pushrod's design philosophy: no glass, no bevels, no gradients, no shadows,
//! no rounded corners.
//!
//! This crate uses only the public API of the `pushrod` core, so another widget library can do
//! everything it does: extend it with new widgets, or replace it with a more graphically
//! ambitious set, without changing the core.
//!
//! ```rust,ignore
//! use pushrod::prelude::*;
//! use pushrod_widgets::prelude::*;
//!
//! #[derive(Default)]
//! struct AppState {
//!     clicks: u32,
//! }
//!
//! let mut engine: Engine<AppState> = Engine::new(size(400, 200), 30);
//!
//! let label = engine.add_widget(Text::new("Not clicked").at(20, 20).size(200, 20));
//! let button = engine.add_widget(Button::new("Click me").at(20, 60).size(120, 24));
//!
//! engine.on_click(button, move |state, ui| {
//!     state.clicks += 1;
//!     ui.get_mut::<Text>(label)
//!         .unwrap()
//!         .set_text(format!("Clicked {} times", state.clicks));
//! });
//!
//! pushrod_sdl2::run(engine, pushrod_sdl2::WindowSettings::new("app", size(400, 200)), AppState::default());
//! ```

// Every public item is documented, including `# Panics` and `# Errors` sections where they
// apply.  scripts/check_docs.py checks the rest (see CONTRIBUTING.md, "Code documentation").
#![warn(
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

pub mod boxed_label;
pub mod button;
pub mod check_box;
pub mod custom_paint;
pub mod frame;
pub mod glyph_button;
pub mod group_box;
pub mod image_widget;
pub mod panel;
pub mod prelude;
pub mod progress_bar;
pub mod radio_button;
pub mod scroll_bar;
pub mod separator;
pub mod slider;
pub mod text;
pub mod text_field;
