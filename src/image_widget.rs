// Image Widget
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

use crate::base_widget::BaseWidget;
use crate::event::PushrodEvent;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::any::Any;
use crate::impl_widget_base;

pub struct ImageWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
}

/// ImageWidget is a widget that contains a `BaseWidget`, then copies an image over the top of the
/// base after the base has been drawn.  The drawing order is:
///
/// - Draw the base widget
/// - Borrow the base widget's base widget canvas texture
/// - Blit the image transparently over the base widget's canvas texture.
impl Widget for ImageWidget {
    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]> {
        match event {
            PushrodEvent::SystemEvent(ev) => {
                eprintln!("[ImageWidget::handle_event] event: {:?}", ev);
            }

            _ => {}
        }

        None
    }

    fn draw(&mut self, _c: &mut Canvas<Window>) -> Option<&Texture> {
        None
    }

    impl_widget_base!();
}
