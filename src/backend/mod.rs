// Backend
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

//! # Backends
//!
//! A backend is everything between Pushrod and a screen: the window, input, per-widget textures,
//! the clock, the clipboard, and opening URLs.  The core never names a windowing library; it
//! talks to a [`Backend`], and a backend hands it a [`Painter`] to draw
//! each `Widget`.
//!
//! * [`headless`] - [`HeadlessBackend`](headless::HeadlessBackend) and
//!   [`RecordingPainter`](headless::RecordingPainter), in this crate: no window, records paint
//!   calls, for tests.
//! * `pushrod-sdl2` - the `SDL2` backend, in its own crate.
//!
//! An application picks a backend by starting the loop with it, for example
//! `pushrod_sdl2::run(engine, WindowSettings::new("App", size(400, 300)), state)`.

pub mod headless;

use crate::color::Color;
use crate::event::{EngineEvent, EventMask};
use crate::geometry::{Rect, Size};
use crate::input::InputEvent;
use crate::paint::{CachePolicy, Painter};
use crate::widget::WidgetId;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A function another thread calls to wake a waiting backend; see [`Backend::waker`].
pub type Waker = Arc<dyn Fn() + Send + Sync>;

/// The services a windowing system provides to the engine.
///
/// A backend owns the window and one texture per `Widget`.  The engine asks it to paint a
/// `Widget` only when that `Widget` is invalidated, then composes each frame by drawing the
/// cached textures.  Window creation happens when the backend is built (each backend has its own
/// settings); [`Backend::resize_window`] changes it afterward.
pub trait Backend {
    /// Returns how this backend caches painted `Widget`s.
    ///
    /// # Returns
    ///
    /// The backend's [`CachePolicy`].
    fn cache_policy(&self) -> CachePolicy;

    /// Returns the size of the window.
    ///
    /// # Returns
    ///
    /// The window's size in pixels.
    fn window_size(&self) -> Size;

    /// Resizes the window.
    ///
    /// # Arguments
    ///
    /// * `size` - The new size in pixels.
    fn resize_window(&mut self, size: Size);

    /// Tells the backend which input classes the application wants, so it can stop delivering the
    /// rest.  A backend may ignore this; the engine filters as well.
    ///
    /// # Arguments
    ///
    /// * `mask` - The run loop's event mask.
    fn set_event_mask(&mut self, mask: EventMask);

    /// Starts or stops text input to match the focused `Widget`.
    ///
    /// # Arguments
    ///
    /// * `active` - `true` while a text editor has focus.
    fn set_text_input(&mut self, active: bool);

    /// Collects waiting input.
    ///
    /// # Arguments
    ///
    /// * `wait` - How long to block for the first event; `None` returns at once.
    ///
    /// # Returns
    ///
    /// The events that arrived, in order; empty if none did.
    fn poll_events(&mut self, wait: Option<Duration>) -> Vec<InputEvent>;

    /// Returns a function other threads call to end a [`Backend::poll_events`] wait early; the
    /// next poll then yields [`InputEvent::Wake`].
    ///
    /// # Returns
    ///
    /// The waker, or `None` if the backend cannot be woken (the loop then notices posted events
    /// within a frame).
    fn waker(&self) -> Option<Waker>;

    /// Returns the current time.
    ///
    /// # Returns
    ///
    /// The backend's clock reading.
    fn now(&self) -> Instant {
        Instant::now()
    }

    /// Paints one `Widget` into its own texture, creating or resizing the texture as needed.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` the texture belongs to.
    /// * `size` - The `Widget`'s size in pixels; never zero in either dimension.
    /// * `paint` - Draws the `Widget`, given the backend's [`Painter`].
    fn paint_widget(&mut self, id: WidgetId, size: Size, paint: &mut dyn FnMut(&mut dyn Painter));

    /// Starts a frame by clearing the window.
    ///
    /// # Arguments
    ///
    /// * `clear` - The color to clear to.
    fn begin_frame(&mut self, clear: Color);

    /// Copies a `Widget`'s cached texture into the frame; nothing happens if it has none.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` to draw.
    /// * `bounds` - Where to draw it, in window coordinates.
    fn draw_widget(&mut self, id: WidgetId, bounds: Rect);

    /// Finishes the frame and shows it.
    fn present(&mut self);

    /// Returns the engine events the backend produced since the last call (fonts loaded, glyph
    /// atlas pages added).
    ///
    /// # Returns
    ///
    /// The events, in order.
    fn take_notices(&mut self) -> Vec<EngineEvent>;

    /// Reads the clipboard.
    ///
    /// # Returns
    ///
    /// The clipboard text, or `None` if it is empty or unavailable.
    fn clipboard_text(&mut self) -> Option<String>;

    /// Writes the clipboard.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to store.
    fn set_clipboard_text(&mut self, text: &str);

    /// Opens a URL in the system's default handler.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to open.
    ///
    /// # Returns
    ///
    /// `Ok` if the request was made.
    ///
    /// # Errors
    ///
    /// Returns the reason the request failed, such as no handler for the URL.
    fn open_url(&mut self, url: &str) -> Result<(), String>;
}
