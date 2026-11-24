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
//! Pushrod is a library written entirely in Rust, utilizing the SDL2 later for drawing interactive
//! elements to the screen via OpenGL.
//!
//! It is an event-based GUI library that utilizes an event loop to handle the event, dispatch, and
//! draw loop.
//!
//! # Core and widgets
//!
//! This crate is the core: the engine, the `Widget` trait and widget tree, events, the `paint`
//! helpers, and texture and font caches.  It contains no widgets except the window's
//! [`RootWidget`](root_widget::RootWidget).  The standard widgets (buttons, text, boxes, and
//! the rest) live in the `pushrod-widgets` crate.  A different widget library can extend or
//! replace `pushrod-widgets`, using the same public API.
//!
//! # Dependencies
//!
//! Pushrod uses the following dependency:
//! ```ignore
//! [dependencies.sdl2]
//! version = "0.38"
//! features = ["ttf", "image", "unsafe_textures"]
//! ```
//!
//! Note, the `unsafe_textures` feature is required for OpenGL functionality.

// Every public item is documented, including `# Panics` and `# Errors` sections where they
// apply.  scripts/check_docs.py checks the rest (see CONTRIBUTING.md, "Code documentation").
#![warn(missing_docs, clippy::missing_panics_doc, clippy::missing_errors_doc)]

pub mod cache;
pub mod engine;
pub mod event;
pub mod font;
pub mod geometry;
pub mod paint;
pub mod root_widget;
pub mod texture;
pub mod widget;
