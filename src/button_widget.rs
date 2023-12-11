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

use crate::box_widget::BoxWidget;
use crate::event::PushrodEvent;
use crate::geometry::{make_origin, make_rect, Point, Size};
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::any::Any;
use crate::impl_widget_base;

pub struct ButtonWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    in_bounds: bool,
    texture: TextureStore,
    base_widget: BoxWidget,
}

/// ButtonWidget is a widget that contains a `BaseWidget` and inverts the color of the base
/// based on whether or not the mouse button is down/toggled in the bounds of the button.
/// The drawing order is:
///
/// - Draw the base widget
/// - Borrow the base widget's base widget canvas texture
/// - Draw text on top of the canvas, inverted if selected
impl Widget for ButtonWidget {
    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]> {
        match event {
            PushrodEvent::SystemEvent(ev) => {
                eprintln!("[ButtonWidget::handle_event] event: {:?}", ev);
            }

            PushrodEvent::EnteredBounds(x) => {
                eprintln!("[ButtonWidget::handle_event] Entered bounds: {}", x);
            }

            PushrodEvent::ExitedBounds(x) => {
                eprintln!("[ButtonWidget::handle_event] Exited bounds: {}", x);
            }

            _default => {}
        }

        None
    }

    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        // Draw the base first
        if self.invalidated {
            self.texture.create_or_resize_texture(c, self.size);

            let base_widget_texture = self.base_widget.draw(c).unwrap();
            let widget_size = self.size;

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                // Copy the base texture to the current texture
                texture
                    .copy(
                        base_widget_texture,
                        None,
                        make_rect(make_origin(), widget_size),
                    )
                    .unwrap();
            })
            .unwrap();
        }

        self.texture.get_optional_ref()
    }

    impl_widget_base!();
}

/// This is a `ButtonWidget` that draws a `BoxWidget` as its base, drawing text inside the box.
impl ButtonWidget {
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            origin: origin.clone(),
            size: size.clone(),
            invalidated: true,
            in_bounds: false,
            texture: TextureStore::default(),
            base_widget: BoxWidget::new(make_origin(), size, Color::BLACK, 2),
        }
    }
}
