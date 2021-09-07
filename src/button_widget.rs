// Button Widget
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
use crate::widget::{Widget};
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use crate::base_widget::BaseWidget;
use sdl2::render::{Texture, Canvas};
use sdl2::video::Window;
use sdl2::pixels::Color;

pub struct ButtonWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
}

/// ButtonWidget is a widget that contains a `BaseWidget` and inverts the color of the base
/// based on whether or not the mouse button is down/toggled in the bounds of the button.
/// The drawing order is:
///
/// - Draw the base widget
/// - Borrow the base widget's base widget canvas texture
/// - Draw text on top of the canvas, inverted if selected
impl Widget for ButtonWidget {
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
        self.base_widget.get_color()
    }

    fn set_origin(&mut self, point: Point) {
        self.origin = point;
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.base_widget.set_size(size);
        self.set_invalidated(true);
    }

    fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
        self.base_widget.set_invalidated(state);
    }

    fn set_color(&mut self, color: Color) {
        self.base_widget.set_color(color);
        self.set_invalidated(true);
    }

    fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    fn get_texture(&mut self) -> &mut TextureStore {
        &mut self.texture
    }

    fn draw(&mut self, _c: &mut Canvas<Window>) -> Option<&Texture> {
        None
    }

}
