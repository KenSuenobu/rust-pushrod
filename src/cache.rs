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

use crate::widget::{SystemWidget, Widget};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use crate::geometry::make_rect;

pub struct WidgetCache {
    cache: Vec<SystemWidget>,
    current_widget_id: u32,
}

/// The `WidgetCache` is the store that stores all of the `Widget` objects for a Window.  It handles
/// the widget drawing order, the hidden states, etc. for each `Widget`.  It contains a draw loop
/// which is responsible for determining which objects need to be drawn (which are invalidated),
/// and in what order.
///
/// Any interactions that are performed by the `Engine` are sent here.  This means, the `WidgetCache`
/// has its own event handler.  This is necessary such that `Widget`s can interpret events that may
/// apply to them, for instance: mouse movement, clicks, key press, etc.
impl WidgetCache {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            current_widget_id: 0,
        }
    }

    /// Adds a `SystemWidget` to the cache.
    pub fn add(&mut self, widget: SystemWidget) -> i32 {
        self.cache.push(widget);

        // let origin = widget.get_config().get_point(CONFIG_ORIGIN);
        // let widget_id = self.cache.len();
        //
        // self.cache.push(WidgetContainer::new(
        //     widget,
        //     widget_name,
        //     origin,
        //     widget_id as i32,
        //     0,
        // ));
        //
        // (self.cache.len() - 1) as i32

        (self.cache.len() - 1) as i32
    }

    /// This handles the direct events from the `Engine` class.  Events are not handled by the
    /// `Engine` via indirection.  They are handled by the `Cache`, so that objects that are
    /// selected or have focus are handled by this class.
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                eprintln!("Cache: mouse down: button={} clicks={} x={} y={}", mouse_btn as i32, clicks, x, y);
            },

            Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                eprintln!("Cache: mouse up: button={} clicks={} x={} y={}", mouse_btn as i32, clicks, x, y);
            },

            Event::MouseMotion { x, y, .. } => {
                self.current_widget_id = self.get_widget_id(x as u32, y as u32);

                // Mouse motion in relation to the widget needs to have the X and Y coordinates
                // subtracted from the X/Y coordinates sent here in mouse motion.  Since these
                // values can be negative based on the relation to the currently active object.

                eprintln!("Cache: mouse motion: x={} y={} widget={}", x, y, self.current_widget_id);
            },

            _default => {},
        }
    }

    /// Draws `Widget`s into the `Canvas`.  Determines whether or not a `Widget` is invalidated,
    /// draws it (and its children), and exits after draw completes.  Calls private function
    /// `draw`, which is responsible for copying a texture to the main `Canvas`.
    ///
    /// Returns a boolean indicating whether or not the canvas was invalidated and needs to be
    /// redrawn.  If not, the event loop will not redraw the canvas.
    pub fn draw_loop(&mut self, c: &mut Canvas<Window>) -> bool {
        let mut invalidated = false;
        let cache_size = self.cache.len();

        for i in 0..cache_size {
            match &self.cache[i] {
                SystemWidget::Base(x) => {
                    if x.is_invalidated() {
                        invalidated = true;
                        self.draw(0, c);
                    }
                },

                _unused => {
                    // Do nothing
                },
            }
        }

        invalidated
    }

    /// Draws an object to the screen by calling a `Widget`'s draw function, which draws to a
    /// `TextureStore`.  Once the `TextureStore` has been rendered, it is copied to the
    /// `Canvas` at the widget's origin and size coordinates.
    fn draw(&mut self, widget_id: u32, c: &mut Canvas<Window>) {
        match &mut self.cache[widget_id as usize] {
            SystemWidget::Base(ref mut widget) => {
                let widget_origin = *widget.get_origin();
                let widget_size = *widget.get_size();

                eprintln!("[Base] Drawing ID {} to x {} y {} w {} h {}", widget_id, widget_origin.x, widget_origin.y, widget_size.w, widget_size.h);

                match widget.draw(c) {
                    Some(texture) => {
                        c.copy(
                            texture,
                            None,
                            make_rect(widget_origin, widget_size),
                        ).unwrap()
                    }

                    None => eprintln!("No texture presented."),
                };

                widget.set_invalidated(false);
            }

            _default => { },
        }
    }

    /// Determines the ID of the widget at the given X/Y coordinates.
    fn get_widget_id(&self, _x: u32, _y: u32) -> u32 {
        0
    }

    // fn draw(&mut self, _widget_id: i32, _c: &mut Canvas<Window>) {
    //     // let parents_of_widget = self.get_children_of(widget_id);
    //     //
    //     // if parents_of_widget.is_empty() {
    //     //     return;
    //     // }
    //     //
    //     // for paint_id in &parents_of_widget {
    //     //     let paint_widget = &mut self.cache[*paint_id as usize];
    //     //     let is_hidden = paint_widget.widget.borrow_mut().get_config().is_hidden();
    //     //     let is_enabled = paint_widget.widget.borrow_mut().get_config().is_enabled();
    //     //     let widget_x = paint_widget.widget.borrow_mut().get_config().to_x(0);
    //     //     let widget_y = paint_widget.widget.borrow_mut().get_config().to_y(0);
    //     //     let widget_w = paint_widget
    //     //         .widget
    //     //         .borrow_mut()
    //     //         .get_config()
    //     //         .get_size(CONFIG_SIZE)[0];
    //     //     let widget_h = paint_widget
    //     //         .widget
    //     //         .borrow_mut()
    //     //         .get_config()
    //     //         .get_size(CONFIG_SIZE)[1];
    //     //
    //     //     if !is_hidden {
    //     //         match paint_widget
    //     //             .widget
    //     //             .borrow_mut()
    //     //             .draw(c, &mut self.texture_cache)
    //     //         {
    //     //             Some(texture) => {
    //     //                 c.copy(
    //     //                     texture,
    //     //                     None,
    //     //                     Rect::new(widget_x, widget_y, widget_w, widget_h),
    //     //                 )
    //     //                     .unwrap();
    //     //             }
    //     //             None => eprintln!("No texture presented: ID={}", paint_id),
    //     //         };
    //     //
    //     //         paint_widget.widget.borrow_mut().set_invalidated(false);
    //     //     }
    //     //
    //     //     if *paint_id != widget_id {
    //     //         self.draw(*paint_id, c);
    //     //     }
    //     //
    //     //     if !is_enabled {
    //     //         c.set_draw_color(Color::RGBA(0, 0, 0, 128));
    //     //         c.draw_rect(Rect::new(widget_x, widget_y, widget_w, widget_h))
    //     //             .unwrap();
    //     //     }
    //     // }
    // }
}

impl Default for WidgetCache {
    fn default() -> Self {
        Self::new()
    }
}