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

use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::FontStyle;
use sdl2::video::Window;
use std::any::Any;
use crate::base_widget::BaseWidget;
use crate::event::PushrodEvent;
use crate::font::FontCache;
use crate::geometry::{origin_point, Point, Size};
use crate::texture::TextureStore;
use crate::impl_widget_base;
use crate::widget::Widget;

pub enum TextJustify {
    Left,
    Center,
    Right,
}

pub struct TextWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    font_cache: FontCache,
    base_widget: BaseWidget,
    font_name: String,
    font_style: FontStyle,
    font_size: u32,
    font_color: Color,
    justification: TextJustify,
    msg: String,
}

impl Widget for TextWidget {
    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]> { None }

    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        if self.invalidated {
            // self.texture.create_or_resize_texture(c, self.size);
            //
            // let base_widget_texture = self.base_widget.draw(c).unwrap();
            // let border_color = self.border_color;
            // let border_width = self.border_width;
            // let widget_size = self.size;
            // let widget_width = self.size.w;
            // let widget_height = self.size.h;
            //
            // c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
            //     texture
            //         .copy(
            //             base_widget_texture,
            //             None,
            //             make_rect(origin_point(), widget_size),
            //         )
            //         .unwrap();
            //
            //     texture.set_draw_color(border_color);
            //
            //     for i in 0..border_width as i32 {
            //         let computed_width = (widget_width as u32 - (i as u32 * 2u32)) as u32;
            //         let computed_height = (widget_height as u32 - (i as u32 * 2u32)) as u32;
            //
            //         texture
            //             .draw_rect(Rect::new(i, i, computed_width, computed_height))
            //             .unwrap();
            //     }
            // })
            //     .unwrap();
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
        font_size: u32,
        font_color: Color,
        justification: TextJustify,
        msg: String,
    ) -> Self {
        Self {
            origin,
            size,
            invalidated: true,
            texture: TextureStore::default(),
            font_cache: FontCache::default(),
            base_widget: BaseWidget::new(origin_point(), size),
            font_name,
            font_style,
            font_size,
            font_color,
            justification,
            msg,
        }
    }

    pub fn set_text(&mut self, msg: String) {
        self.msg = msg;
        self.set_invalidated(true);
    }

    pub fn get_text(&self) -> String {
        self.msg.clone()
    }
}
