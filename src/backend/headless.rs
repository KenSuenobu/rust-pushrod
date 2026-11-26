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
use crate::geometry::{Point, Rect, Size};
use crate::input::InputEvent;
use crate::paint::{CachePolicy, Painter, TextStyle};
use crate::widget::WidgetId;
use std::collections::VecDeque;
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
    /// The number of frames presented.
    pub frames: usize,
    /// The last mask the engine set.
    pub mask: Option<EventMask>,
    /// Whether text input is currently on.
    pub text_input: bool,
    /// The clipboard contents.
    pub clipboard: Option<String>,
    /// The URLs the engine asked to open.
    pub opened_urls: Vec<String>,
    /// The font registry the engine gave the backend when the run loop started.
    pub fonts: Option<FontRegistry>,
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

    /// Paints the `Widget` with a [`RecordingPainter`] and stores the calls in
    /// [`HeadlessBackend::painted`].
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`.
    /// * `_size` - The `Widget`'s size; not needed to record.
    /// * `paint` - Draws the `Widget`.
    fn paint_widget(&mut self, id: WidgetId, _size: Size, paint: &mut dyn FnMut(&mut dyn Painter)) {
        let mut painter = RecordingPainter::default();

        paint(&mut painter);
        self.painted.push((id, painter.calls));
    }

    /// Starts a frame; nothing to clear.
    ///
    /// # Arguments
    ///
    /// * `_clear` - The clear color; ignored.
    fn begin_frame(&mut self, _clear: Color) {}

    /// Records the draw in [`HeadlessBackend::draws`].
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget` drawn.
    /// * `bounds` - Where it was drawn.
    fn draw_widget(&mut self, id: WidgetId, bounds: Rect) {
        self.draws.push((id, bounds));
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
        b.draw_widget(3, Rect::new(0, 0, 4, 4));
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
