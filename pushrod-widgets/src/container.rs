// Pushrod Widgets
// Container
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

//! # Container
//!
//! A scrollable viewport onto a content area that has its own size and its own `0,0`.
//!
//! ```text
//! content (its own size and 0,0)               screen
//! +-----------------------------------+
//! |0,0                                |       container origin (anywhere)
//! |      +------------------+         |            +------------------+-+
//! |      | viewport         |  ---------------->   | visible content  |^|
//! |      | (scroll_offset   |         |            |                  |#|
//! |      |  = top-left)     |         |            +------------------+v+
//! |      +------------------+         |            |<|####          |>| |
//! |                                   |            +--------------------+
//! +-----------------------------------+
//! resize container -> viewport grows/shrinks; content size unchanged
//! move container   -> viewport moves on screen; content positions unchanged
//! ```
//!
//! It is a [composite](pushrod::composite) of four parts:
//!
//! ```text
//!   Container (composite)
//!    ├── "content"     Panel      the viewport; add the children to this part
//!    ├── "vertical"    ScrollBar  shown when the content is taller than the viewport
//!    ├── "horizontal"  ScrollBar  shown when the content is wider than the viewport
//!    └── "corner"      Frame      the square between the two bars, when both are shown
//! ```
//!
//! * **Children** are added to the `content` part ([`Container::content`]), at positions
//!   relative to the content's `0,0`, like the children of a form.
//! * **The container's frame is the viewport.**  Resizing the container changes how much of
//!   the content is visible; moving it moves the viewport on screen.  Neither moves a child
//!   within the content.
//! * **Panning** changes [`scroll_offset`](Container::scroll_offset), the content point shown
//!   at the viewport's upper left-hand corner.  It is always clamped to the content.  The
//!   scroll bars, the wheel, and (while the container has the focus) Page Up, Page Down, Home,
//!   End, and the arrow keys pan it; so do [`scroll_to`](Container::scroll_to),
//!   [`scroll_by`](Container::scroll_by), and [`ensure_visible`](Container::ensure_visible).
//! * **Drawing:** the content is never drawn into one texture, so it can be larger than the
//!   GPU's largest texture.  Each child keeps its own texture; a frame copies only the children
//!   that intersect the viewport, cropped to it.  Panning repaints nothing.
//!
//! It emits `ViewportChanged { id, offset }` whenever the viewport pans or is resized.

use crate::frame::Frame;
use crate::panel::Panel;
use crate::scroll_bar::{DEFAULT_THICKNESS, ScrollBar};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::event::{Event, EventCtx, EventMask, EventPhase, Propagation};
use pushrod::geometry::{Point, Rect, Size, point, size};
use pushrod::input::Key;
use pushrod::paint::{self, Painter};
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};

/// The default distance an arrow key or a scroll bar arrow pans, in pixels.
pub const DEFAULT_LINE_STEP: u32 = 20;

/// How many line steps one notch of the wheel pans.
pub const WHEEL_LINES: i32 = 3;

/// When a container shows its scroll bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollBars {
    /// Each bar is shown only while the content is larger than the viewport in its direction.
    #[default]
    Auto,
    /// Both bars are always shown.
    Always,
    /// No bar is shown.  The wheel, the keys, and code still pan the viewport.
    Never,
}

/// How a container's content gets its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentSize {
    /// The size given to [`Container::new`] or [`Container::set_content_size`].
    #[default]
    Fixed,
    /// The content is as large as its children's extent: it reaches the right edge of the
    /// right-most visible child and the bottom edge of the lowest one.
    FitChildren,
    /// The content is as wide as the viewport, so it never scrolls sideways: for long vertical
    /// forms.
    FollowWidth {
        /// `true` to make the content as tall as its children (as tall as its layout manager
        /// asks, if the content has one); `false` to keep the height given to
        /// [`Container::new`] or [`Container::set_content_size`].
        fit_height: bool,
    },
}

/// A container's parts, by role.  Get it with [`TypedParts::typed_parts`].
///
/// # Examples
///
/// ```rust
/// use pushrod::cache::WidgetCache;
/// use pushrod::composite::TypedParts;
/// use pushrod::geometry::{point, size};
/// use pushrod::root_widget::RootWidget;
/// use pushrod_widgets::container::Container;
///
/// let mut cache = WidgetCache::new();
/// cache.add(Box::new(RootWidget::new(size(320, 240))));
///
/// let id = cache.add(Box::new(Container::new(point(10, 10), size(200, 100), size(200, 600))));
/// let parts = cache.get_as::<Container>(id).unwrap().typed_parts();
///
/// assert_eq!(cache.part(id, "content"), Some(parts.content));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerParts {
    /// The viewport that holds the children: a `Panel`.  Add children to this part.
    pub content: WidgetId,
    /// The vertical scroll bar: a `ScrollBar`.
    pub vertical: WidgetId,
    /// The horizontal scroll bar: a `ScrollBar`.
    pub horizontal: WidgetId,
    /// The square between the two scroll bars: a `Frame`.
    pub corner: WidgetId,
}

/// The sizes a container works out from its frame, its content, and its scroll bar policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Geometry {
    /// The part of the frame that shows content: the frame less the scroll bars shown.
    viewport: Size,
    /// The content's size.
    content: Size,
    /// Whether the vertical scroll bar is shown.
    vertical: bool,
    /// Whether the horizontal scroll bar is shown.
    horizontal: bool,
}

impl Geometry {
    /// Returns the largest scroll offset: the content that does not fit in the viewport.
    ///
    /// # Returns
    ///
    /// The offset that shows the content's lower right-hand corner; `0` in a direction where
    /// the content fits.
    fn max_offset(&self) -> Point {
        point(
            self.content.w.saturating_sub(self.viewport.w) as i32,
            self.content.h.saturating_sub(self.viewport.h) as i32,
        )
    }

    /// Clamps a scroll offset to the content.
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset wanted.
    ///
    /// # Returns
    ///
    /// The nearest offset that keeps the viewport inside the content.
    fn clamp(&self, offset: Point) -> Point {
        let max = self.max_offset();

        point(offset.x.clamp(0, max.x), offset.y.clamp(0, max.y))
    }
}

/// A scrollable viewport onto content with its own size and `0,0`.
///
/// See the [module documentation](self) for its behavior.
///
/// # Examples
///
/// ```rust
/// use pushrod::geometry::{point, rect, size};
/// use pushrod_widgets::container::Container;
///
/// // A 200 by 100 viewport onto 150 by 600 of content.
/// let mut container = Container::new(point(10, 10), size(200, 100), size(150, 600));
///
/// container.scroll_to(point(0, 250));
///
/// // The content is taller than the container, so the vertical scroll bar is shown; it
/// // takes 16 pixels of the width.
/// assert_eq!(container.viewport(), rect(0, 250, 184, 100));
///
/// // The offset is clamped to the content.
/// container.scroll_by(0, 10_000);
/// assert_eq!(container.scroll_offset(), point(0, 500));
/// ```
#[derive(Debug)]
pub struct Container {
    core: WidgetCore,
    parts: PartList,
    /// The content size the application set; the size in effect comes from `geometry`.
    content_size: Size,
    mode: ContentSize,
    bars: ScrollBars,
    offset: Point,
    line_step: u32,
    /// The children's extent, as of the last time the parts were arranged.
    children_extent: Size,
    /// Whether the user moved each scroll bar since the parts were last arranged.
    vertical_moved: bool,
    horizontal_moved: bool,
    /// The offset and viewport size last reported with `ViewportChanged`.
    reported: Option<(Point, Size)>,
    /// The focused `Widget` as of the last time the parts were arranged.
    last_focused: Option<WidgetId>,
}

impl Widget for Container {
    /// Returns the container's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the container's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the background (transparent unless set with `set_color`); the parts and the
    /// children draw the rest.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the container.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.style().background.unwrap_or(self.get_color()));
    }

    /// Pans for the wheel, and for Page Up, Page Down, Home, End, and the arrow keys while the
    /// container itself has the keyboard focus.
    ///
    /// A wheel event that cannot pan any further (the viewport is at that end, or the content
    /// fits) is passed on, so a container around this one can scroll.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the container, or bubbling up from a child.
    /// * `ctx` - The context used to emit `ViewportChanged`.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for a wheel event that panned and for the keys it handles;
    /// otherwise [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        match event {
            Event::Scrolled { delta, .. } => {
                let step = (self.line_step as i32).saturating_mul(WHEEL_LINES);
                let target = point(
                    self.offset.x.saturating_add(delta.x.saturating_mul(step)),
                    self.offset.y.saturating_sub(delta.y.saturating_mul(step)),
                );

                if self.pan_to(target, ctx) {
                    Propagation::Stop
                } else {
                    Propagation::Continue
                }
            }
            Event::KeyPressed { key, .. } if ctx.phase() == EventPhase::Target => {
                let geometry = self.geometry();
                let max = geometry.max_offset();
                let line = self.line_step as i32;
                let Point { x, y } = self.offset;
                let target = match key {
                    Key::PageUp => point(x, y.saturating_sub(geometry.viewport.h as i32)),
                    Key::PageDown => point(x, y.saturating_add(geometry.viewport.h as i32)),
                    Key::Home => point(x, 0),
                    Key::End => point(x, max.y),
                    Key::Up => point(x, y.saturating_sub(line)),
                    Key::Down => point(x, y.saturating_add(line)),
                    Key::Left => point(x.saturating_sub(line), y),
                    Key::Right => point(x.saturating_add(line), y),
                    _ => return Propagation::Continue,
                };

                self.pan_to(target, ctx);
                Propagation::Stop
            }
            _ => Propagation::Continue,
        }
    }

    /// Asks for the parts to be arranged again, so the container follows its content: a child
    /// was added, moved, or resized (the content may have grown), or took the keyboard focus
    /// (it is scrolled into view).
    fn descendant_changed(&mut self) {
        self.request_arrange_parts();
    }

    /// Returns the container as a composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    /// Returns the container as a mutable composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Container {
    /// Creates the content viewport, the two scroll bars, and the corner, in that order (so the
    /// bars are drawn over the content).
    ///
    /// # Arguments
    ///
    /// * `parts` - Adds the parts.
    fn build(&mut self, parts: &mut PartsBuilder) {
        parts.add("content", Panel::new());
        parts.add("vertical", ScrollBar::vertical());
        parts.add("horizontal", ScrollBar::horizontal());

        let mut corner = Frame::new();

        corner.set_color(Color::WHITE);
        parts.add("corner", corner);
    }

    /// Returns the container's parts.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list(&self) -> &PartList {
        &self.parts
    }

    /// Returns the container's parts for modification.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    /// Brings the parts up to date: measures the children, follows a scroll bar the user moved,
    /// scrolls a newly focused child into view, clamps the offset, pans the content, sets each
    /// scroll bar's thumb and visibility, and emits `ViewportChanged` if the viewport panned or
    /// changed size since it was last reported.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the parts and the children.
    /// * `ctx` - Queues `ViewportChanged`.
    fn sync_parts(&mut self, cache: &mut WidgetCache, ctx: &mut EventCtx) {
        if self.parts.is_empty() {
            return;
        }

        let parts = self.typed_parts();

        self.children_extent = content_extent(cache, parts.content);
        self.follow_scroll_bars(cache, parts);
        self.reveal_focused(cache, parts.content);

        let geometry = self.geometry();

        self.offset = geometry.clamp(self.offset);

        let area_changed = cache.set_child_area(parts.content, Some(geometry.content));

        // A layout manager now arranges the children in the new area, which can change their
        // extent: look again once it has.
        if area_changed && cache.get(parts.content).is_some_and(|w| w.has_layout()) {
            self.request_arrange_parts();
        }

        cache.set_child_offset(parts.content, self.offset);

        let max = geometry.max_offset();

        self.update_scroll_bar(
            cache,
            parts.vertical,
            geometry.vertical,
            (geometry.viewport.h, geometry.content.h),
            (self.offset.y, max.y),
        );
        self.update_scroll_bar(
            cache,
            parts.horizontal,
            geometry.horizontal,
            (geometry.viewport.w, geometry.content.w),
            (self.offset.x, max.x),
        );
        set_hidden(
            cache,
            parts.corner,
            !(geometry.vertical && geometry.horizontal),
        );

        let state = (self.offset, geometry.viewport);

        if self.reported != Some(state) {
            // The first arrangement is the starting state, not a change.
            if self.reported.is_some() {
                ctx.emit(Event::ViewportChanged {
                    id: self.get_id(),
                    offset: self.offset,
                });
            }

            self.reported = Some(state);
        }
    }

    /// Places the content over the viewport, the scroll bars along the right and bottom edges,
    /// and the corner between them.
    ///
    /// # Returns
    ///
    /// Each part and its rectangle.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let geometry = self.geometry();
        let frame = self.get_size();
        let Size { w, h } = geometry.viewport;
        let thickness = DEFAULT_THICKNESS;
        let right = frame.w.saturating_sub(thickness) as i32;
        let bottom = frame.h.saturating_sub(thickness) as i32;
        let placements = [
            ("content", Rect::new(0, 0, w, h)),
            ("vertical", Rect::new(right, 0, thickness, h)),
            ("horizontal", Rect::new(0, bottom, w, thickness)),
            ("corner", Rect::new(right, bottom, thickness, thickness)),
        ];

        placements
            .into_iter()
            .filter_map(|(name, rect)| self.parts.get(name).map(|id| (id, rect)))
            .collect()
    }

    /// Notes that the user moved a scroll bar, and asks for the parts to be arranged; the
    /// content follows the bar, and `ViewportChanged` is emitted, before the next frame.
    ///
    /// # Arguments
    ///
    /// * `part` - The part that emitted the event.
    /// * `event` - The event.
    /// * `_ctx` - Not used.
    fn handle_part_event(&mut self, part: WidgetId, event: &Event, _ctx: &mut EventCtx) {
        if !matches!(event, Event::ValueChanged { .. }) {
            return;
        }

        match self.parts.name_of(part) {
            Some("vertical") => self.vertical_moved = true,
            Some("horizontal") => self.horizontal_moved = true,
            _ => return,
        }

        self.request_arrange_parts();
    }
}

impl TypedParts for Container {
    type Parts = ContainerParts;

    /// Returns the container's parts by role.
    ///
    /// # Returns
    ///
    /// The part IDs; before the container is added to a `WidgetCache` they are all the
    /// container's own ID.
    fn typed_parts(&self) -> ContainerParts {
        let find = |name: &str| self.parts.get(name).unwrap_or(self.get_id());

        ContainerParts {
            content: find("content"),
            vertical: find("vertical"),
            horizontal: find("horizontal"),
            corner: find("corner"),
        }
    }
}

impl Container {
    /// Creates a container.
    ///
    /// It shows its scroll bars when they are needed ([`ScrollBars::Auto`]), keeps the content
    /// at `content_size` ([`ContentSize::Fixed`]), starts at the content's upper left-hand
    /// corner, and can take the keyboard focus.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent.
    /// * `viewport_size` - The container's width and height on screen, including its scroll
    ///   bars.
    /// * `content_size` - The width and height of the content area.
    ///
    /// # Returns
    ///
    /// The new `Container`.
    pub fn new(origin: Point, viewport_size: Size, content_size: Size) -> Self {
        let mut core = WidgetCore::new(origin, viewport_size);

        // Wheel events bubble up from the children; keys arrive while the container is focused.
        core.set_event_mask(EventMask::SCROLL | EventMask::KEYBOARD);

        let mut container = Self {
            core,
            parts: PartList::new(),
            content_size,
            mode: ContentSize::Fixed,
            bars: ScrollBars::Auto,
            offset: point(0, 0),
            line_step: DEFAULT_LINE_STEP,
            children_extent: size(0, 0),
            vertical_moved: false,
            horizontal_moved: false,
            reported: None,
            last_focused: None,
        };

        container.set_behavior(Behavior::FOCUSABLE);
        container
    }

    /// Sets when the scroll bars are shown (see [`Container::set_scroll_bars`]).
    ///
    /// # Arguments
    ///
    /// * `bars` - The policy.
    ///
    /// # Returns
    ///
    /// The container, for chaining.
    #[must_use]
    pub fn scroll_bars(mut self, bars: ScrollBars) -> Self {
        self.set_scroll_bars(bars);
        self
    }

    /// Sets how the content gets its size (see [`Container::set_content_mode`]).
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode.
    ///
    /// # Returns
    ///
    /// The container, for chaining.
    #[must_use]
    pub fn content_mode(mut self, mode: ContentSize) -> Self {
        self.set_content_mode(mode);
        self
    }

    /// Sets how far an arrow key or a scroll bar arrow pans (see
    /// [`Container::set_line_step`]).
    ///
    /// # Arguments
    ///
    /// * `pixels` - The distance in pixels.
    ///
    /// # Returns
    ///
    /// The container, for chaining.
    #[must_use]
    pub fn line_step(mut self, pixels: u32) -> Self {
        self.set_line_step(pixels);
        self
    }

    /// Returns the part that holds the children.  Add each child to it, with an origin relative
    /// to the content's `0,0`:
    /// `engine.add_child_widget(container.content(), child)`.
    ///
    /// # Returns
    ///
    /// The ID of the `content` part; the container's own ID before it is added to a
    /// `WidgetCache`.
    pub fn content(&self) -> WidgetId {
        self.typed_parts().content
    }

    /// Returns the content point shown at the viewport's upper left-hand corner.
    ///
    /// # Returns
    ///
    /// The scroll offset, in content coordinates.
    pub fn scroll_offset(&self) -> Point {
        self.offset
    }

    /// Pans so that a content point is at the viewport's upper left-hand corner, or as near to
    /// it as the content allows.  If the offset changes, `ViewportChanged` is emitted before
    /// the next frame.
    ///
    /// # Arguments
    ///
    /// * `offset` - The content point to show at the upper left-hand corner.
    pub fn scroll_to(&mut self, offset: Point) {
        let offset = self.geometry().clamp(offset);

        if offset != self.offset {
            self.offset = offset;
            self.request_arrange_parts();
        }
    }

    /// Pans by a distance, stopping at the edges of the content (see
    /// [`Container::scroll_to`]).
    ///
    /// # Arguments
    ///
    /// * `dx` - How far to pan right, in pixels; negative pans left.
    /// * `dy` - How far to pan down, in pixels; negative pans up.
    pub fn scroll_by(&mut self, dx: i32, dy: i32) {
        self.scroll_to(point(
            self.offset.x.saturating_add(dx),
            self.offset.y.saturating_add(dy),
        ));
    }

    /// Pans as little as possible so that a rectangle of the content is visible.  Nothing
    /// happens if it already is.  A rectangle larger than the viewport is shown from its upper
    /// left-hand corner.
    ///
    /// The container calls this for a child that takes the keyboard focus.
    ///
    /// # Arguments
    ///
    /// * `rect` - The rectangle, in content coordinates.
    pub fn ensure_visible(&mut self, rect: Rect) {
        let viewport = self.geometry().viewport;
        let along = |start: i32, length: u32, offset: i32, visible: u32| {
            let end = start.saturating_add(length as i32);

            if start < offset || length >= visible {
                start
            } else if end > offset.saturating_add(visible as i32) {
                end.saturating_sub(visible as i32)
            } else {
                offset
            }
        };

        self.scroll_to(point(
            along(rect.x, rect.w, self.offset.x, viewport.w),
            along(rect.y, rect.h, self.offset.y, viewport.h),
        ));
    }

    /// Returns the visible content rectangle.
    ///
    /// # Returns
    ///
    /// The scroll offset, with the viewport's size: the container's size less the scroll bars
    /// that are shown.
    pub fn viewport(&self) -> Rect {
        let Size { w, h } = self.geometry().viewport;

        Rect::new(self.offset.x, self.offset.y, w, h)
    }

    /// Returns the content's size in effect: the fixed size, or the size worked out from the
    /// children or the viewport (see [`ContentSize`]) when the parts were last arranged.
    ///
    /// # Returns
    ///
    /// The content's width and height.
    pub fn content_size(&self) -> Size {
        self.geometry().content
    }

    /// Sets the content's size.  The scroll offset is clamped to the new size before the next
    /// frame.  [`ContentSize::FitChildren`] ignores it, and [`ContentSize::FollowWidth`] uses
    /// only its height (unless it fits the height too).
    ///
    /// # Arguments
    ///
    /// * `content_size` - The new width and height of the content area.
    pub fn set_content_size(&mut self, content_size: Size) {
        if content_size != self.content_size {
            self.content_size = content_size;
            self.request_arrange_parts();
        }
    }

    /// Returns how the content gets its size.
    ///
    /// # Returns
    ///
    /// The [`ContentSize`] mode.
    pub fn get_content_mode(&self) -> ContentSize {
        self.mode
    }

    /// Sets how the content gets its size.
    ///
    /// # Arguments
    ///
    /// * `mode` - The new mode.
    pub fn set_content_mode(&mut self, mode: ContentSize) {
        if mode != self.mode {
            self.mode = mode;
            self.request_arrange_parts();
        }
    }

    /// Returns when the scroll bars are shown.
    ///
    /// # Returns
    ///
    /// The [`ScrollBars`] policy.
    pub fn get_scroll_bars(&self) -> ScrollBars {
        self.bars
    }

    /// Sets when the scroll bars are shown.
    ///
    /// # Arguments
    ///
    /// * `bars` - The new policy.
    pub fn set_scroll_bars(&mut self, bars: ScrollBars) {
        if bars != self.bars {
            self.bars = bars;
            self.request_arrange_parts();
        }
    }

    /// Returns how far an arrow key or a scroll bar arrow pans.
    ///
    /// # Returns
    ///
    /// The distance in pixels.
    pub fn get_line_step(&self) -> u32 {
        self.line_step
    }

    /// Sets how far an arrow key or a scroll bar arrow pans.  One notch of the wheel pans
    /// [`WHEEL_LINES`] times as far.
    ///
    /// # Arguments
    ///
    /// * `pixels` - The distance in pixels.
    pub fn set_line_step(&mut self, pixels: u32) {
        if pixels != self.line_step {
            self.line_step = pixels;
            self.request_arrange_parts();
        }
    }

    /// Returns the content's size for a viewport of a given width.
    ///
    /// # Arguments
    ///
    /// * `viewport_width` - The width of the viewport, in pixels.
    ///
    /// # Returns
    ///
    /// The content's size under the current [`ContentSize`] mode.
    fn content_for(&self, viewport_width: u32) -> Size {
        match self.mode {
            ContentSize::Fixed => self.content_size,
            ContentSize::FitChildren => self.children_extent,
            ContentSize::FollowWidth { fit_height: true } => {
                size(viewport_width, self.children_extent.h)
            }
            ContentSize::FollowWidth { fit_height: false } => {
                size(viewport_width, self.content_size.h)
            }
        }
    }

    /// Works out the viewport, the content size, and which scroll bars are shown.
    ///
    /// With [`ScrollBars::Auto`], a bar that is shown takes space from the viewport, which can
    /// make the other bar necessary too.
    ///
    /// # Returns
    ///
    /// The [`Geometry`] for the container's current size and settings.
    fn geometry(&self) -> Geometry {
        let frame = self.get_size();
        let measure = |vertical: bool, horizontal: bool| {
            let viewport = size(
                frame
                    .w
                    .saturating_sub(if vertical { DEFAULT_THICKNESS } else { 0 }),
                frame
                    .h
                    .saturating_sub(if horizontal { DEFAULT_THICKNESS } else { 0 }),
            );

            (viewport, self.content_for(viewport.w))
        };
        let (vertical, horizontal) = match self.bars {
            ScrollBars::Always => (true, true),
            ScrollBars::Never => (false, false),
            ScrollBars::Auto => {
                let (viewport, content) = measure(false, false);
                let mut vertical = content.h > viewport.h;
                let mut horizontal = content.w > viewport.w;

                if vertical != horizontal {
                    let (viewport, content) = measure(vertical, horizontal);

                    vertical = vertical || content.h > viewport.h;
                    horizontal = horizontal || content.w > viewport.w;
                }

                (vertical, horizontal)
            }
        };
        let (viewport, content) = measure(vertical, horizontal);

        Geometry {
            viewport,
            content,
            vertical,
            horizontal,
        }
    }

    /// Pans from inside event handling, and emits `ViewportChanged` at once if the offset
    /// changed.
    ///
    /// # Arguments
    ///
    /// * `offset` - The content point to show at the upper left-hand corner.
    /// * `ctx` - The context used to emit the event.
    ///
    /// # Returns
    ///
    /// `true` if the offset changed.
    fn pan_to(&mut self, offset: Point, ctx: &mut EventCtx) -> bool {
        let before = self.offset;

        self.scroll_to(offset);

        if self.offset == before {
            return false;
        }

        self.reported = Some((self.offset, self.geometry().viewport));
        ctx.emit(Event::ViewportChanged {
            id: self.get_id(),
            offset: self.offset,
        });

        true
    }

    /// Moves the scroll offset to match each scroll bar the user moved since the parts were
    /// last arranged.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the scroll bars.
    /// * `parts` - The container's parts.
    fn follow_scroll_bars(&mut self, cache: &WidgetCache, parts: ContainerParts) {
        let max = self.geometry().max_offset();
        let position = |id: WidgetId, max: i32| {
            cache
                .get_as::<ScrollBar>(id)
                .map(|bar| (bar.get_value() * f64::from(max)).round() as i32)
        };

        if std::mem::take(&mut self.vertical_moved)
            && let Some(y) = position(parts.vertical, max.y)
        {
            self.offset.y = y;
        }

        if std::mem::take(&mut self.horizontal_moved)
            && let Some(x) = position(parts.horizontal, max.x)
        {
            self.offset.x = x;
        }
    }

    /// Scrolls the focused `Widget` into view, if the focus moved to one of the content's
    /// descendants since the parts were last arranged.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the children.
    /// * `content` - The ID of the `content` part.
    fn reveal_focused(&mut self, cache: &WidgetCache, content: WidgetId) {
        let focused = cache.focused();

        if focused == self.last_focused {
            return;
        }

        self.last_focused = focused;

        let Some(id) = focused else {
            return;
        };

        if !is_descendant(cache, id, content) {
            return;
        }

        let (Some(widget), Some(viewport)) = (cache.get(id), cache.get(content)) else {
            return;
        };

        // Window positions reflect the offset the content was last drawn with.
        let shown = viewport.child_offset();
        let at = cache.absolute_origin(id);
        let base = cache.absolute_origin(content);
        let bounds = widget.get_size();

        self.ensure_visible(Rect::new(
            at.x - base.x + shown.x,
            at.y - base.y + shown.y,
            bounds.w,
            bounds.h,
        ));
    }

    /// Shows or hides a scroll bar, and sets its thumb to match the viewport.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the scroll bar.
    /// * `id` - The scroll bar's ID.
    /// * `shown` - Whether the bar is shown.
    /// * `extent` - The visible and total lengths of the content along the bar.
    /// * `position` - The scroll offset along the bar, and the largest it can be.
    fn update_scroll_bar(
        &self,
        cache: &mut WidgetCache,
        id: WidgetId,
        shown: bool,
        extent: (u32, u32),
        position: (i32, i32),
    ) {
        set_hidden(cache, id, !shown);

        let Some(bar) = cache.get_as_mut::<ScrollBar>(id) else {
            return;
        };
        let (visible, total) = extent;
        let (offset, max) = position;

        bar.set_proportion(f64::from(visible), f64::from(total));

        if max <= 0 {
            bar.set_step(0.0);
            bar.set_value(0.0);
            return;
        }

        bar.set_step(f64::from(self.line_step) / f64::from(max));

        // A bar the user is dragging already shows this offset; leave its value alone so the
        // thumb does not jump by a rounding step under the pointer.
        if (bar.get_value() * f64::from(max)).round() as i32 != offset {
            bar.set_value(f64::from(offset) / f64::from(max));
        }
    }
}

/// Hides or shows a `Widget`, only if that changes it (hiding repaints and lays out again).
///
/// # Arguments
///
/// * `cache` - The cache holding the `Widget`.
/// * `id` - The `Widget`.
/// * `hidden` - `true` to hide it.
fn set_hidden(cache: &mut WidgetCache, id: WidgetId, hidden: bool) {
    if let Some(widget) = cache.get_mut(id)
        && widget.is_hidden() != hidden
    {
        widget.set_hidden(hidden);
    }
}

/// Returns whether a `Widget` is below another in the tree.
///
/// # Arguments
///
/// * `cache` - The cache holding both.
/// * `id` - The `Widget` to look for.
/// * `ancestor` - The `Widget` it may be below.
///
/// # Returns
///
/// `true` if `ancestor` is the parent of `id`, or its parent's parent, and so on.
fn is_descendant(cache: &WidgetCache, id: WidgetId, ancestor: WidgetId) -> bool {
    let mut current = id;

    while let Some(widget) = cache.get(current) {
        let parent = widget.get_parent();

        if parent == ancestor {
            return true;
        }

        if parent == current {
            return false;
        }

        current = parent;
    }

    false
}

/// Measures how much room the content's children need.
///
/// # Arguments
///
/// * `cache` - The cache holding the children.
/// * `content` - The ID of the `content` part.
///
/// # Returns
///
/// The preferred size its layout manager measured, if the content has one and it has been
/// measured; otherwise the right and bottom edges of the visible children that reach farthest.
fn content_extent(cache: &WidgetCache, content: WidgetId) -> Size {
    if cache.get(content).is_some_and(|w| w.has_layout())
        && let Some(hint) = cache.measured_hint(content)
    {
        return hint.preferred;
    }

    let mut extent = size(0, 0);

    for &id in cache.children_of(content) {
        let Some(child) = cache.get(id) else {
            continue;
        };

        if child.is_hidden() {
            continue;
        }

        let origin = child.get_origin();
        let bounds = child.get_size();
        let right = origin.x.saturating_add(bounds.w as i32).max(0) as u32;
        let bottom = origin.y.saturating_add(bounds.h as i32).max(0) as u32;

        extent = size(extent.w.max(right), extent.h.max(bottom));
    }

    extent
}
