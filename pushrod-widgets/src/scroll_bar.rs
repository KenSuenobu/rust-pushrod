// ScrollBar Widget
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

//! # ScrollBar
//!
//! A vertical or horizontal scroll bar: two arrow buttons, a track, and a thumb whose length
//! shows how much of the content is visible.
//!
//! It is a [composite](pushrod::composite): its four parts are ordinary `Widget`s with their own
//! IDs, which an application can find and restyle like any other `Widget`.
//!
//! ```text
//!   ScrollBar (composite)
//!    ├── "track"  Frame        white, one-pixel black border
//!    ├── "thumb"  Frame        light gray; dark gray while dragged (its PRESSED style)
//!    ├── "up"     GlyphButton  ↑ (← when horizontal), activate-on-press with auto-repeat
//!    └── "down"   GlyphButton  ↓ (→ when horizontal), activate-on-press with auto-repeat
//! ```
//!
//! * **Value:** a number from `0.0` (top, or left) to `1.0` (bottom, or right): how far the
//!   content is scrolled through the part that does not fit.
//! * **Page size:** the visible fraction of the content (`visible / total`, from `0.0` to `1.0`).
//!   The thumb's length is that fraction of the track (never shorter than
//!   [`MIN_THUMB_LENGTH`] pixels).
//! * **Arrows:** pressing an arrow moves the value by the `step` at once, then repeats while the
//!   button is held (after 400 ms, every 100 ms; see [`GlyphButton`]).
//! * **Track:** pressing the track before or after the thumb moves one page that way: the
//!   content moves by the visible amount.
//! * **Thumb:** dragging the thumb moves the value with the pointer, even outside the bar.
//!
//! It emits `ValueChanged { id }` each time the value actually changes; read the new value with
//! [`ScrollBar::get_value`].  Changing the value with [`ScrollBar::set_value`] emits nothing.

use crate::frame::Frame;
use crate::glyph_button::GlyphButton;
use pushrod::appearance::StateStyle;
use pushrod::color::Color;
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::error::Result;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Rect, point, size};
use pushrod::input::MouseButton;
use pushrod::paint::{self, Painter};
use pushrod::properties::{
    Property, core_properties, expect_float, read_only, set_core_property, track,
};
use pushrod::registry::PropertyValue;
use pushrod::widget::{VisualState, Widget, WidgetCore, WidgetId};

/// The default thickness of a scroll bar (its width when vertical), in pixels.
pub const DEFAULT_THICKNESS: u32 = 16;

/// The default length of a scroll bar (its height when vertical), in pixels.
pub const DEFAULT_LENGTH: u32 = 200;

/// The shortest the thumb gets, in pixels, so it can always be grabbed.
pub const MIN_THUMB_LENGTH: u32 = 8;

/// The default amount an arrow moves the value.
pub const DEFAULT_STEP: f64 = 0.05;

/// The default page size (visible fraction of the content).
pub const DEFAULT_PAGE_SIZE: f64 = 0.25;

/// The thumb's color.
pub const THUMB_COLOR: Color = Color::RGB(200, 200, 200);

/// The thumb's color while it is being dragged (its `PRESSED` style).
pub const THUMB_DRAGGED_COLOR: Color = Color::RGB(96, 96, 96);

/// The gap between the track's edges and the thumb, in pixels (the border plus one pixel).
const THUMB_INSET: i32 = 2;

/// A scroll bar's parts, by role.  Get it with [`TypedParts::typed_parts`].
///
/// # Examples
///
/// ```rust
/// use pushrod::cache::WidgetCache;
/// use pushrod::composite::TypedParts;
/// use pushrod::geometry::size;
/// use pushrod::root_widget::RootWidget;
/// use pushrod_widgets::scroll_bar::ScrollBar;
///
/// let mut cache = WidgetCache::new();
/// cache.add(Box::new(RootWidget::new(size(100, 240))));
///
/// let bar = cache.add(Box::new(ScrollBar::vertical().at(10, 10)));
/// let parts = cache.get_as::<ScrollBar>(bar).unwrap().typed_parts();
///
/// assert_eq!(cache.part(bar, "thumb"), Some(parts.thumb));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollBarParts {
    /// The arrow that moves the value toward `0.0` (up, or left): a `GlyphButton`.
    pub up: WidgetId,
    /// The arrow that moves the value toward `1.0` (down, or right): a `GlyphButton`.
    pub down: WidgetId,
    /// The track the thumb runs in: a `Frame`.
    pub track: WidgetId,
    /// The thumb: a `Frame`.
    pub thumb: WidgetId,
}

/// A scroll bar with arrows, a track, and a proportional thumb.
///
/// See the [module documentation](self) for its behavior.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::scroll_bar::ScrollBar;
///
/// // 30 of 120 lines are visible, and the view starts a third of the way down.
/// let mut bar = ScrollBar::vertical().at(380, 10).size(16, 280).value(1.0 / 3.0);
/// bar.set_proportion(30.0, 120.0);
///
/// assert_eq!(bar.get_page_size(), 0.25);
/// ```
#[derive(Debug)]
pub struct ScrollBar {
    core: WidgetCore,
    parts: PartList,
    vertical: bool,
    value: f64,
    page_size: f64,
    step: f64,
    dragging: bool,
    /// While dragging, the pointer's distance from the start of the thumb, along the track.
    grab: i32,
}

impl Widget for ScrollBar {
    /// Returns the scroll bar's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the scroll bar's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Lists the widget's properties: the core properties, then its own.
    ///
    /// # Returns
    ///
    /// The properties.
    fn properties(&self) -> Vec<Property> {
        let mut list = core_properties(self);

        list.push(Property::float("value", self.get_value()));
        list.push(Property::float("step", self.get_step()));
        list.push(Property::float("page_size", self.get_page_size()));
        list.push(Property::bool("vertical", self.is_vertical()));

        list
    }

    /// Sets a property by name; the widget's own, or else a core property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed, `false` if the widget already had it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Property`](pushrod::error::Error::Property) if there is no such
    /// property, it is read only, or the value is the wrong kind or out of range.
    fn set_property(&mut self, name: &str, value: &PropertyValue) -> Result<bool> {
        match name {
            "value" => {
                let n = expect_float(name, value)?;

                track(self, |w| w.set_value(n))
            }
            "step" => {
                let n = expect_float(name, value)?;

                track(self, |w| w.set_step(n))
            }
            "page_size" => {
                let n = expect_float(name, value)?;

                track(self, |w| w.set_page_size(n))
            }
            "vertical" => Err(read_only(name)),
            _ => set_core_property(self, name, value),
        }
    }

    /// Fills the background (transparent unless set with `set_color`); the parts draw the rest.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the scroll bar.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.style().background.unwrap_or(self.get_color()));
    }

    /// Pages for a press on the track, and drags the thumb.  The track and thumb parts take no
    /// input themselves, so presses on them bubble up to the scroll bar.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the scroll bar.
    /// * `ctx` - The context used to emit `ValueChanged`, and to turn window coordinates into
    ///   the scroll bar's own.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the pointer events it handles; otherwise
    /// [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        match event {
            Event::PointerPressed {
                button: MouseButton::Left,
                position,
                ..
            } => {
                let (x, y) = (position.x - ctx.origin().x, position.y - ctx.origin().y);

                if !self.track_bounds().contains(x, y) {
                    return Propagation::Continue;
                }

                let at = self.along(x, y);
                let (thumb_start, _) = self.thumb_span();

                if self.thumb_bounds().contains(x, y) {
                    self.dragging = true;
                    self.grab = at - thumb_start;
                } else if at < thumb_start {
                    self.change_to(self.value - self.page_step(), ctx);
                } else {
                    self.change_to(self.value + self.page_step(), ctx);
                }

                Propagation::Stop
            }
            Event::PointerMoved { position, .. } if self.dragging => {
                let at = self.along(position.x - ctx.origin().x, position.y - ctx.origin().y);

                self.drag_thumb_to(at - self.grab, ctx);
                Propagation::Stop
            }
            Event::PointerReleased {
                button: MouseButton::Left,
                ..
            } if self.dragging => {
                self.dragging = false;
                Propagation::Stop
            }
            _ => Propagation::Continue,
        }
    }

    /// Returns the scroll bar as a composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    /// Returns the scroll bar as a mutable composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for ScrollBar {
    /// Creates the track, the thumb, and the two arrows, in that order (so the arrows are drawn
    /// over the ends of the track).
    ///
    /// # Arguments
    ///
    /// * `parts` - Adds the parts.
    fn build(&mut self, parts: &mut PartsBuilder) {
        let (back, forward) = if self.vertical {
            ('\u{2191}', '\u{2193}')
        } else {
            ('\u{2190}', '\u{2192}')
        };

        let mut track = Frame::new();
        track.set_color(Color::WHITE);
        parts.add("track", track);

        let mut thumb = Frame::new();
        thumb.set_color(THUMB_COLOR);
        thumb.set_style(
            VisualState::PRESSED,
            StateStyle {
                background: Some(THUMB_DRAGGED_COLOR),
                ..StateStyle::default()
            },
        );
        parts.add("thumb", thumb);

        for (name, glyph) in [("up", back), ("down", forward)] {
            let mut arrow = GlyphButton::new(glyph).activate_on_press();
            arrow.set_color(Color::WHITE);
            parts.add(name, arrow);
        }
    }

    /// Returns the scroll bar's parts.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list(&self) -> &PartList {
        &self.parts
    }

    /// Returns the scroll bar's parts for modification.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    /// Places the arrows at the two ends, the track between them (sharing their border lines),
    /// and the thumb inside the track at the current value.
    ///
    /// # Returns
    ///
    /// Each part and its rectangle.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let (length, _) = self.extent();
        let button = self.button_length();
        let placements = [
            ("track", self.track_bounds()),
            ("thumb", self.thumb_bounds()),
            ("up", self.span_rect(0, button)),
            ("down", self.span_rect(length - button, button)),
        ];

        placements
            .into_iter()
            .filter_map(|(name, rect)| self.parts.get(name).map(|id| (id, rect)))
            .collect()
    }

    /// Turns each `Clicked` from an arrow (the first press, and every repeat) into a step of the
    /// value.  Nothing happens while the scroll bar is disabled.
    ///
    /// # Arguments
    ///
    /// * `part` - The part that emitted the event.
    /// * `event` - The event.
    /// * `ctx` - Queues the scroll bar's `ValueChanged`.
    fn handle_part_event(&mut self, part: WidgetId, event: &Event, ctx: &mut EventCtx) {
        if !matches!(event, Event::Clicked { .. }) || !self.is_enabled() {
            return;
        }

        match self.parts.name_of(part) {
            Some("up") => self.change_to(self.value - self.step, ctx),
            Some("down") => self.change_to(self.value + self.step, ctx),
            _ => {}
        }
    }
}

impl TypedParts for ScrollBar {
    type Parts = ScrollBarParts;

    /// Returns the scroll bar's parts by role.
    ///
    /// # Returns
    ///
    /// The part IDs; before the scroll bar is added to a `WidgetCache` they are all the scroll
    /// bar's own ID.
    fn typed_parts(&self) -> ScrollBarParts {
        let find = |name: &str| self.parts.get(name).unwrap_or(self.get_id());

        ScrollBarParts {
            up: find("up"),
            down: find("down"),
            track: find("track"),
            thumb: find("thumb"),
        }
    }
}

impl ScrollBar {
    /// Creates a vertical scroll bar, [`DEFAULT_THICKNESS`] wide and [`DEFAULT_LENGTH`] tall, at
    /// the upper left-hand corner of its parent.
    ///
    /// The value starts at `0.0` (the top), the page size is [`DEFAULT_PAGE_SIZE`], and the step
    /// is [`DEFAULT_STEP`].
    ///
    /// # Returns
    ///
    /// The new `ScrollBar`.
    pub fn vertical() -> Self {
        Self::create(true, size(DEFAULT_THICKNESS, DEFAULT_LENGTH))
    }

    /// Creates a horizontal scroll bar, [`DEFAULT_LENGTH`] wide and [`DEFAULT_THICKNESS`] tall.
    /// Otherwise it is like [`ScrollBar::vertical`], with `0.0` at the left.
    ///
    /// # Returns
    ///
    /// The new `ScrollBar`.
    pub fn horizontal() -> Self {
        Self::create(false, size(DEFAULT_LENGTH, DEFAULT_THICKNESS))
    }

    /// Creates a scroll bar in either direction.
    fn create(vertical: bool, bounds: pushrod::geometry::Size) -> Self {
        let mut core = WidgetCore::new(point(0, 0), bounds);

        // Presses on the track and thumb bubble up from the parts; motion drags the thumb.
        core.set_event_mask(EventMask::POINTER_BUTTONS | EventMask::POINTER_MOTION);

        Self {
            core,
            parts: PartList::new(),
            vertical,
            value: 0.0,
            page_size: DEFAULT_PAGE_SIZE,
            step: DEFAULT_STEP,
            dragging: false,
            grab: 0,
        }
    }

    /// Sets the upper left-hand corner, relative to the parent.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The scroll bar, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height.  To resize the scroll bar after it is added, call
    /// `set_size`; the parts follow before the next frame.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The scroll bar, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Starts the scroll bar at a value (see [`ScrollBar::set_value`]).
    ///
    /// # Arguments
    ///
    /// * `value` - The starting value, from `0.0` to `1.0`.
    ///
    /// # Returns
    ///
    /// The scroll bar, for chaining.
    #[must_use]
    pub fn value(mut self, value: f64) -> Self {
        self.set_value(value);
        self
    }

    /// Sets the page size (see [`ScrollBar::set_page_size`]).
    ///
    /// # Arguments
    ///
    /// * `page_size` - The visible fraction of the content, from `0.0` to `1.0`.
    ///
    /// # Returns
    ///
    /// The scroll bar, for chaining.
    #[must_use]
    pub fn page_size(mut self, page_size: f64) -> Self {
        self.set_page_size(page_size);
        self
    }

    /// Sets how far an arrow moves the value (see [`ScrollBar::set_step`]).
    ///
    /// # Arguments
    ///
    /// * `step` - The amount, as a fraction of the whole range.
    ///
    /// # Returns
    ///
    /// The scroll bar, for chaining.
    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        self.set_step(step);
        self
    }

    /// Returns the current value.
    ///
    /// # Returns
    ///
    /// The value, from `0.0` (top, or left) to `1.0` (bottom, or right).
    pub fn get_value(&self) -> f64 {
        self.value
    }

    /// Sets the value, clamped to `0.0..=1.0` (not a number counts as `0.0`), and moves the
    /// thumb before the next frame.  Emits no event.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value.
    pub fn set_value(&mut self, value: f64) {
        let value = clamp_unit(value, 0.0);

        if value != self.value {
            self.value = value;
            self.request_arrange_parts();
        }
    }

    /// Returns the page size.
    ///
    /// # Returns
    ///
    /// The visible fraction of the content, from `0.0` to `1.0`.
    pub fn get_page_size(&self) -> f64 {
        self.page_size
    }

    /// Sets the page size: the visible fraction of the content, which sets the thumb's length
    /// and how far a press on the track moves.  It is clamped to `0.0..=1.0`; not a number
    /// counts as `1.0` (everything visible).  Emits no event.
    ///
    /// # Arguments
    ///
    /// * `page_size` - The new page size.
    pub fn set_page_size(&mut self, page_size: f64) {
        let page_size = clamp_unit(page_size, 1.0);

        if page_size != self.page_size {
            self.page_size = page_size;
            self.request_arrange_parts();
        }
    }

    /// Sets the page size from the amount of content that is visible and the total, for
    /// example 30 visible lines of 120.  A total that is not a positive number means everything
    /// is visible (`1.0`).
    ///
    /// # Arguments
    ///
    /// * `visible` - The amount visible.
    /// * `total` - The total amount.
    pub fn set_proportion(&mut self, visible: f64, total: f64) {
        if total.is_finite() && total > 0.0 {
            self.set_page_size(visible / total);
        } else {
            self.set_page_size(1.0);
        }
    }

    /// Returns how far an arrow moves the value.
    ///
    /// # Returns
    ///
    /// The step, as a fraction of the whole range.
    pub fn get_step(&self) -> f64 {
        self.step
    }

    /// Sets how far an arrow moves the value, as a fraction of the whole range.  A negative or
    /// non-finite step becomes `0.0` (the arrows do nothing).
    ///
    /// # Arguments
    ///
    /// * `step` - The new step.
    pub fn set_step(&mut self, step: f64) {
        self.step = if step.is_finite() && step > 0.0 {
            step.min(1.0)
        } else {
            0.0
        };
    }

    /// Returns how far a press on the track moves the value: one page, so that the content
    /// moves by the visible amount, and never less than the arrow step.
    ///
    /// # Returns
    ///
    /// `page_size / (1 - page_size)` (or `1.0` when everything is visible), at least the step.
    pub fn page_step(&self) -> f64 {
        let page = if self.page_size < 1.0 {
            self.page_size / (1.0 - self.page_size)
        } else {
            1.0
        };

        page.max(self.step)
    }

    /// Returns whether the scroll bar runs vertically.
    ///
    /// # Returns
    ///
    /// `true` for a vertical scroll bar.
    pub fn is_vertical(&self) -> bool {
        self.vertical
    }

    /// Returns whether the thumb is being dragged.
    ///
    /// # Returns
    ///
    /// `true` from the press on the thumb until the mouse button is released.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Returns the track's rectangle, in the scroll bar's own coordinates.  It runs between the
    /// arrows and shares their border lines.
    ///
    /// # Returns
    ///
    /// The rectangle.
    pub fn track_bounds(&self) -> Rect {
        let (start, length) = self.track_span();

        self.span_rect(start, length)
    }

    /// Returns the thumb's rectangle, in the scroll bar's own coordinates, for the current value
    /// and page size.
    ///
    /// # Returns
    ///
    /// The rectangle; empty when the track is too short to hold a thumb.
    pub fn thumb_bounds(&self) -> Rect {
        let (_, thickness) = self.extent();
        let (start, length) = self.thumb_span();
        let cross = (thickness - 2 * THUMB_INSET).max(0);

        self.oriented(start, THUMB_INSET, length, cross)
    }

    /// Returns the scroll bar's length along its direction and its thickness across it.
    fn extent(&self) -> (i32, i32) {
        let bounds = self.get_size();
        let (w, h) = (bounds.w as i32, bounds.h as i32);

        if self.vertical { (h, w) } else { (w, h) }
    }

    /// Returns the length of each arrow button: square, but at most half the bar.
    fn button_length(&self) -> i32 {
        let (length, thickness) = self.extent();

        thickness.min(length / 2).max(0)
    }

    /// Returns where the track starts and how long it is, along the bar.
    fn track_span(&self) -> (i32, i32) {
        let (length, _) = self.extent();
        let button = self.button_length();
        let start = (button - 1).max(0);
        let end = (length - button + 1).min(length);

        (start, (end - start).max(0))
    }

    /// Returns where the thumb's travel starts and how far the thumb can move, along the bar,
    /// with the thumb's length.
    fn travel(&self) -> (i32, i32, i32) {
        let (track_start, track_length) = self.track_span();
        let inner = (track_length - 2 * THUMB_INSET).max(0);
        let thumb = if inner == 0 {
            0
        } else {
            ((inner as f64 * self.page_size).round() as i32)
                .max(MIN_THUMB_LENGTH as i32)
                .min(inner)
        };

        (track_start + THUMB_INSET, inner - thumb, thumb)
    }

    /// Returns where the thumb starts and how long it is, along the bar.
    fn thumb_span(&self) -> (i32, i32) {
        let (start, travel, thumb) = self.travel();

        (start + (self.value * travel as f64).round() as i32, thumb)
    }

    /// Builds a rectangle that spans the bar's whole thickness.
    fn span_rect(&self, start: i32, length: i32) -> Rect {
        let (_, thickness) = self.extent();

        self.oriented(start, 0, length.max(0), thickness)
    }

    /// Builds a rectangle from positions along and across the bar.
    fn oriented(&self, along: i32, across: i32, length: i32, thickness: i32) -> Rect {
        let (length, thickness) = (length.max(0) as u32, thickness.max(0) as u32);

        if self.vertical {
            Rect::new(across, along, thickness, length)
        } else {
            Rect::new(along, across, length, thickness)
        }
    }

    /// Picks the coordinate along the bar out of a point in the bar's own coordinates.
    fn along(&self, x: i32, y: i32) -> i32 {
        if self.vertical { y } else { x }
    }

    /// Moves the thumb so it starts at `start` (along the bar), and sets the value to match.
    fn drag_thumb_to(&mut self, start: i32, ctx: &mut EventCtx) {
        let (travel_start, travel, _) = self.travel();

        if travel > 0 {
            self.change_to((start - travel_start) as f64 / travel as f64, ctx);
        }
    }

    /// Sets a new value (clamped) and, if it differs from the current one, moves the thumb and
    /// emits `ValueChanged`.
    fn change_to(&mut self, value: f64, ctx: &mut EventCtx) {
        let value = clamp_unit(value, self.value);

        if value != self.value {
            self.value = value;
            self.request_arrange_parts();
            ctx.emit(Event::ValueChanged { id: self.get_id() });
        }
    }
}

/// Clamps a number to `0.0..=1.0`.
///
/// # Arguments
///
/// * `value` - The number.
/// * `nan` - What not a number becomes.
///
/// # Returns
///
/// The clamped number.
fn clamp_unit(value: f64, nan: f64) -> f64 {
    if value.is_nan() {
        nan
    } else {
        value.clamp(0.0, 1.0)
    }
}
