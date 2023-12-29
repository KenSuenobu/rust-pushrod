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

//! # BoxWidget
//!
//! This is a simple system `Widget` that utilizes the `BaseWidget`, and draws on its canvas to
//! create a border of a specific width and color.

use crate::base_widget::BaseWidget;
use crate::event::PushrodEvent;
use crate::geometry::{origin_point, make_rect, Point, Size};
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::any::Any;
use crate::font::FontCache;
use crate::impl_widget_base;

pub struct BoxWidget {
    id: u32,
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
    border_color: Color,
    border_width: u8,
}

impl Widget for BoxWidget {
    fn handle_event(&self, _event: PushrodEvent) -> Option<&[PushrodEvent]> { None }

    fn draw(&mut self, c: &mut Canvas<Window>, fc: &mut FontCache) -> Option<&Texture> {
        if self.invalidated && self.border_width > 0 {
            self.texture.create_or_resize_texture(c, self.size);

            let base_widget_texture = self.base_widget.draw(c, fc).unwrap();
            let border_color = self.border_color;
            let border_width = self.border_width;
            let widget_size = self.size;
            let widget_width = self.size.w;
            let widget_height = self.size.h;

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture
                    .copy(
                        base_widget_texture,
                        None,
                        make_rect(origin_point(), widget_size),
                    )
                    .unwrap();

                texture.set_draw_color(border_color);

                for i in 0..border_width as i32 {
                    let computed_width = (widget_width as u32 - (i as u32 * 2u32)) as u32;
                    let computed_height = (widget_height as u32 - (i as u32 * 2u32)) as u32;

                    texture
                        .draw_rect(Rect::new(i, i, computed_width, computed_height))
                        .unwrap();
                }
            })
            .unwrap();
        }

        self.texture.get_optional_ref()
    }

    impl_widget_base!();
}

impl BoxWidget {
    /// Creates a new `BoxWidget` based on its point of origin, size, border color, and border width.
    /// Any borders with a width of 0 will not be drawn.
    pub fn new(origin: Point, size: Size, border_color: Color, border_width: u8) -> Self {
        Self {
            id: 0,
            origin: origin.clone(),
            size: size.clone(),
            invalidated: true,
            texture: TextureStore::default(),
            base_widget: BaseWidget::new(origin_point(), size),
            border_color,
            border_width,
        }
    }

    /// Sets the border width in pixels.
    pub fn set_border_width(&mut self, width: u8) {
        self.border_width = width;
        self.set_invalidated(true);
    }

    /// Sets the border color, which can be a `Color::RGB` or `Color::RGBA`.
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
        self.set_invalidated(true);
    }

    /// Retrieves the border width in pixels.
    pub fn get_border_width(&self) -> u8 {
        self.border_width
    }

    /// Retrieves the border color.
    pub fn get_border_color(&self) -> Color {
        self.border_color
    }
}
