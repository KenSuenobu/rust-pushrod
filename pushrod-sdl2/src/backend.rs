// Sdl2Backend
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

//! The `SDL2` [`Backend`]: window, canvas, input pump, per-widget textures, clipboard, and URLs,
//! plus [`run`] and [`event_loop`], which build a window and start the engine's loop on it.

use crate::font::{FontCache, FontNotice};
use crate::image_cache::ImageCache;
use crate::input::{Wake, to_input_event, types_to_disable};
use crate::painter::{Sdl2Painter, to_sdl_color, to_sdl_rect};
use crate::texture::TextureStore;
use pushrod::backend::{Backend, Waker};
use pushrod::color::Color;
use pushrod::engine::{Engine, EventLoop};
use pushrod::error::Error;
use pushrod::event::{EngineEvent, EventMask};
use pushrod::geometry::{Rect, Size};
use pushrod::input::InputEvent;
use pushrod::paint::{CachePolicy, Painter};
use pushrod::widget::WidgetId;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::clipboard::ClipboardUtil;
use sdl2::image::SaveSurface;
use sdl2::keyboard::TextInputUtil;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// How the window is created.
///
/// # Examples
///
/// ```rust
/// use pushrod::geometry::size;
/// use pushrod_sdl2::WindowSettings;
///
/// let settings = WindowSettings::new("Inventory", size(640, 480));
///
/// assert_eq!(settings.title, "Inventory");
/// assert!(settings.centered);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct WindowSettings {
    /// The window title.
    pub title: String,
    /// The size of the window's drawing area, in pixels.
    pub size: Size,
    /// Whether the window opens centered on the screen.
    pub centered: bool,
    /// Whether the user can resize the window.
    pub resizable: bool,
}

impl WindowSettings {
    /// Creates settings for a centered, fixed-size window.
    ///
    /// # Arguments
    ///
    /// * `title` - The window title.
    /// * `size` - The size of the drawing area in pixels.
    ///
    /// # Returns
    ///
    /// The settings.
    pub fn new(title: impl Into<String>, size: Size) -> Self {
        Self {
            title: title.into(),
            size,
            centered: true,
            resizable: false,
        }
    }
}

/// The `SDL2` [`Backend`].
///
/// Each `Widget` paints into its own GPU texture, only when invalidated, and frames are composed
/// by copying the cached textures to the window.  Text comes from the glyph atlas in the
/// [`FontCache`].
pub struct Sdl2Backend {
    _sdl: Sdl,
    // Declared before `canvas` so the cached textures are freed while the renderer lives.
    images: ImageCache,
    canvas: Canvas<Window>,
    pump: EventPump,
    text_input: TextInputUtil,
    clipboard: ClipboardUtil,
    waker: Option<Waker>,
    textures: HashMap<WidgetId, TextureStore>,
    fonts: FontCache,
    applied_mask: EventMask,
    /// Errors that happened since the last [`Backend::take_notices`].
    errors: Vec<Error>,
    /// Every error reported so far, so a failure that repeats on every repaint (a missing font)
    /// is reported once.
    reported: HashSet<Error>,
}

impl std::fmt::Debug for Sdl2Backend {
    /// Formats the value for debugging.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sdl2Backend").finish_non_exhaustive()
    }
}

impl Sdl2Backend {
    /// Initializes `SDL2` and opens a window.
    ///
    /// On macOS the OpenGL renderer is requested unless the `SDL_RENDER_DRIVER` environment
    /// variable is set: with the Metal and software renderers the whole window appears to move
    /// one pixel each time a click repaints a widget, and OpenGL does not.  Set
    /// `SDL_RENDER_DRIVER` to choose another renderer.
    ///
    /// # Arguments
    ///
    /// * `settings` - How to create the window.
    ///
    /// # Returns
    ///
    /// The backend, with a hardware-accelerated canvas cleared to white.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Backend`] with the `SDL2` message if `SDL2`, its video subsystem, the
    /// window, the canvas or the event pump cannot be created.
    pub fn new(settings: &WindowSettings) -> Result<Self, Error> {
        Self::open(settings).map_err(Error::backend)
    }

    /// Does the work of [`Sdl2Backend::new`], with `SDL2`'s own messages as errors.
    ///
    /// # Arguments
    ///
    /// * `settings` - How to create the window.
    ///
    /// # Returns
    ///
    /// The backend.
    ///
    /// # Errors
    ///
    /// Returns the `SDL2` message if anything cannot be created.
    fn open(settings: &WindowSettings) -> Result<Self, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let mut builder = video.window(&settings.title, settings.size.w, settings.size.h);

        if settings.centered {
            builder.position_centered();
        }

        if settings.resizable {
            builder.resizable();
        }

        let window = builder.build().map_err(|e| e.to_string())?;

        #[cfg(target_os = "macos")]
        if std::env::var_os("SDL_RENDER_DRIVER").is_none() {
            sdl2::hint::set("SDL_RENDER_DRIVER", "opengl");
        }

        let mut canvas = window
            .into_canvas()
            .target_texture()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(to_sdl_color(Color::WHITE));
        canvas.clear();
        canvas.present();

        let events = sdl.event()?;
        let waker = if events.register_custom_event::<Wake>().is_ok() {
            let sender = Mutex::new(events.event_sender());
            let waker: Waker = Arc::new(move || {
                if let Ok(sender) = sender.lock() {
                    let _ = sender.push_custom_event(Wake);
                }
            });

            Some(waker)
        } else {
            None
        };

        Ok(Self {
            pump: sdl.event_pump()?,
            text_input: video.text_input(),
            clipboard: video.clipboard(),
            _sdl: sdl,
            canvas,
            images: ImageCache::new(),
            waker,
            textures: HashMap::new(),
            fonts: FontCache::new(),
            applied_mask: EventMask::ALL,
            errors: Vec::new(),
            reported: HashSet::new(),
        })
    }

    /// Saves what the window currently shows as an image.
    ///
    /// # Arguments
    ///
    /// * `path` - Where to save; the format follows the extension (`.png`).
    ///
    /// # Returns
    ///
    /// `Ok` once the file is written.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] with the `SDL2` message if the pixels cannot be read or the file
    /// cannot be written.
    pub fn save_screenshot(&mut self, path: &str) -> Result<(), Error> {
        let (w, h) = self
            .canvas
            .output_size()
            .map_err(|m| Error::image(path, m))?;
        let mut pixels = self
            .canvas
            .read_pixels(None, PixelFormatEnum::ABGR8888)
            .map_err(|m| Error::image(path, m))?;
        let surface = Surface::from_data(&mut pixels, w, h, w * 4, PixelFormatEnum::ABGR8888)
            .map_err(|m| Error::image(path, m))?;

        surface.save(path).map_err(|m| Error::image(path, m))
    }

    /// Queues an error for the engine, unless the same error was reported before.
    ///
    /// # Arguments
    ///
    /// * `error` - What failed.
    fn report(&mut self, error: Error) {
        if self.reported.insert(error.clone()) {
            self.errors.push(error);
        }
    }

    /// Returns the font cache the backend draws text with.
    ///
    /// # Returns
    ///
    /// The `FontCache`; its [`stats`](FontCache::stats) show how many fonts and glyphs it has
    /// loaded.
    pub fn fonts(&self) -> &FontCache {
        &self.fonts
    }
}

impl Backend for Sdl2Backend {
    fn cache_policy(&self) -> CachePolicy {
        Sdl2Painter::CACHE_POLICY
    }

    fn window_size(&self) -> Size {
        let (w, h) = self.canvas.window().size();

        Size::new(w, h)
    }

    fn resize_window(&mut self, size: Size) {
        let _ = self.canvas.window_mut().set_size(size.w, size.h);
    }

    fn set_event_mask(&mut self, mask: EventMask) {
        let before = types_to_disable(self.applied_mask);
        let after = types_to_disable(mask);

        for event_type in before.iter().filter(|t| !after.contains(t)) {
            self.pump.enable_event(*event_type);
        }

        for event_type in after.iter().filter(|t| !before.contains(t)) {
            self.pump.disable_event(*event_type);
        }

        self.applied_mask = mask;
    }

    fn set_text_input(&mut self, active: bool) {
        if active {
            self.text_input.start();
        } else {
            self.text_input.stop();
        }
    }

    fn poll_events(&mut self, wait: Option<Duration>) -> Vec<InputEvent> {
        let mut raw = Vec::new();

        if let Some(wait) = wait {
            let ms = wait.as_millis().min(u32::MAX as u128) as u32;

            raw.extend(self.pump.wait_event_timeout(ms));
        }

        raw.extend(self.pump.poll_iter());

        raw.into_iter().filter_map(to_input_event).collect()
    }

    fn set_fonts(&mut self, fonts: pushrod::font::FontRegistry) {
        self.fonts.set_fonts(fonts);
    }

    fn waker(&self) -> Option<Waker> {
        self.waker.clone()
    }

    fn measure(&mut self, measure: &mut dyn FnMut(&mut dyn Painter)) {
        let mut painter = Sdl2Painter::new(&mut self.canvas, &mut self.fonts);

        measure(&mut painter);
        // Measuring draws nothing, so any error it found is reported when the text is painted.
        let _ = painter.take_errors();
    }

    /// Failures (a texture that cannot be created, a drawing call that fails) skip that
    /// `Widget`'s painting and are reported through [`Backend::take_notices`].
    fn paint_widget(&mut self, id: WidgetId, size: Size, paint: &mut dyn FnMut(&mut dyn Painter)) {
        let store = self.textures.entry(id).or_default();

        if let Err(error) = store.create_or_resize_texture(&mut self.canvas, size) {
            self.report(error);

            return;
        }

        let Some(mut texture) = store.take() else {
            return;
        };
        let fonts = &mut self.fonts;
        let images = &mut self.images;
        let mut painted = Vec::new();
        let result = self
            .canvas
            .with_texture_canvas(&mut texture, |texture_canvas| {
                let mut painter = Sdl2Painter::new(texture_canvas, fonts).with_images(images);

                paint(&mut painter);
                painted = painter.take_errors();
            });

        // The texture goes back even when drawing failed, so it is not leaked.
        if let Some(store) = self.textures.get_mut(&id) {
            store.restore(texture);
        }

        for error in painted {
            self.report(error);
        }

        if let Err(error) = result {
            self.report(Error::texture(error.to_string()));
        }
    }

    fn begin_frame(&mut self, clear: Color) {
        self.canvas.set_draw_color(to_sdl_color(clear));
        self.canvas.clear();
    }

    /// A failed copy is reported through [`Backend::take_notices`] and the `Widget` is skipped.
    fn draw_widget(&mut self, id: WidgetId, source: Rect, dest: Rect) {
        if let Some(store) = self.textures.get_mut(&id)
            && let Some(texture) = store.get_optional_ref()
            && let Err(message) = self
                .canvas
                .copy(texture, to_sdl_rect(source), to_sdl_rect(dest))
        {
            self.report(Error::texture(message));
        }
    }

    fn present(&mut self) {
        self.canvas.present();
    }

    fn take_notices(&mut self) -> Vec<EngineEvent> {
        let errors = std::mem::take(&mut self.errors)
            .into_iter()
            .map(|error| EngineEvent::Error { error });

        self.fonts
            .take_notices()
            .into_iter()
            .map(|notice| match notice {
                FontNotice::FontLoaded {
                    font_name,
                    font_size,
                } => EngineEvent::FontLoaded {
                    font_name,
                    font_size,
                },
                FontNotice::AtlasPageAdded { pages } => EngineEvent::GlyphAtlasPageAdded { pages },
            })
            .chain(errors)
            .collect()
    }

    fn clipboard_text(&mut self) -> Option<String> {
        self.clipboard
            .clipboard_text()
            .ok()
            .filter(|t| !t.is_empty())
    }

    fn set_clipboard_text(&mut self, text: &str) {
        let _ = self.clipboard.set_clipboard_text(text);
    }

    fn open_url(&mut self, url: &str) -> Result<(), String> {
        sdl2::url::open_url(url).map_err(|e| e.to_string())
    }
}

/// Opens a window and runs the application until the window is closed or a callback calls
/// `shutdown`.
///
/// # Arguments
///
/// * `engine` - The engine holding the `Widget`s and callbacks.
/// * `settings` - How to create the window.
/// * `state` - The application state, passed to every callback.
///
/// # Returns
///
/// The application state, after the last callback has run.
///
/// # Errors
///
/// Returns [`Error::Backend`] if the window cannot be created (see [`Sdl2Backend::new`]).
/// Failures after that point do not stop the loop; they arrive as
/// [`EngineEvent::Error`](pushrod::event::EngineEvent::Error).
pub fn run<S>(mut engine: Engine<S>, settings: WindowSettings, state: S) -> Result<S, Error> {
    let mut backend = Sdl2Backend::new(&settings)?;

    Ok(engine.run(&mut backend, state))
}

/// Opens a window and starts an application-driven event loop, as an alternative to [`run`] and
/// callbacks.  Pass the result to [`Engine::next_event`].
///
/// # Arguments
///
/// * `engine` - The engine holding the `Widget`s.
/// * `settings` - How to create the window.
///
/// # Returns
///
/// The [`EventLoop`] to pass to [`Engine::next_event`].
///
/// # Errors
///
/// Returns [`Error::Backend`] if the window cannot be created (see [`Sdl2Backend::new`]).
pub fn event_loop<S>(engine: &mut Engine<S>, settings: WindowSettings) -> Result<EventLoop, Error> {
    let backend = Sdl2Backend::new(&settings)?;

    Ok(engine.event_loop(Box::new(backend)))
}
