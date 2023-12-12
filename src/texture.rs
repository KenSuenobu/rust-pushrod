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

use crate::geometry::Size;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

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

    pub fn get_optional_ref(&mut self) -> Option<&Texture> {
        self.store.as_ref()
    }

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

    pub fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    pub fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
    }
}
