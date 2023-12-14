// Texture Store
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

//! # TextureStore
//!
//! The `TextureStore` is used to store the texture on a GPU that can be used as a drawing
//! canvas object.
//!
//! The `Canvas` and `Texture` refer to the `sdl2::render` crate.  Please visit that crate for
//! more information on the drawing features available.

use crate::geometry::Size;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

/// Contains the `TextureStore` components: the optional `Texture`, size, and invalidation flags.
#[derive(Default)]
pub struct TextureStore {
    store: Option<Texture>,
    size: Size,
    invalidated: bool,
}

impl TextureStore {
    /// Retrieves a `&mut Texture` reference to the stored `Texture` object.
    ///
    /// Example use:
    /// ```rust,no_run
    ///   c.with_texture_canvas(texture_store.get_mut_ref(), |texture| {
    ///     texture.set_draw_color(base_color);
    ///     texture.clear();
    ///
    ///     texture.set_draw_color(border_color);
    ///     texture
    ///       .draw_rect(Rect::new(0, 0, 200, 200))
    ///       .unwrap();
    ///   })
    ///   .unwrap();
    /// ```
    pub fn get_mut_ref(&mut self) -> &mut Texture {
        self.store.as_mut().unwrap()
    }

    /// Retrieves a `&Texture` reference object as an optional.  This is a safer way to
    /// draw, as it will not return an object if `create_or_resize_texture` has been not
    /// yet been called.
    pub fn get_optional_ref(&mut self) -> Option<&Texture> {
        self.store.as_ref()
    }

    /// Creates or resizes the texture for drawing.  It will create a new `Texture` object
    /// to draw against if the `store` object currently does not contain a `Texture`, or if
    /// the object size has changed.
    pub fn create_or_resize_texture(&mut self, c: &mut Canvas<Window>, size: Size) {
        if self.store.is_none() || self.size.w != size.w || self.size.h != size.h {
            self.size.w = size.w;
            self.size.h = size.h;
            self.store = Some(c.create_texture_target(None, size.w, size.h).unwrap());

            eprintln!(
                "[create_or_resize_texture] Created texture: size={}x{} (memory={})",
                size.w,
                size.h,
                size.get_memory_size()
            );

            self.set_invalidated(true);
        }
    }

    /// Returns `true` if the object needs to be redrawn to the screen, `false` otherwise.
    pub fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    /// Sets the invalidation flag, forcing the object to be redrawn on the screen if set
    /// to `true`.
    pub fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
    }
}
