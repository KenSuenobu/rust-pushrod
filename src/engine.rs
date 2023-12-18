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

//! # Engine
//!
//! This is the run loop processor that enables `Pushrod` to open a new window, layout the `Widget`s
//! in the display, and handle the `Event` loop.
//!
//! The event loop processes events in order of receipt from the `SDL2` library, which uses the
//! `event_pump` function to get a list of events to process.  The events are then processed, and
//! sent to the `widget_cache` to be handled.  Any `Event`s generated from the `Widget`s in the
//! cache will be added to the list of `Event`s to be processed, and sent along the call chain.
//!
//! If an `event_handler` has been attached to the `Engine`, the event handler will be called,
//! and passed in the resulting `PushrodEvent` with the message to process.  This function is where
//! you would handle an event generated from a `Widget` that was interacted with, such as a
//! mouse click, keyboard press, and so on.

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

/// Engine structure.  Contains the desired display frame rate, size of the UI window, a
/// running flag, the `WidgetCache` for the tree of `Widget`s in the UI, and an optional event
/// handler.
pub struct Engine {
    frame_rate: u32,
    size: Size,
    running: bool,
    widget_cache: WidgetCache,
    event_handler: Option<Box<dyn EventHandler>>,
}

impl Engine {
    /// Creates a new engine given a `Size` and a `frame_rate` in which to refresh the screen.
    /// Default frame rates are 60 and 120.
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

    /// Shuts down the running state, terminating processing.
    pub fn shutdown(&mut self) {
        self.running = false;
    }

    /// Adds a new `Widget` to the UI.
    pub fn add_widget(&mut self, widget: SystemWidget) -> i32 {
        self.widget_cache.add(widget)
    }

    /// Adds an event handler to the `Engine` against which generated events will be sent.
    pub fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.event_handler = Some(handler);
    }

    /// The main run loop.
    pub fn run(&mut self, sdl: Sdl, window: Window) {
        // Initializes the canvas, creating a textured canvas against which GPU textures will be
        // used, specifying hardware acceleration.
        let mut canvas = window
            .into_canvas()
            .target_texture()
            .accelerated()
            .build()
            .unwrap();

        // Sets the screen color to all white, clears and presents the canvas to display.
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();
        canvas.present();

        // Attaches to the `SDL` library event pump, against which all windowed events are
        // sent.  Also determines the amount of time to wait between frames.
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

            // Sleep a determinant amount of milliseconds to generate enough of a delay for the
            // frames per second to be honored, so no screen tearing occurs (for vsync)
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
