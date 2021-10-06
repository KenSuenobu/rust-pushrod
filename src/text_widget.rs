// Text Widget
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

use crate::event::PushrodEvent;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::any::Any;

pub struct TextWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
}

/// `TextWidget` is a widget that renders text from a string within the bounds of the `Widget`.
impl Widget for TextWidget {
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
        Color::BLACK
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
    }

    fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    fn get_texture(&mut self) -> &mut TextureStore {
        &mut self.texture
    }

    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]> {
        match event {
            PushrodEvent::SystemEvent(ev) => {
                eprintln!("[TextWidget::handle_event] event: {:?}", ev);
            },

            _ => {

            }
        }

        None
    }

    fn draw(&mut self, _c: &mut Canvas<Window>) -> Option<&Texture> {
        None
    }
}
