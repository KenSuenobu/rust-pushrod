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

//! # WidgetCache
//!
//! Contains a cache of the `Widget`s that are members of a display `Window`.  `Widget`s are
//! stored in the order of creation.

use crate::event::PushrodEvent;
use crate::geometry::make_rect;
use crate::widget::{SystemWidget, Widget};
use sdl2::event::Event;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Contains a list of the `Widget`s in a `Vec`.  The `current_widget_id` indicates the currently
/// active `Widget` ID under which the mouse pointer has located.
pub struct WidgetCache {
    cache: Vec<SystemWidget>,
    current_widget_id: u32,
}

impl WidgetCache {
    /// Creates a new `WidgetCache`.
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            current_widget_id: 0,
        }
    }

    /// Adds a `SystemWidget` to the cache, returning its ID after insertion.
    pub fn add(&mut self, widget: SystemWidget) -> i32 {
        self.cache.push(widget);
        (self.cache.len() - 1) as i32
    }

    /// Retrieves an optional reference to the `SystemWidget` object by ID, `None` if not found.
    pub fn get(&self, widget: i32) -> Option<&SystemWidget> {
        if widget > self.cache.len() as i32 {
            None
        } else {
            Some(&self.cache[widget as usize])
        }
    }

    /// Retrieves the next available `Widget` cache ID.
    pub fn get_current_widget(&self) -> u32 {
        self.current_widget_id
    }

    /// Internal function that sends a `PushrodEvent` to a widget, and captures the returned event.
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

            _unused => {
                panic!("[WidgetCache::send_and_receive_event_to_widget] I am trying to handle an event with a widget that I can't handle yet!");
            }
        }

        None
    }

    /// This handles the direct events from the `Engine`.  Raw events are sent in from the
    /// `event_pump` from the `SDL2` library, are translated, and are returned as a `PushrodEvent`
    /// list after being processed.  Any `Widget` that takes in an event, processes it, and generates
    /// a responding `PushrodEvent` in turn are returned here, so that the `Engine` can take the
    /// list of events, and pass them on to the `handle_event` function that was registered in the
    /// runtime loop.
    ///
    /// To expand, the following flow is used:
    ///
    /// - `SDL2` `Event` comes in
    /// - `Event` is deconstructed, handled for `Widget`s in the cache, and sent to the `Widget` using
    ///   `send_and_receive_event_to_widget`
    /// - `PushrodEvent(s)` returned from the function are then yielded back to the `Engine`.
    pub fn handle_event(&mut self, event: Event) -> Vec<&PushrodEvent> {
        // This is our return list of `PushrodEvent` references that are sent back to the
        // `Engine` for processing by the `handle_event` function that may or may not have been
        // set in the `Engine` at runtime.
        let mut return_vector: Vec<&PushrodEvent> = Vec::new();

        // Main event match
        match event {
            // Event::MouseButtonDown {
            //     mouse_btn,
            //     clicks,
            //     x,
            //     y,
            //     ..
            // } => {
            //     eprintln!(
            //         "[WidgetCache::handle_event] mouse down: button={} clicks={} x={} y={}",
            //         mouse_btn as i32, clicks, x, y
            //     );
            // }
            //
            // Event::MouseButtonUp {
            //     mouse_btn,
            //     clicks,
            //     x,
            //     y,
            //     ..
            // } => {
            //     eprintln!(
            //         "[WidgetCache::handle_event] mouse up: button={} clicks={} x={} y={}",
            //         mouse_btn as i32, clicks, x, y
            //     );
            // }

            // Handles a `MouseMotion` event, capturing the timestamp, UI window ID, mouse button ID,
            // the mouse state (down, up), `X` and `Y` coordinates relative to the `Window`, and the
            // `xrel` and `yrel` relative values from the previous and current `x` and `y` mouse
            // states.
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

                // Retrieve the top-most widget ID located within the bounds of the `X` and `Y`
                // coordinates of the mouse.
                self.current_widget_id = self.get_widget_id(x, y);

                // We check the current and previous widget IDs here.  If they have changed, this
                // means the bounds of the pointer have changed from one `Widget` to another.
                // In that case, we generate a `BoundsChange` event, which indicates this change.
                if self.current_widget_id != previous_widget_id {
                    let bounds_event = PushrodEvent::BoundsChange(previous_widget_id, self.current_widget_id);

                    if let Some(x) =
                        self.send_and_receive_event_to_widget(previous_widget_id, bounds_event.clone())
                    {
                        for i in 0..x.len() {
                            return_vector.push(&x[i]);
                        }
                    }

                    if let Some(x) =
                        self.send_and_receive_event_to_widget(self.current_widget_id, bounds_event.clone())
                    {
                        for i in 0..x.len() {
                            return_vector.push(&x[i]);
                        }
                    }
                }

                // Determine the offset of the X and Y coordinates within the `Widget` where the
                // mouse is currently located in relation to its bounds.
                match &self.cache[self.current_widget_id as usize] {
                    SystemWidget::Base(x) => {
                        x_offset = x.get_origin().x;
                        y_offset = x.get_origin().y;
                    }

                    SystemWidget::Box(x) => {
                        x_offset = x.get_origin().x;
                        y_offset = x.get_origin().y;
                    }

                    _unused => {
                        panic!(
                            "[WidgetCache::handle_event] I am trying to handle an event with a widget that I can't handle yet!"
                        );
                    }
                }

                // Wrap the event in a `SystemEvent`, and send it to the `Widget`.  If the `Widget`
                // handles the event and generates its own, add any additional `Event`s generated
                // to the list of return events.
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

    /// This is the main draw loop for all of the `Widget`s in the cache.  Since we are drawing
    /// to a GPU texture, and not the screen directly, there is no need to compute overlapping
    /// components.  Just call the draw method on any of the invalidated components, and let the
    /// GPU blit them to the screen.  Returns `true` if any members of the cache need to be redrawn
    /// to the screen by flipping the GPU texture cache, `false` indicating no change.
    pub fn draw_loop(&mut self, c: &mut Canvas<Window>) -> bool {
        let mut invalidated = false;
        let cache_size = self.cache.len();

        // Walk the cache and only draw objects that are invalidated.
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

                _unused => { }
            }
        }

        invalidated
    }

    /// Internal function that draws the object texture to the GPU.  Clears the invalidation flag
    /// on the `Widget` once blitted.
    fn draw(&mut self, widget_id: u32, c: &mut Canvas<Window>) {
        match &mut self.cache[widget_id as usize] {
            SystemWidget::Base(ref mut widget) => {
                let widget_origin = widget.get_origin().clone();
                let widget_size = widget.get_size().clone();

                match widget.draw(c) {
                    Some(texture) => c
                        .copy(texture, None, make_rect(widget_origin, widget_size))
                        .unwrap(),

                    None => panic!("[WidgetCache::draw] BASE: No texture presented."),
                };

                widget.set_invalidated(false);
            }

            SystemWidget::Box(ref mut widget) => {
                let widget_origin = widget.get_origin().clone();
                let widget_size = widget.get_size().clone();

                match widget.draw(c) {
                    Some(texture) => c
                        .copy(texture, None, make_rect(widget_origin, widget_size))
                        .unwrap(),

                    None => panic!("[WidgetCache::draw] BOX: No texture presented."),
                };

                widget.set_invalidated(false);
            }

            _default => {
                panic!("[WidgetCache::draw] I'm sent a widget that I can't draw yet! (needs to be implemented in 'draw')");
            }
        }
    }

    // Returns the top-most `Widget` ID given `x` and `y` coordinates.  Returns 0 if no widget
    // was found (which indicates the top-level `Widget` ID of the window).
    //
    // BUG: Does not currently handle widget Z coordinates, as no Z coordinates currently exist.
    // "Widget on Top" is not yet implemented, so get widget ID will return the last object that
    // exists in the coordinates given.  Overlapped objects will return the top-most object based
    // on the insertion order in the cache.
    fn get_widget_id(&self, x: i32, y: i32) -> u32 {
        let cache_size = self.cache.len();

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
