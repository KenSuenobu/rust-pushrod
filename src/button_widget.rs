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

//! # ButtonWidget
//!
//! This is a simple system `Widget` that utilizes the `BaseWidget`, and draws on its canvas
//! using a `TextWidget` to display a text message within its bounds.  Displays a border
//! and a filled button with a 3D-like effect.  Generates a `Clicked(u32, u8)` event when
//! a mouse button is clicked and released within its bounds.

use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::FontStyle;
use sdl2::video::Window;
use std::any::Any;
use sdl2::rect::Rect;
use crate::base_widget::BaseWidget;
use crate::event::PushrodEvent;
use crate::font::FontCache;
use crate::geometry::{make_rect, origin_point, Point, point, Size};
use crate::text_widget::{TextJustify, TextWidget};
use crate::texture::TextureStore;
use crate::widget::Widget;
use crate::impl_widget_base;

pub struct ButtonWidget {
    id: i32,
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
    text_widget: TextWidget,
    border_width: u8,
}

impl Widget for ButtonWidget {
    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]> {
        match event {
            PushrodEvent::SystemEvent(widget_id, x) => {
                match &x {
                    _default => {
                        eprintln!("[ButtonWidget] Wrapped SystemEvent: Widget={:?} {:?}", widget_id, &x);
                    }
                }

            }

            _default => {}
        }

        None
    }

    fn draw(&mut self, c: &mut Canvas<Window>, fc: &mut FontCache) -> Option<&Texture> {
        if self.invalidated && self.border_width > 0 {
            self.texture.create_or_resize_texture(c, self.size);

            let base_widget_texture = self.base_widget.draw(c, fc).unwrap();
            let text_widget_texture = self.text_widget.draw(c, fc).unwrap();
            // let border_color = self.border_color;
            let border_width = self.border_width as u32;
            let widget_size = self.size;
            // let widget_width = self.size.w;
            // let widget_height = self.size.h;

            // Draw the background
            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture
                    .copy(
                        base_widget_texture,
                        None,
                        make_rect(origin_point(), widget_size),
                    )
                    .unwrap();
            })
                .unwrap();

            // Overlay the text
            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture
                    .copy(
                        text_widget_texture,
                        None,
                        make_rect(point((border_width + 1) as i32, (border_width + 1) as i32),
                                  Size::new(self.size.w - ((border_width + 1) * 2), self.size.h - ((border_width + 1) * 2))),
                    )
                            .unwrap();
            })
                .unwrap();

            // Draw the border
            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture.set_draw_color(Color::BLACK);
                texture.draw_rect(Rect::new(0, 0, widget_size.w, widget_size.h))
                    .unwrap();
            })
                .unwrap();
        }

        self.texture.get_optional_ref()
    }

    impl_widget_base!();
}

impl ButtonWidget {
    /// Creates a new `BoxWidget` based on its point of origin, size, border color, and border width.
    /// Any borders with a width of 0 will not be drawn.
    pub fn new(origin: Point, size: Size, font_name: String, font_style: FontStyle, font_size: u16,
               font_color: Color, justification: TextJustify, msg: String, border_width: u8) -> Self {
        Self {
            id: 0,
            origin: origin.clone(),
            size: size.clone(),
            invalidated: true,
            texture: TextureStore::default(),
            base_widget: BaseWidget::new(origin_point(), size),
            text_widget: TextWidget::new(origin_point(),
                                         Size::new(size.w - (border_width as u32 * 2), size.h - (border_width as u32 * 2)),
                                         font_name, font_style, font_size, font_color, justification, msg),
            border_width,
        }
    }

    /// Sets the border width in pixels.
    pub fn set_border_width(&mut self, width: u8) {
        self.border_width = width;
        self.set_invalidated(true);
    }

    /// Retrieves the border width in pixels.
    pub fn get_border_width(&self) -> u8 {
        self.border_width
    }

    /// Changes the text displayed inside the button.
    pub fn set_text(&mut self, msg: String) {
        self.text_widget.set_text(msg.clone());
    }
}
