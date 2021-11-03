// Engine
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
use crate::cache::WidgetCache;
use crate::event::EventHandler;
use crate::geometry::{Point, Size};
use crate::widget::{SystemWidget, Widget};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::Sdl;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Engine {
    frame_rate: u32,
    size: Size,
    running: bool,
    widget_cache: WidgetCache,
    event_handler: Option<Box<dyn EventHandler>>,
}

/// The main engine of Pushrod.  Runs the run loop after adding widgets to a management cache.
impl Engine {
    pub fn new(size: Size, frame_rate: u32) -> Self {
        let mut widget_cache = WidgetCache::default();
        let mut base_widget = BaseWidget::new(Point::new(0, 0), Size::new(size.w, size.h));

        base_widget.set_color(Color::RGBA(255, 255, 255, 255));

        widget_cache.add(SystemWidget::Base(Box::new(base_widget)));

        Self {
            frame_rate,
            size,
            running: true,
            widget_cache,
            event_handler: None,
        }
    }

    /// Shuts down Pushrod for this `Window`.
    pub fn shutdown(&mut self) {
        self.running = false;
    }

    /// Adds a `SystemWidget` to the management stack.
    pub fn add_widget(&mut self, widget: SystemWidget) -> i32 {
        self.widget_cache.add(widget)
    }

    pub fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.event_handler = Some(handler);
    }

    /// Runs an instance of the `Engine`.  Handles events for the given `Window` through `Sdl`,
    /// translating events into usable events by each `Widget`, and by the main application.
    pub fn run(&mut self, sdl: Sdl, window: Window) {
        let mut canvas = window
            .into_canvas()
            .target_texture()
            .accelerated()
            .build()
            .unwrap();

        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();
        canvas.present();

        let mut event_pump = sdl.event_pump().unwrap();
        let fps_as_ms = (1000.0 / self.frame_rate as f64) as u128;

        'running: loop {
            let start = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        break 'running;
                    }

                    remaining_event => {
                        let event_result = self.widget_cache.handle_event(remaining_event);

                        if let Some(handler) = &self.event_handler {
                            // Needs to support handling of multiple events being generated
                            // here.

                            if !event_result.is_empty() {
                                handler.process_event(event_result);
                            }
                        }
                    }
                }
            }

            // Walk through each event that was generated (if not empty)
            // call the event_handler and pass each event one-by-one so it can be processed
            // in the order in which the events were generated.

            // self.widget_cache.tick(self.layout_cache.get_layout_cache());
            // self.layout_cache
            //     .do_layout(self.widget_cache.borrow_cache());

            canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            canvas.clear();

            if self.widget_cache.draw_loop(&mut canvas) {
                canvas.present();
            }

            // This obeys thread sleep time.
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            if now - start < fps_as_ms {
                let diff = fps_as_ms - (now - start);

                sleep(Duration::from_millis(diff as u64));
            }

            if !self.running {
                break 'running;
            }
        }
    }
}
