// Cache
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

use crate::widget::SystemWidget;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct WidgetCache {
    cache: Vec<SystemWidget>,
}

impl WidgetCache {
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
        }
    }

    fn draw(&mut self, _widget_id: i32, _c: &mut Canvas<Window>) {
        // let parents_of_widget = self.get_children_of(widget_id);
        //
        // if parents_of_widget.is_empty() {
        //     return;
        // }
        //
        // for paint_id in &parents_of_widget {
        //     let paint_widget = &mut self.cache[*paint_id as usize];
        //     let is_hidden = paint_widget.widget.borrow_mut().get_config().is_hidden();
        //     let is_enabled = paint_widget.widget.borrow_mut().get_config().is_enabled();
        //     let widget_x = paint_widget.widget.borrow_mut().get_config().to_x(0);
        //     let widget_y = paint_widget.widget.borrow_mut().get_config().to_y(0);
        //     let widget_w = paint_widget
        //         .widget
        //         .borrow_mut()
        //         .get_config()
        //         .get_size(CONFIG_SIZE)[0];
        //     let widget_h = paint_widget
        //         .widget
        //         .borrow_mut()
        //         .get_config()
        //         .get_size(CONFIG_SIZE)[1];
        //
        //     if !is_hidden {
        //         match paint_widget
        //             .widget
        //             .borrow_mut()
        //             .draw(c, &mut self.texture_cache)
        //         {
        //             Some(texture) => {
        //                 c.copy(
        //                     texture,
        //                     None,
        //                     Rect::new(widget_x, widget_y, widget_w, widget_h),
        //                 )
        //                     .unwrap();
        //             }
        //             None => eprintln!("No texture presented: ID={}", paint_id),
        //         };
        //
        //         paint_widget.widget.borrow_mut().set_invalidated(false);
        //     }
        //
        //     if *paint_id != widget_id {
        //         self.draw(*paint_id, c);
        //     }
        //
        //     if !is_enabled {
        //         c.set_draw_color(Color::RGBA(0, 0, 0, 128));
        //         c.draw_rect(Rect::new(widget_x, widget_y, widget_w, widget_h))
        //             .unwrap();
        //     }
        // }
    }
}

impl Default for WidgetCache {
    fn default() -> Self {
        Self::new()
    }
}