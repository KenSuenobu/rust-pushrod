// Widget
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
use crate::image_widget::ImageWidget;
use crate::button_widget::ButtonWidget;
use crate::geometry::{Size, Point};
use crate::base_widget::BaseWidget;
use crate::texture::TextureStore;
use sdl2::render::{Texture, Canvas};
use sdl2::video::Window;

pub trait Widget {
    fn as_any(&self) -> &dyn Any;

    fn get_origin(&self) -> &Point;

    fn get_size(&self) -> &Size;

    fn set_origin(&mut self, point: Point);

    fn set_size(&mut self, size: Size);

    fn set_invalidated(&mut self, state: bool);

    fn is_invalidated(&self) -> bool;

    fn get_texture(&mut self) -> &mut TextureStore;

    fn draw(&mut self, _c: &mut Canvas<Window>) -> Option<&Texture>;
}

pub enum SystemWidget {
    Base(Box<BaseWidget>),
    Button(Box<ButtonWidget>),
    Image(Box<ImageWidget>),
    Custom(Box<dyn Widget>),
}
