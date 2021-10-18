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

use crate::event::PushrodEvent;
use crate::geometry::make_rect;
use crate::widget::{SystemWidget, Widget};
use sdl2::event::Event;
use sdl2::render::Canvas;
use sdl2::video::Window;

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
        (self.cache.len() - 1) as i32
    }

    /// Retrieves a `SystemWidget` from the cache by its ID.
    pub fn get(&self, widget: i32) -> Option<&SystemWidget> {
        if widget > self.cache.len() as i32 {
            None
        } else {
            Some(&self.cache[widget as usize])
        }
    }

    /// Retrieves the current widget ID
    pub fn get_current_widget(&self) -> u32 {
        self.current_widget_id
    }

    fn send_and_receive_event_to_widget(
        &self,
        widget_id: u32,
        event: PushrodEvent,
    ) -> Option<&[PushrodEvent]> {
        match &self.cache[widget_id as usize] {
            SystemWidget::Base(x) => {
                return x.handle_event(event);
            }

            SystemWidget::Box(x) => {
                return x.handle_event(event);
            }

            SystemWidget::Button(x) => {
                return x.handle_event(event);
            }

            SystemWidget::Image(x) => {
                return x.handle_event(event);
            }

            _unused => {
                // Do nothing
                eprintln!("[WidgetCache::send_and_receive_event_to_widget] I am trying to handle an event with a widget that I can't handle yet!");
            }
        }

        None
    }

    /// This handles the direct events from the `Engine` class.  Events are not handled by the
    /// `Engine` via indirection.  They are handled by the `Cache`, so that objects that are
    /// selected or have focus are handled by this class.
    pub fn handle_event(&mut self, event: Event) -> Vec<&PushrodEvent> {
        let mut return_vector: Vec<&PushrodEvent> = Vec::new();

        match event {
            Event::MouseButtonDown {
                mouse_btn,
                clicks,
                x,
                y,
                ..
            } => {
                eprintln!(
                    "[WidgetCache::handle_event] mouse down: button={} clicks={} x={} y={}",
                    mouse_btn as i32, clicks, x, y
                );
            }

            Event::MouseButtonUp {
                mouse_btn,
                clicks,
                x,
                y,
                ..
            } => {
                eprintln!(
                    "[WidgetCache::handle_event] mouse up: button={} clicks={} x={} y={}",
                    mouse_btn as i32, clicks, x, y
                );
            }

            Event::MouseMotion {
                timestamp,
                window_id,
                which,
                mousestate,
                x,
                y,
                xrel,
                yrel,
            } => {
                let mut x_offset = 0;
                let mut y_offset = 0;
                let previous_widget_id = self.current_widget_id;

                self.current_widget_id = self.get_widget_id(x, y);

                // Send previous widget ID event that an event left it bounds
                // Send current widget ID event that event entered its bounds
                if self.current_widget_id != previous_widget_id {
                    let exited_event = PushrodEvent::ExitedBounds(previous_widget_id);
                    let entered_event = PushrodEvent::EnteredBounds(self.current_widget_id);

                    // Exited event - copy return events only if some are generated
                    if let Some(x) =
                        self.send_and_receive_event_to_widget(previous_widget_id, exited_event)
                    {
                        for i in 0..x.len() {
                            return_vector.push(&x[i]);
                        }
                    }

                    // Entered event
                    if let Some(x) =
                        self.send_and_receive_event_to_widget(self.current_widget_id, entered_event)
                    {
                        for i in 0..x.len() {
                            return_vector.push(&x[i]);
                        }
                    }
                }

                match &self.cache[self.current_widget_id as usize] {
                    SystemWidget::Base(x) => {
                        x_offset = x.get_origin().x;
                        y_offset = x.get_origin().y;
                    }

                    SystemWidget::Box(x) => {
                        x_offset = x.get_origin().x;
                        y_offset = x.get_origin().y;
                    }

                    SystemWidget::Button(x) => {
                        x_offset = x.get_origin().x;
                        y_offset = x.get_origin().y;
                    }

                    _unused => {
                        // Do nothing
                        eprintln!(
                            "[WidgetCache::handle_event] I am trying to handle an event with a widget that I can't handle yet!"
                        );
                    }
                }

                if let Some(x) = self.send_and_receive_event_to_widget(
                    self.current_widget_id,
                    PushrodEvent::SystemEvent(Event::MouseMotion {
                        timestamp,
                        window_id,
                        which,
                        mousestate,
                        x: x - x_offset,
                        y: y - y_offset,
                        xrel,
                        yrel,
                    }),
                ) {
                    for i in 0..x.len() {
                        return_vector.push(&x[i]);
                    }
                }
            }

            _default => {}
        }

        return_vector
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

        // Walk the size of the cache and only draw objects that are invalidated.
        for i in 0..cache_size {
            match &self.cache[i] {
                SystemWidget::Base(x) => {
                    if x.is_invalidated() {
                        invalidated = true;
                        self.draw(i as u32, c);
                    }
                }

                SystemWidget::Box(x) => {
                    if x.is_invalidated() {
                        invalidated = true;
                        self.draw(i as u32, c);
                    }
                }

                SystemWidget::Button(x) => {
                    if x.is_invalidated() {
                        invalidated = true;
                        self.draw(i as u32, c);
                    }
                }

                _unused => {
                    // Do nothing
                    eprintln!("[WidgetCache::draw_loop] I'm sent a widget that I can't draw yet!");
                }
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

                eprintln!(
                    "[WidgetCache::draw] Base: Drawing ID {} to x {} y {} w {} h {}",
                    widget_id, widget_origin.x, widget_origin.y, widget_size.w, widget_size.h
                );

                match widget.draw(c) {
                    Some(texture) => c
                        .copy(texture, None, make_rect(widget_origin, widget_size))
                        .unwrap(),

                    None => eprintln!("[WidgetCache::draw] BASE: No texture presented."),
                };

                widget.set_invalidated(false);
            }

            SystemWidget::Box(ref mut widget) => {
                let widget_origin = *widget.get_origin();
                let widget_size = *widget.get_size();

                eprintln!(
                    "[WidgetCache::draw] Box: Drawing ID {} to x {} y {} w {} h {}",
                    widget_id, widget_origin.x, widget_origin.y, widget_size.w, widget_size.h
                );

                match widget.draw(c) {
                    Some(texture) => c
                        .copy(texture, None, make_rect(widget_origin, widget_size))
                        .unwrap(),

                    None => eprintln!("[WidgetCache::draw] BOX: No texture presented."),
                };

                widget.set_invalidated(false);
            }

            SystemWidget::Button(ref mut widget) => {
                let widget_origin = *widget.get_origin();
                let widget_size = *widget.get_size();

                eprintln!(
                    "[WidgetCache::draw] Button: Drawing ID {} to x {} y {} w {} h {}",
                    widget_id, widget_origin.x, widget_origin.y, widget_size.w, widget_size.h
                );

                match widget.draw(c) {
                    Some(texture) => c
                        .copy(texture, None, make_rect(widget_origin, widget_size))
                        .unwrap(),

                    None => eprintln!("[WidgetCache::draw] BUTTON: No texture presented."),
                };

                widget.set_invalidated(false);
            }

            _default => {
                // Do nothing
                eprintln!("[WidgetCache::draw] I'm sent a widget that I can't draw yet!");
            }
        }
    }

    /// Determines the ID of the widget at the given X/Y coordinates.
    fn get_widget_id(&self, x: i32, y: i32) -> u32 {
        let cache_size = self.cache.len();

        // Mouse motion in relation to the widget needs to have the X and Y coordinates
        // subtracted from the X/Y coordinates sent here in mouse motion.  Since these
        // values can be negative based on the relation to the currently active object.

        let mut widget_id = 0;

        for i in 0..cache_size {
            let mut start_x = 0;
            let mut end_x = 0;
            let mut start_y = 0;
            let mut end_y = 0;

            match &self.cache[i] {
                SystemWidget::Base(x) => {
                    start_x = x.get_origin().x;
                    start_y = x.get_origin().y;
                    end_x = start_x + x.get_size().w as i32;
                    end_y = start_y + x.get_size().h as i32;
                }

                SystemWidget::Box(x) => {
                    start_x = x.get_origin().x;
                    start_y = x.get_origin().y;
                    end_x = start_x + x.get_size().w as i32;
                    end_y = start_y + x.get_size().h as i32;
                }

                SystemWidget::Button(x) => {
                    start_x = x.get_origin().x;
                    start_y = x.get_origin().y;
                    end_x = start_x + x.get_size().w as i32;
                    end_y = start_y + x.get_size().h as i32;
                }

                _default => {
                    // Do nothing.
                }
            }

            if x >= start_x && x <= end_x && y >= start_y && y <= end_y {
                widget_id = i as u32;
            }
        }

        widget_id
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
