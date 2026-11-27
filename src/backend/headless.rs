// Headless Backend
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

//! # Headless backend
//!
//! A [`Painter`] that draws nothing and records every call, and a [`HeadlessBackend`] that has no
//! window and records what the engine asks of it, so a `Widget`'s `paint` and a whole `Engine` can
//! be unit tested without a display.
//!
//! ```rust
//! use pushrod::backend::headless::{PaintCall, RecordingPainter};
//! use pushrod::color::Color;
//! use pushrod::paint;
//!
//! let mut p = RecordingPainter::default();
//! paint::fill(&mut p, Color::WHITE);
//! assert_eq!(p.calls, vec![PaintCall::Fill(Color::WHITE)]);
//! ```

use super::{Backend, Waker};
use crate::color::Color;
use crate::error::Error;
use crate::event::{EngineEvent, EventMask};
use crate::font::FontRegistry;
use crate::geometry::{Point, Rect, Size, sanitize_scale, to_physical_size};
use crate::input::InputEvent;
use crate::paint::{CachePolicy, Painter, TextStyle};
use crate::widget::WidgetId;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// One call made on a [`RecordingPainter`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaintCall {
    /// `fill(color)`
    Fill(Color),
    /// `fill_rect(bounds, color)`
    FillRect(Rect, Color),
    /// `rect(bounds, color)`
    Rect(Rect, Color),
    /// `line(from, to, color)`
    Line(Point, Point, Color),
    /// `text(bounds, msg, color)`
    Text(Rect, String, Color),
    /// `image(path, bounds)`
    Image(String, Rect),
    /// `image_part(path, source, bounds)`
    ImagePart(String, Option<Rect>, Rect),
    /// `dither(bounds, first, second)`
    Dither(Rect, Color, Color),
    /// `dither_overlay(bounds, color)`
    DitherOverlay(Rect, Color),
    /// `clip(bounds)`
    Clip(Option<Rect>),
}

/// A [`Painter`] that records its calls in [`calls`](RecordingPainter::calls).
#[derive(Debug, Default)]
pub struct RecordingPainter {
    /// Every call made, in order.
    pub calls: Vec<PaintCall>,
    /// The images this painter pretends exist, by path, with their pixel sizes.  A path not
    /// listed behaves like a missing file: [`Painter::image_size`] and [`Painter::image_part`]
    /// fail with [`Error::Image`].
    pub images: HashMap<String, Size>,
    /// The scale factor [`Painter::scale_factor`] reports; `0.0` (the default) reads as `1.0`.
    pub scale: f32,
}

impl Painter for RecordingPainter {
    /// Returns the policy of a recording painter.
    ///
    /// # Returns
    ///
    /// [`CachePolicy::DirectDraw`], since nothing is cached.
    fn cache_policy(&self) -> CachePolicy {
        CachePolicy::DirectDraw
    }

    /// Returns the recorded scale factor.
    ///
    /// # Returns
    ///
    /// [`RecordingPainter::scale`], or `1.0` if it is not a usable scale.
    fn scale_factor(&self) -> f32 {
        sanitize_scale(self.scale)
    }

    /// Records a fill.
    ///
    /// # Arguments
    ///
    /// * `color` - The fill color.
    fn fill(&mut self, color: Color) {
        self.calls.push(PaintCall::Fill(color));
    }

    /// Records a rectangle fill.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The fill color.
    fn fill_rect(&mut self, bounds: Rect, color: Color) {
        self.calls.push(PaintCall::FillRect(bounds, color));
    }

    /// Records a rectangle outline.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The line color.
    fn rect(&mut self, bounds: Rect, color: Color) {
        self.calls.push(PaintCall::Rect(bounds, color));
    }

    /// Records a line.
    ///
    /// # Arguments
    ///
    /// * `from` - The start point.
    /// * `to` - The end point.
    /// * `color` - The line color.
    fn line(&mut self, from: Point, to: Point, color: Color) {
        self.calls.push(PaintCall::Line(from, to, color));
    }

    /// Records text.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle to draw within.
    /// * `style` - The font (its size sets the recorded text height).
    /// * `color` - The text color.
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// A deterministic size: half the font size per character wide (at most the width of `bounds`),
    /// one font size tall.
    fn text(&mut self, bounds: Rect, style: &TextStyle, color: Color, msg: &str) -> Size {
        self.calls
            .push(PaintCall::Text(bounds, msg.to_string(), color));

        let width = (msg.chars().count() as u32 * (style.font.size() as u32 / 2)).min(bounds.w);

        Size::new(width, style.font.size() as u32)
    }

    /// Returns the line height: the font size.
    ///
    /// # Arguments
    ///
    /// * `style` - The font.
    ///
    /// # Returns
    ///
    /// The font size in pixels.
    fn line_height(&mut self, style: &TextStyle) -> u32 {
        style.font.size() as u32
    }

    /// Measures text the way [`text`](Painter::text) sizes it, without recording a call.
    ///
    /// # Arguments
    ///
    /// * `style` - The font (its size sets the width of each character).
    /// * `msg` - The text.
    ///
    /// # Returns
    ///
    /// Half the font size per character.
    fn text_width(&mut self, style: &TextStyle, msg: &str) -> u32 {
        msg.chars().count() as u32 * (style.font.size() as u32 / 2)
    }

    /// Records an image.
    ///
    /// # Arguments
    ///
    /// * `path` - The image path.
    /// * `bounds` - The rectangle.
    ///
    /// # Returns
    ///
    /// `Ok(())`, always.
    ///
    /// # Errors
    ///
    /// Never fails.
    fn image(&mut self, path: &str, bounds: Rect) -> Result<(), Error> {
        self.calls.push(PaintCall::Image(path.to_string(), bounds));
        Ok(())
    }

    /// Returns the size registered for an image in [`images`](RecordingPainter::images).
    ///
    /// # Arguments
    ///
    /// * `path` - The image path.
    ///
    /// # Returns
    ///
    /// The registered size.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] if the path is not registered (a missing file).
    fn image_size(&mut self, path: &str) -> Result<Size, Error> {
        self.images
            .get(path)
            .copied()
            .ok_or_else(|| Error::image(path, "file not found"))
    }

    /// Records part of an image.
    ///
    /// # Arguments
    ///
    /// * `path` - The image path.
    /// * `source` - The part of the image drawn, or `None` for all of it.
    /// * `bounds` - The rectangle drawn into.
    ///
    /// # Returns
    ///
    /// `Ok(())` for a registered image.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`] if the path is not registered (a missing file).
    fn image_part(&mut self, path: &str, source: Option<Rect>, bounds: Rect) -> Result<(), Error> {
        self.image_size(path)?;
        self.calls
            .push(PaintCall::ImagePart(path.to_string(), source, bounds));
        Ok(())
    }

    /// Records a dither.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `first` - The corner pixel color.
    /// * `second` - The other color.
    fn dither(&mut self, bounds: Rect, first: Color, second: Color) {
        self.calls.push(PaintCall::Dither(bounds, first, second));
    }

    /// Records a dither overlay.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangle.
    /// * `color` - The color to blend.
    fn dither_overlay(&mut self, bounds: Rect, color: Color) {
        self.calls.push(PaintCall::DitherOverlay(bounds, color));
    }

    /// Records a clip change.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The clip rectangle, or `None`.
    fn clip(&mut self, bounds: Option<Rect>) {
        self.calls.push(PaintCall::Clip(bounds));
    }
}

/// A [`Backend`] with no window: input is scripted, drawing is recorded, and nothing needs a
/// display.
///
/// Queue input with [`HeadlessBackend::push_event`]; after the engine runs, read
/// [`painted`](HeadlessBackend::painted), [`draws`](HeadlessBackend::draws) and
/// [`frames`](HeadlessBackend::frames).  Each `Widget` is painted with a [`RecordingPainter`].
///
/// # Examples
///
/// ```rust
/// use pushrod::backend::Backend;
/// use pushrod::backend::headless::HeadlessBackend;
/// use pushrod::geometry::size;
/// use pushrod::input::InputEvent;
///
/// let mut backend = HeadlessBackend::new(size(320, 200));
///
/// backend.push_event(InputEvent::Quit);
/// assert_eq!(backend.poll_events(None), vec![InputEvent::Quit]);
/// ```
#[derive(Debug, Default)]
pub struct HeadlessBackend {
    size: Size,
    script: VecDeque<InputEvent>,
    /// Every `Widget` paint, in order: the widget, and the calls it made.
    pub painted: Vec<(WidgetId, Vec<PaintCall>)>,
    /// Every `Widget` texture drawn into a frame, in order, with its bounds.
    pub draws: Vec<(WidgetId, Rect)>,
    /// The part of each texture in [`draws`](HeadlessBackend::draws) that was copied, in the same
    /// order, relative to the widget's upper left-hand corner.
    pub draw_sources: Vec<(WidgetId, Rect)>,
    /// The number of frames presented.
    pub frames: usize,
    /// The last mask the engine set.
    pub mask: Option<EventMask>,
    /// Whether text input is currently on.
    pub text_input: bool,
    /// The caret rectangle the engine last gave the backend for the candidate window.
    pub text_input_rect: Option<Rect>,
    /// The clipboard contents.
    pub clipboard: Option<String>,
    /// The URLs the engine asked to open.
    pub opened_urls: Vec<String>,
    /// The font registry the engine gave the backend when the run loop started.
    pub fonts: Option<FontRegistry>,
    /// The pretend window's scale factor; `0.0` (the default) reads as `1.0`.  Change it with
    /// [`HeadlessBackend::set_scale_factor`].
    scale: f32,
    /// The minimum window size the engine last set.
    pub minimum_size: Option<Size>,
    /// The physical size of each `Widget`'s pretend texture, as of its latest paint.  A removed
    /// `Widget`'s entry is dropped when its texture is released.
    pub texture_sizes: HashMap<WidgetId, Size>,
    /// Every `Widget` whose texture was released ([`Backend::release_widget`]), in order.
    pub released: Vec<WidgetId>,
    /// Every modal backdrop drawn ([`Backend::draw_backdrop`]), in order: how many entries
    /// [`draws`](HeadlessBackend::draws) had when it was drawn, the shade, and the blur.
    pub backdrops: Vec<(usize, Color, u8)>,
    /// Every rectangle filled into the frame ([`Backend::fill_frame_rect`]), in order.
    pub frame_fills: Vec<(Rect, Color)>,
}

impl HeadlessBackend {
    /// Creates a headless backend.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the pretend window.
    ///
    /// # Returns
    ///
    /// A new `HeadlessBackend` with no scripted input.
    pub fn new(size: Size) -> Self {
        Self {
            size,
            ..Self::default()
        }
    }

    /// Queues an input event for the next [`Backend::poll_events`].
    ///
    /// # Arguments
    ///
    /// * `event` - The event to deliver.
    pub fn push_event(&mut self, event: InputEvent) {
        self.script.push_back(event);
    }

    /// Sets the pretend window's scale factor and queues [`InputEvent::ScaleChanged`], as a real
    /// backend does when its window moves to a display with a different pixel density.
    ///
    /// # Arguments
    ///
    /// * `scale` - Physical pixels per logical pixel; an unusable value reads as `1.0`.
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale = sanitize_scale(scale);
        self.push_event(InputEvent::ScaleChanged(self.scale));
    }

    /// Creates a headless backend whose window starts at a scale factor, like a window opened on a
    /// high-DPI display.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the pretend window, in logical pixels.
    /// * `scale` - Physical pixels per logical pixel.
    ///
    /// # Returns
    ///
    /// A new `HeadlessBackend` with no scripted input.
    pub fn with_scale_factor(size: Size, scale: f32) -> Self {
        Self {
            scale: sanitize_scale(scale),
            ..Self::new(size)
        }
    }
}

impl Backend for HeadlessBackend {
    /// Records the font registry in [`HeadlessBackend::fonts`].
    ///
    /// # Arguments
    ///
    /// * `fonts` - The registry shared with the engine.
    fn set_fonts(&mut self, fonts: FontRegistry) {
        self.fonts = Some(fonts);
    }

    /// Returns the cache policy.
    ///
    /// # Returns
    ///
    /// [`CachePolicy::TextureCache`].
    fn cache_policy(&self) -> CachePolicy {
        CachePolicy::TextureCache
    }

    /// Returns the size of the pretend window.
    ///
    /// # Returns
    ///
    /// The size.
    fn window_size(&self) -> Size {
        self.size
    }

    /// Records a new window size.
    ///
    /// # Arguments
    ///
    /// * `size` - The new size.
    fn resize_window(&mut self, size: Size) {
        self.size = size;
    }

    /// Returns the pretend window's scale factor.
    ///
    /// # Returns
    ///
    /// The scale set by [`HeadlessBackend::with_scale_factor`] or
    /// [`HeadlessBackend::set_scale_factor`]; `1.0` by default.
    fn scale_factor(&self) -> f32 {
        sanitize_scale(self.scale)
    }

    /// Records the minimum size in [`HeadlessBackend::minimum_size`].
    ///
    /// # Arguments
    ///
    /// * `size` - The minimum size in logical pixels.
    fn set_minimum_window_size(&mut self, size: Size) {
        self.minimum_size = Some(size);
    }

    /// Records the event mask in [`HeadlessBackend::mask`].
    ///
    /// # Arguments
    ///
    /// * `mask` - The run loop's event mask.
    fn set_event_mask(&mut self, mask: EventMask) {
        self.mask = Some(mask);
    }

    /// Records the text input state in [`HeadlessBackend::text_input`].
    ///
    /// # Arguments
    ///
    /// * `active` - Whether text input is on.
    fn set_text_input(&mut self, active: bool) {
        self.text_input = active;
    }

    /// Records the caret rectangle.
    ///
    /// # Arguments
    ///
    /// * `rect` - The caret rectangle in window coordinates.
    fn set_text_input_rect(&mut self, rect: Rect) {
        self.text_input_rect = Some(rect);
    }

    /// Returns the scripted events, all at once; it never waits.  With an empty script the
    /// window is treated as closed, so a loop run against this backend always ends.
    ///
    /// # Arguments
    ///
    /// * `_wait` - Ignored.
    ///
    /// # Returns
    ///
    /// The scripted events, or [`InputEvent::Quit`] if there are none.
    fn poll_events(&mut self, _wait: Option<Duration>) -> Vec<InputEvent> {
        if self.script.is_empty() {
            return vec![InputEvent::Quit];
        }

        self.script.drain(..).collect()
    }

    /// A headless backend cannot be woken.
    ///
    /// # Returns
    ///
    /// `None`.
    fn waker(&self) -> Option<Waker> {
        None
    }

    /// Paints the `Widget` with a [`RecordingPainter`] at the window's scale factor, stores the
    /// calls in [`HeadlessBackend::painted`] and the physical texture size in
    /// [`HeadlessBackend::texture_sizes`].
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`.
    /// * `size` - The `Widget`'s size in logical pixels.
    /// * `paint` - Draws the `Widget`.
    fn paint_widget(&mut self, id: WidgetId, size: Size, paint: &mut dyn FnMut(&mut dyn Painter)) {
        let scale = self.scale_factor();
        let mut painter = RecordingPainter {
            scale,
            ..RecordingPainter::default()
        };

        self.texture_sizes.insert(id, to_physical_size(size, scale));
        paint(&mut painter);
        self.painted.push((id, painter.calls));
    }

    /// Drops the `Widget`'s pretend texture and records it in [`HeadlessBackend::released`].
    ///
    /// # Arguments
    ///
    /// * `id` - The removed `Widget`.
    fn release_widget(&mut self, id: WidgetId) {
        self.texture_sizes.remove(&id);
        self.released.push(id);
    }

    /// Measures with a [`RecordingPainter`], whose text sizes are deterministic.
    ///
    /// # Arguments
    ///
    /// * `measure` - Receives the painter.
    fn measure(&mut self, measure: &mut dyn FnMut(&mut dyn Painter)) {
        measure(&mut RecordingPainter {
            scale: self.scale_factor(),
            ..RecordingPainter::default()
        });
    }

    /// Starts a frame; nothing to clear.
    ///
    /// # Arguments
    ///
    /// * `_clear` - The clear color; ignored.
    fn begin_frame(&mut self, _clear: Color) {}

    /// Records the draw in [`HeadlessBackend::draws`] and [`HeadlessBackend::draw_sources`].
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` drawn.
    /// * `source` - The part of its texture copied.
    /// * `dest` - Where it was drawn.
    fn draw_widget(&mut self, id: WidgetId, source: Rect, dest: Rect) {
        self.draw_sources.push((id, source));
        self.draws.push((id, dest));
    }

    /// Records the backdrop in [`HeadlessBackend::backdrops`], with the number of draws before
    /// it, so tests can check what lies beneath each dialog.
    ///
    /// # Arguments
    ///
    /// * `shade` - The color that would be blended over the frame.
    /// * `blur` - How many halvings the frame would be softened by.
    fn draw_backdrop(&mut self, shade: Color, blur: u8) {
        self.backdrops.push((self.draws.len(), shade, blur));
    }

    /// Records the fill in [`HeadlessBackend::frame_fills`].
    ///
    /// # Arguments
    ///
    /// * `rect` - The area, in logical window coordinates.
    /// * `color` - The fill color.
    fn fill_frame_rect(&mut self, rect: Rect, color: Color) {
        self.frame_fills.push((rect, color));
    }

    /// Counts a presented frame in [`HeadlessBackend::frames`].
    fn present(&mut self) {
        self.frames += 1;
    }

    /// A headless backend loads no fonts, so it has no notices.
    ///
    /// # Returns
    ///
    /// An empty list.
    fn take_notices(&mut self) -> Vec<EngineEvent> {
        Vec::new()
    }

    /// Reads the stored clipboard text.
    ///
    /// # Returns
    ///
    /// The text, or `None` if nothing was stored.
    fn clipboard_text(&mut self) -> Option<String> {
        self.clipboard.clone()
    }

    /// Stores clipboard text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to store.
    fn set_clipboard_text(&mut self, text: &str) {
        self.clipboard = Some(text.to_string());
    }

    /// Records the URL in [`HeadlessBackend::opened_urls`] without opening anything.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL.
    ///
    /// # Returns
    ///
    /// Always `Ok`.
    ///
    /// # Errors
    ///
    /// Never fails.
    fn open_url(&mut self, url: &str) -> Result<(), String> {
        self.opened_urls.push(url.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::FontStyle;
    use crate::font::Font;

    #[test]
    fn the_engines_font_registry_is_shared_with_the_backend() {
        let engine: crate::engine::Engine = crate::engine::Engine::new(Size::new(10, 10), 30);
        let mut backend = HeadlessBackend::new(Size::new(10, 10));

        engine.fonts().load_bytes("Mem", vec![0, 1, 0, 0]).unwrap();
        backend.set_fonts(engine.fonts().clone());

        let shared = backend.fonts.as_ref().unwrap();
        assert_eq!(shared.families(), engine.fonts().families());
        assert!(shared.families().contains(&String::from("Mem")));
    }
    use crate::paint::TextJustify;

    #[test]
    fn scale_factor_sizes_textures_and_reaches_the_painter() {
        let mut backend = HeadlessBackend::with_scale_factor(Size::new(10, 10), 2.0);
        let mut seen = 0.0;

        backend.paint_widget(4, Size::new(30, 7), &mut |p| seen = p.scale_factor());

        assert_eq!(backend.scale_factor(), 2.0);
        assert_eq!(seen, 2.0);
        assert_eq!(backend.texture_sizes[&4], Size::new(60, 14));

        backend.set_scale_factor(1.5);
        assert_eq!(
            backend.poll_events(None),
            vec![InputEvent::ScaleChanged(1.5)]
        );
        backend.set_minimum_window_size(Size::new(5, 6));
        assert_eq!(backend.minimum_size, Some(Size::new(5, 6)));
        assert_eq!(RecordingPainter::default().scale_factor(), 1.0);
    }

    #[test]
    fn records_every_call_in_order() {
        let mut p = RecordingPainter::default();
        let r = Rect::new(1, 2, 3, 4);

        p.fill(Color::BLACK);
        p.fill_rect(r, Color::RED);
        p.line(Point::new(0, 0), Point::new(5, 5), Color::BLUE);
        p.dither(r, Color::BLACK, Color::WHITE);
        p.clip(Some(r));
        p.clip(None);
        assert!(p.image("a.png", r).is_ok());

        assert_eq!(p.calls.len(), 7);
        assert_eq!(p.calls[0], PaintCall::Fill(Color::BLACK));
        assert_eq!(p.calls[5], PaintCall::Clip(None));
        assert_eq!(p.calls[6], PaintCall::Image(String::from("a.png"), r));
    }

    #[test]
    fn registered_images_have_a_size_and_record_parts() {
        let mut p = RecordingPainter::default();
        let (src, dst) = (Rect::new(0, 0, 2, 2), Rect::new(5, 5, 4, 4));

        p.images.insert("a.png".to_string(), Size::new(8, 6));

        assert_eq!(p.image_size("a.png").unwrap(), Size::new(8, 6));
        assert!(p.image_part("a.png", Some(src), dst).is_ok());
        assert_eq!(
            p.calls,
            vec![PaintCall::ImagePart("a.png".into(), Some(src), dst)]
        );
    }

    #[test]
    fn unregistered_images_fail_like_missing_files() {
        let mut p = RecordingPainter::default();

        assert!(matches!(p.image_size("x.png"), Err(Error::Image { .. })));
        assert!(p.image_part("x.png", None, Rect::new(0, 0, 1, 1)).is_err());
        assert!(p.calls.is_empty());
    }

    #[test]
    fn text_size_is_deterministic_and_clamped() {
        let mut p = RecordingPainter::default();
        let style = TextStyle::new(
            Font::new("f", 10, FontStyle::NORMAL),
            Color::BLACK,
            TextJustify::Left,
        );

        assert_eq!(
            p.text(Rect::new(0, 0, 100, 20), &style, Color::BLACK, "abcd"),
            Size::new(20, 10)
        );
        assert_eq!(
            p.text(Rect::new(0, 0, 8, 20), &style, Color::BLACK, "abcd"),
            Size::new(8, 10)
        );
    }

    #[test]
    fn policy_is_direct_draw() {
        assert_eq!(
            RecordingPainter::default().cache_policy(),
            CachePolicy::DirectDraw
        );
    }

    #[test]
    fn headless_backend_scripts_input_and_records_drawing() {
        let mut b = HeadlessBackend::new(Size::new(10, 10));

        b.push_event(InputEvent::Exposed);
        assert_eq!(b.poll_events(None), vec![InputEvent::Exposed]);
        assert_eq!(b.poll_events(None), vec![InputEvent::Quit]);

        b.paint_widget(3, Size::new(4, 4), &mut |p| p.fill(Color::RED));
        b.draw_widget(3, Rect::new(0, 0, 4, 4), Rect::new(0, 0, 4, 4));
        b.present();
        b.set_clipboard_text("hi");

        assert_eq!(b.painted, vec![(3, vec![PaintCall::Fill(Color::RED)])]);
        assert_eq!(b.draws.len(), 1);
        assert_eq!(b.frames, 1);
        assert_eq!(b.clipboard_text().as_deref(), Some("hi"));
        assert!(b.open_url("https://example.com").is_ok());
        assert_eq!(b.opened_urls, vec!["https://example.com"]);
    }
}
