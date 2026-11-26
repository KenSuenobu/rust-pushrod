// pushrod-sdl2
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

//! # pushrod-sdl2
//!
//! The `SDL2` backend for Pushrod.  The `pushrod` core and `pushrod-widgets` contain no `SDL2`
//! code; this crate implements the core's [`Backend`](pushrod::backend::Backend) and
//! [`Painter`](pushrod::paint::Painter) with `SDL2`: one GPU texture per `Widget`, repainted only
//! when invalidated, frames composed by blitting, and text from a glyph atlas.
//!
//! An application starts the loop through this crate:
//!
//! ```rust,no_run
//! use pushrod::engine::Engine;
//! use pushrod::geometry::size;
//! use pushrod_sdl2::{WindowSettings, run};
//!
//! let engine: Engine<u32> = Engine::new(size(400, 300), 60);
//! let clicks = run(engine, WindowSettings::new("My app", size(400, 300)), 0);
//! ```
//!
//! The `bundled` and `static-link` Cargo features live here.  Pushrod uses the following
//! dependency, and the `unsafe_textures` feature is required:
//!
//! ```ignore
//! [dependencies.sdl2]
//! version = "0.38"
//! features = ["ttf", "image", "unsafe_textures"]
//! ```

#![warn(missing_docs, clippy::missing_panics_doc, clippy::missing_errors_doc)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod backend;
pub mod font;
mod input;
pub mod painter;
pub mod texture;
pub mod ttf_layout;

pub use backend::{Sdl2Backend, WindowSettings, event_loop, run};
pub use painter::Sdl2Painter;
pub use ttf_layout::TtfLayout;
