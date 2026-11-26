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
//! The documentation site, <https://pushrod.nobudata.com>, has step-by-step guides for installing
//! Pushrod and SDL2 on each platform, writing applications, and writing widgets, with many examples.
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
//! ## Cargo features
//!
//! | Feature | Effect | Needs on the build machine |
//! |---|---|---|
//! | `bundled` | Builds `SDL2` from source instead of linking the system library. | `CMake` and a C compiler; `SDL2_ttf` and `SDL2_image` from the system |
//! | `static-link` | Links `SDL2`, `SDL2_ttf`, and `SDL2_image` into the executable instead of loading them at run time. | Static builds of all three libraries; not with `bundled` |
//! | `parley` | Lays out text with Parley and rasterizes glyphs with swash. | Nothing extra |
//!
//! `bundled` does not yet give a fully self-contained build: the `sdl2` crate builds `SDL2`
//! only, and text and images still use `SDL2_ttf` and `SDL2_image`.  Don't combine `bundled`
//! with `static-link`: `sdl2-sys` 0.38 then links no `SDL2_ttf` or `SDL2_image`, so the build
//! fails with undefined symbols.
//!
//! Pushrod uses the following dependency, and the `unsafe_textures` feature is required:
//!
//! ```ignore
//! [dependencies.sdl2]
//! version = "0.38"
//! features = ["ttf", "image", "unsafe_textures"]
//! ```

#![warn(
    missing_docs,
    missing_debug_implementations,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod backend;
mod compositor;
pub mod font;
pub mod image_cache;
mod input;
pub mod painter;
pub mod scale;
pub mod texture;
pub mod ttf_layout;

pub use backend::{Sdl2Backend, WindowSettings, event_loop, run};
pub use image_cache::ImageCache;
pub use painter::Sdl2Painter;
pub use ttf_layout::TtfLayout;
