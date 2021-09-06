// Box Widget
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
use crate::geometry::{Point, Size, make_rect, make_origin};
use crate::texture::TextureStore;
use crate::base_widget::BaseWidget;
use sdl2::render::{Texture, Canvas};
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub struct BoxWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
    border_color: Color,
    border_width: u8,
}

/// `BoxWidget` is a widget that contains a `BaseWidget`, and draws a border around the base after
/// it is rendered.  The border color is controlled as part of the `BoxWidget`'s properties.
///
/// The drawing order is:
///
/// - Draw the base widget
/// - Borrow the base widget's base widget canvas texture
/// - Draw a box around it with the specified with and color
impl Widget for BoxWidget {
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

    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        // Draw the base first
        if self.invalidated {
            self.texture
                .create_or_resize_texture(c, self.size);

            let base_widget_texture = self.base_widget.draw(c).unwrap();
            let border_color = self.border_color;
            let border_width = self.border_width;
            let widget_width = self.size.w;
            let widget_height = self.size.h;

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                // Copy the base texture to the current texture
                texture
                    .copy(
                        base_widget_texture,
                        None,
                        make_rect(make_origin(), self.size),
                    )
                    .unwrap();

                // Now we draw the box.
                texture.set_draw_color(border_color);

                for i in 0..border_width as i32 {
                    let computed_width = (widget_width as u32 - (i as u32 * 2u32)) as u32;
                    let computed_height = (widget_height as u32 - (i as u32 * 2u32)) as u32;

                    texture.draw_rect(Rect::new(i, i, computed_width, computed_height))
                        .unwrap();
                }
            }).unwrap();
        }

        self.texture.get_optional_ref()
    }

}

impl BoxWidget {
    fn set_border_width(&mut self, width: u8) {
        self.border_width = width;
        self.set_invalidated(true);
    }

    fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
        self.set_invalidated(true);
    }

    fn get_border_width(&self) -> u8 {
        self.border_width
    }

    fn get_border_color(&self) -> Color {
        self.border_color
    }
}