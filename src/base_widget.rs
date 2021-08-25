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
use sdl2::rect::Rect;

pub struct BaseWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
}

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

    fn set_origin(&mut self, point: Point) {
        self.origin = point;
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.set_invalidated();
    }

    fn set_invalidated(&mut self) {
        self.invalidated = true;
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
            let base_color = Color::RGB(255, 255, 255);
            let border_color = Color::RGB(0, 0, 0);

            self.texture
                .create_or_resize_texture(c, self.size);

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture.set_draw_color(base_color);
                texture.clear();

                texture.set_draw_color(border_color);
                texture
                    .draw_rect(Rect::new(0, 0, bounds.w, bounds.h))
                    .unwrap();
            }).unwrap();

            self.invalidated = false;
        }

        self.texture.get_optional_ref()
    }

}
