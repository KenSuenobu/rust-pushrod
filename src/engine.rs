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

use crate::geometry::Size;
use sdl2::Sdl;
use sdl2::video::Window;
use std::thread::sleep;
use std::time::{Duration, UNIX_EPOCH, SystemTime};
use sdl2::pixels::Color;
use crate::cache::WidgetCache;
use crate::widget::SystemWidget;
use sdl2::event::Event;

pub struct Engine {
    frame_rate: u32,
    size: Size,
    running: bool,
    widget_cache: WidgetCache,
}

/// The main engine of Pushrod.  Runs the run loop after adding widgets to a management cache.
impl Engine {
    pub fn new(size: Size, frame_rate: u32) -> Self {
        Self {
            frame_rate,
            size,
            running: true,
            widget_cache: WidgetCache::new(),
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

    /// Runs an instance of the `Engine`.  Handles events for the given `Window` through `Sdl`,
    /// translating events into usable events by each `Widget`, and by the main application.
    pub fn run(&mut self, sdl: Sdl, window: Window) {
        let mut canvas = window
            .into_canvas()
            .target_texture()
            .accelerated()
            .build()
            .unwrap();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
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
                    // Event::MouseWheel { x, y, .. } => {
                    //     self.widget_cache.mouse_scrolled(
                    //         self.current_widget_id,
                    //         vec![x, y],
                    //         self.layout_cache.get_layout_cache(),
                    //     );
                    // }

                    Event::Quit { .. } => {
                        // if self.call_exit_callback() {
                        //     break 'running;
                        // }

                        break 'running;
                    }

                    remaining_event => {
                        let event_result = self.widget_cache.handle_event(remaining_event);
                    }
                }
            }

            // self.widget_cache.tick(self.layout_cache.get_layout_cache());
            // self.layout_cache
            //     .do_layout(self.widget_cache.borrow_cache());

            canvas.set_draw_color(Color::RGB(255, 255, 255));
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