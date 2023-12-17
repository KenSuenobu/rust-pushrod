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
//! # Dependencies
//!
//! Pushrod uses the following dependency:
//! ```ignore
//! [dependencies.sdl2]
//! version = "^0.36.0"
//! features = ["ttf", "image", "unsafe_textures"]
//! ```
//!
//! Note, the `unsafe_textures` feature is required for OpenGL functionality.

pub mod base_widget;
pub mod box_widget;
pub mod cache;
pub mod engine;
pub mod event;
pub mod geometry;
pub mod texture;
pub mod widget;
pub mod font;
mod text_widget;
