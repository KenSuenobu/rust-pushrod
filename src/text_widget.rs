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
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::video::Window;
use std::any::Any;
use std::path::Path;
use sdl2::ttf::{FontStyle, Sdl2TtfContext};

pub struct TextWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    ttf_context: Sdl2TtfContext,
    text: String,
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

    fn set_color(&mut self, _color: Color) {}

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
            }

            _ => {}
        }

        None
    }

    fn draw(&mut self, _c: &mut Canvas<Window>) -> Option<&Texture> {
        if self.invalidated {
            //     self.texture.create_or_resize_texture(c, self.size);
            //
            //     let base_color = self.base_color;
            //     let size = self.size;
            //
            //     // Draw the background with only the base color.
            //     c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
            //         texture.set_draw_color(base_color);
            //         texture.clear();
            //     })
            //         .unwrap();
            // }
            let bounds = self.properties.get_bounds();
            let (font_texture, width, height) = t.render_text(c);
            let text_justification = self.properties.get_value(PROPERTY_TEXT_JUSTIFICATION);
            let texture_y = 0;
            let widget_w = bounds.0;
            let texture_x: i32 = match text_justification {
                TEXT_JUSTIFY_LEFT => 0,
                TEXT_JUSTIFY_CENTER => (widget_w - width) as i32 / 2,
                TEXT_JUSTIFY_RIGHT => (widget_w - width) as i32,
                _ => 0,
            };

            self.texture_store
                .create_or_resize_texture(c, bounds.0, bounds.1);

            let cloned_properties = self.properties.clone();

            c.with_texture_canvas(self.texture_store.get_mut_ref(), |texture| {
                draw_base(texture, &cloned_properties, None);

                texture
                    .copy(
                        &font_texture,
                        None,
                        Rect::new(texture_x, texture_y, width, height),
                    )
                    .unwrap();
            })
                .unwrap();
        }

        self.texture.get_optional_ref()
    }
}

impl TextWidget {

    pub fn new(origin: Point, size: Size, text: String) -> Self {
        Self {
            origin,
            size,
            invalidated: false,
            texture: TextureStore::default(),
            ttf_context: sdl2::ttf::init().map_err(|e| e.to_string()).unwrap(),
            text,
        }
    }

    /// Renders text, given the font name, size, style, color, string, and max width.  Transfers
    /// ownership of the `Texture` to the calling function, returns the width and height of the
    /// texture after rendering.  By using the identical font name, size, and style, if SDL2 caches
    /// the font data, this will allow the font to be cached internally.
    pub fn render_text(
        &mut self,
        c: &mut Canvas<Window>,
    ) -> (Texture, u32, u32) {
        let ttf_context = &self.ttf_context;
        let texture_creator = c.texture_creator();
        let font_name = "assets/OpenSans-Regular.ttf";
        let text_color = Color::BLACK;
        let font_size = 14;
        let font_style: FontStyle = FontStyle::NORMAL;
        let text_message = "Hello World";
        let mut font = ttf_context
            .load_font(Path::new(&font_name), font_size as u16)
            .unwrap();
        let surface = font
            .render(&text_message)
            .blended_wrapped(text_color, self.size.w)
            .map_err(|e| e.to_string())
            .unwrap();
        let font_texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let TextureQuery { width, height, .. } = font_texture.query();

        font.set_style(font_style);

        (font_texture, width, height)
    }
}
