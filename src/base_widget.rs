// Base Widget
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

use std::any::Any;
use crate::widget::Widget;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use sdl2::render::{Texture, Canvas};
use sdl2::video::Window;
use sdl2::pixels::Color;

pub struct BaseWidget {
    origin: Point,
    size: Size,
    base_color: Color,
    invalidated: bool,
    texture: TextureStore,
}

/// This is a basic widget that just draws a background with a fill color in a canvas of a given
/// size.  The steps for the drawing inside the `Canvas` are:
///
/// - Create/resize the texture if not created
/// - Set the background draw color to the base (`Color::RGB(255, 255, 255)`)
/// - Clear the canvas with that color
impl Widget for BaseWidget {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_origin(&self) -> &Point {
        &self.origin
    }

    fn get_size(&self) -> &Size {
        &self.size
    }

    fn get_color(&self) -> Color {
        self.base_color
    }

    fn set_origin(&mut self, point: Point) {
        self.origin = point;
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.set_invalidated(true);
    }

    fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
    }

    fn set_color(&mut self, color: Color) {
        self.base_color = color;
        self.set_invalidated(false);
    }

    fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    fn get_texture(&mut self) -> &mut TextureStore {
        &mut self.texture
    }

    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        if self.invalidated {
            let bounds = self.size;
            let base_color = self.base_color;

            self.texture
                .create_or_resize_texture(c, self.size);

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture.set_draw_color(base_color);
                texture.clear();
            }).unwrap();
        }

        self.texture.get_optional_ref()
    }
}

impl BaseWidget {
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            origin,
            size,
            base_color: Color::RGBA(255, 255, 255, 0),
            invalidated: true,
            texture: TextureStore::default(),
        }
    }
}