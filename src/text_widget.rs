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

//! # TextWidget
//!
//! This is a simple system `Widget` that utilizes the `BaseWidget`, and draws on its canvas to
//! create a text message with a given font name, style, color, and message.

use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::FontStyle;
use sdl2::video::Window;
use std::any::Any;
use sdl2::rect::Rect;
use crate::base_widget::BaseWidget;
use crate::event::PushrodEvent;
use crate::font::FontCache;
use crate::geometry::{origin_point, Point, rect, Size};
use crate::texture::TextureStore;
use crate::impl_widget_base;
use crate::widget::Widget;

pub enum TextJustify {
    Left,
    Center,
    Right,
}

pub struct TextWidget {
    id: u32,
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
    font_name: String,
    font_style: FontStyle,
    font_size: u16,
    font_color: Color,
    justification: TextJustify,
    msg: String,
}

impl Widget for TextWidget {
    fn handle_event(&self, _event: PushrodEvent) -> Option<&[PushrodEvent]> { None }

    fn draw(&mut self, c: &mut Canvas<Window>, fc: &mut FontCache) -> Option<&Texture> {
        if self.invalidated {
            self.texture.create_or_resize_texture(c, self.size);

            let base_widget_texture = self.base_widget.draw(c, fc).unwrap();
            let widget_width = self.size.w;
            let ( font_texture, font_width, font_height ) = &fc.render_text(
                c, self.font_name.clone(), self.font_size, self.font_style, self.font_color,
                self.msg.clone(), widget_width
            );
            let texture_y = 0;
            let texture_x = match self.justification {
                TextJustify::Left => 0,
                TextJustify::Right => self.size.w as i32 - *font_width as i32,
                TextJustify::Center => (self.size.w as i32 - *font_width as i32) / 2,
            };

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture
                    .copy(
                        &base_widget_texture,
                        None,
                        rect(0, 0, self.size.w, self.size.h)
                    )
                    .unwrap();
            })
                .unwrap();

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture
                    .copy(
                        &font_texture,
                        None,
                        Rect::new(texture_x as i32, texture_y, *font_width, *font_height),
                    )
                    .unwrap();
            })
            .unwrap();
        }

        self.texture.get_optional_ref()
    }

    impl_widget_base!();
}

impl TextWidget {
    pub fn new(
        origin: Point,
        size: Size,
        font_name: String,
        font_style: FontStyle,
        font_size: u16,
        font_color: Color,
        justification: TextJustify,
        msg: String,
    ) -> Self {
        Self {
            id: 0,
            origin,
            size,
            invalidated: true,
            texture: TextureStore::default(),
            base_widget: BaseWidget::new(origin_point(), size),
            font_name,
            font_style,
            font_size,
            font_color,
            justification,
            msg,
        }
    }

    pub fn set_font_color(&mut self, color: Color) {
        self.font_color = color;
    }

    pub fn set_text(&mut self, msg: String) {
        self.msg = msg;
        self.set_invalidated(true);
    }

    pub fn get_font_color(&self) -> Color { self.font_color }

    pub fn get_text(&self) -> String {
        self.msg.clone()
    }
}
