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

//! # rust-pushrod
//!
//! Pushrod is a library written entirely in Rust for drawing interactive elements to a window.
//!
//! It is an event-based GUI library that utilizes an event loop to handle the event, dispatch, and
//! draw loop.
//!
//! The documentation site, <https://pushrod.nobudata.com>, has step-by-step guides for installing
//! Pushrod, writing applications, and writing widgets, with many examples.
//!
//! # Core, widgets, and backends
//!
//! This crate is the core: the engine, the `Widget` trait and widget tree, events, input types,
//! the [`Backend`](backend::Backend) trait, the backend-neutral `Painter` API and `paint`
//! helpers.  It has no windowing-library dependency.  It contains no widgets except the
//! window's [`RootWidget`](root_widget::RootWidget).  The standard widgets (buttons, text, boxes,
//! and the rest) live in the `pushrod-widgets` crate, which depends on this crate only.
//!
//! An application picks a backend crate.  `pushrod-sdl2` draws with `SDL2`:
//!
//! ```ignore
//! pushrod_sdl2::run(engine, pushrod_sdl2::WindowSettings::new("App", size(400, 300)), state);
//! ```
//!
//! For tests, [`HeadlessBackend`](backend::headless::HeadlessBackend) needs no display.
//!
//! # Writing widgets
//!
//! A widget stores a [`WidgetCore`](widget::WidgetCore) and implements
//! [`Widget`](widget::Widget); a widget built from other widgets also implements
//! [`Composite`](composite::Composite).  The guide
//! [Writing widgets and composites](https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/WRITING_WIDGETS.md)
//! covers painting, events and callbacks, behavior and state, input controllers, composites,
//! custom events, and testing with [`Engine::dispatch`](engine::Engine::dispatch), with a
//! compiled sample for each.
//!
//! # Specifications
//!
//! The contracts a widget library or backend must keep are written in
//! [`docs/specs`](https://github.com/KenSuenobu/rust-pushrod/tree/main/docs/specs): module
//! boundaries, widgets, events, painting, and forms.  Each guarantee is checked by a conformance
//! test in `tests/conformance/`, which uses only this crate's public API.

// Every public item is documented, including `# Panics` and `# Errors` sections where they
// apply.  scripts/check_docs.py checks the rest (see CONTRIBUTING.md, "Code documentation").
#![warn(
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
// Library code returns `Error` instead of panicking; tests may unwrap.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

pub mod accessibility;
pub mod action;
pub mod appearance;
pub mod backend;
pub mod cache;
pub mod color;
pub mod composite;
pub mod controllers;
pub mod dnd;
pub mod engine;
pub mod error;
pub mod event;
pub mod font;
pub mod form;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod paint;
pub mod prelude;
pub mod properties;
pub mod registry;
pub mod resource;
pub mod root_widget;
pub mod text;
pub mod timer;
pub mod undo;
pub mod widget;
