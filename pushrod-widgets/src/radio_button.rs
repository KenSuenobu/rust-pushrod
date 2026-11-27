// RadioButton Widget
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

//! # RadioButton and RadioGroup
//!
//! A `RadioButton` is a circle followed by a label.  It has the core's
//! [`Behavior::EXCLUSIVE`]: the radio buttons that share a parent form one group, and when one
//! becomes checked the core clears the others, emitting `Toggled { id, checked: false }` for each
//! one it cleared.  The newly checked one emits `Toggled { id, checked: true }`.
//!
//! Clicking the circle **or its label**, or pressing Space while it has the keyboard focus,
//! checks it.  Checking one that is already checked does nothing: a radio button is cleared by
//! choosing another, never by clicking it again.  The arrow keys move the focus within the group
//! and check the member that receives it; Tab visits the group once, at its checked member.
//!
//! [`RadioGroup`] builds a column of labeled radio buttons in one call.  Ask which one is checked
//! with `ui.checked_in_group(parent)`.

use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::error::Result;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, Size, point, size};
use pushrod::input::Key;
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::properties::{
    Property, core_properties, expect_bool, expect_text, set_core_property, track,
};
use pushrod::registry::PropertyValue;
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore, WidgetId};

/// The width and height of the circle, in pixels.  Odd, so the circle has a center pixel.
pub const CIRCLE_SIZE: u32 = 15;

/// The gap between the circle and the label, in pixels.
const GAP: i32 = 6;

/// The radius of the dot that marks a checked radio button, in pixels.
const DOT_RADIUS: i32 = 3;

/// A radio button with a label.
///
/// See the [module documentation](self) for its behavior.
#[derive(Debug)]
pub struct RadioButton {
    core: WidgetCore,
    style: TextStyle,
    label: String,
    pressed: bool,
}

impl Widget for RadioButton {
    /// Returns the radio button's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the radio button's shared state for modification.
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

        list.push(Property::text("text", self.get_text()));
        list.push(Property::bool("checked", self.is_checked()));

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
            "text" => {
                let t = expect_text(name, value)?.to_string();

                track(self, |w| w.set_text(t))
            }
            "checked" => {
                let b = expect_bool(name, value)?;

                track(self, |w| w.set_checked(b))
            }
            _ => set_core_property(self, name, value),
        }
    }

    /// Draws the circle (white inside, black ring), its dot when checked, and the label.
    ///
    /// While pressed the circle is black and the dot is white.  Colors set with
    /// [`Widget::set_style`] for the current state override the label's color and the ring's
    /// color.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the radio button.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let style = self.style();
        let ring = style.border_color.unwrap_or(Color::BLACK);
        let foreground = style.foreground.unwrap_or(self.style.font_color);
        let font = style.font.as_ref().unwrap_or(&self.style);
        let circle = self.circle_bounds(size);

        paint::fill(p, style.background.unwrap_or(self.get_color()));

        let (inside, mark) = paint::pressed_colors(self.pressed, Color::WHITE, Color::BLACK);
        let radius = CIRCLE_SIZE as i32 / 2;
        let (cx, cy) = (circle.x + radius, circle.y + radius);

        Self::fill_disc(p, cx, cy, radius, ring);
        Self::fill_disc(p, cx, cy, radius - 1, inside);

        if self.is_checked() {
            Self::fill_disc(p, cx, cy, DOT_RADIUS, mark);
        }

        let label_x = circle.x + circle.w as i32 + GAP;

        if size.w as i32 > label_x {
            paint::text(
                p,
                Rect::new(label_x, 0, size.w - label_x as u32, size.h),
                font,
                foreground,
                &self.label,
                Truncate::End,
            );
        }
    }

    /// Feeds the click controller, checks on Space or when activated, and re-emits the pointer
    /// events.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the radio button.
    /// * `ctx` - The context used to emit `Toggled`.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the pointer events and the Space key press it handles;
    /// otherwise [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        match event {
            // An arrow key moved the group's focus here: the focused member is the checked one.
            Event::Activated { id } => {
                if *id == self.get_id() {
                    self.select(ctx);
                }

                return Propagation::Continue;
            }
            Event::KeyPressed {
                key: Key::Char(' '),
                repeat: false,
                ..
            } => {
                self.select(ctx);
                return Propagation::Stop;
            }
            _ => {}
        }

        self.feed_controllers(event, ctx);

        if matches!(
            event,
            Event::PointerPressed { .. }
                | Event::PointerReleased { .. }
                | Event::PointerEntered { .. }
                | Event::PointerExited { .. }
        ) {
            ctx.emit(event.clone());
            return Propagation::Stop;
        }

        Propagation::Continue
    }

    /// Follows the click controller: shows the pressed look, and checks on a click.
    ///
    /// # Arguments
    ///
    /// * `event` - What the controller reported.
    /// * `ctx` - The context used to emit `Toggled`.
    ///
    /// # Returns
    ///
    /// Always [`Propagation::Continue`].
    fn on_controller_event(&mut self, event: &ControllerEvent, ctx: &mut EventCtx) -> Propagation {
        match event {
            ControllerEvent::ClickPressed => self.set_pressed(true),
            ControllerEvent::ClickArmed(armed) => self.set_pressed(*armed),
            ControllerEvent::Clicked { .. } => {
                self.set_pressed(false);
                self.select(ctx);
            }
            ControllerEvent::ClickCancelled => self.set_pressed(false),
            _ => {}
        }

        Propagation::Continue
    }
}

impl RadioButton {
    /// Creates an unchecked radio button with a label.
    ///
    /// It is 150 by 20 pixels at the upper left-hand corner of its parent, can take keyboard
    /// focus, belongs to the radio group of its parent, and has a transparent background until
    /// set with `set_color`.
    ///
    /// ```rust
    /// use pushrod_widgets::radio_button::RadioButton;
    ///
    /// let medium = RadioButton::new("Medium").at(20, 20).checked(true);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `label` - The text after the circle.
    ///
    /// # Returns
    ///
    /// The new `RadioButton`.
    pub fn new(label: impl Into<String>) -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(150, 20));

        // Space arrives as a key press.  Set the mask first: adding a controller ORs in its own.
        core.set_event_mask(EventMask::KEYBOARD);
        core.add_controller(ClickController::new());

        let mut radio = Self {
            core,
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            label: label.into(),
            pressed: false,
        };

        radio.set_behavior(Behavior::FOCUSABLE | Behavior::EXCLUSIVE);
        radio
    }

    /// Sets the upper left-hand corner, relative to the parent `Widget`.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The radio button, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The radio button, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets the label font.
    ///
    /// # Arguments
    ///
    /// * `font` - The font handle.
    ///
    /// # Returns
    ///
    /// The radio button, for chaining.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Sets the label color.
    ///
    /// # Arguments
    ///
    /// * `color` - The label color.
    ///
    /// # Returns
    ///
    /// The radio button, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
        self
    }

    /// Starts the radio button checked or unchecked.  Check at most one button per group this
    /// way: the core clears the others only when a checked button emits `Toggled`.
    ///
    /// # Arguments
    ///
    /// * `checked` - The starting state.
    ///
    /// # Returns
    ///
    /// The radio button, for chaining.
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.set_checked(checked);
        self
    }

    /// Checks or unchecks this radio button.  Emits no event and does not clear its siblings:
    /// the core clears them when a checked button emits `Toggled`.
    ///
    /// # Arguments
    ///
    /// * `checked` - The new state.
    pub fn set_checked(&mut self, checked: bool) {
        if self.is_checked() != checked {
            let mut visual = self.state();

            visual.set(VisualState::CHECKED, checked);
            self.set_state(visual);
            self.set_invalidated(true);
        }
    }

    /// Returns whether the radio button is checked.
    ///
    /// # Returns
    ///
    /// `true` when checked.
    pub fn is_checked(&self) -> bool {
        self.state().contains(VisualState::CHECKED)
    }

    /// Sets the label, and invalidates the radio button.
    ///
    /// # Arguments
    ///
    /// * `label` - The new label.
    pub fn set_text(&mut self, label: impl Into<String>) {
        self.label = label.into();
        self.set_invalidated(true);
    }

    /// Returns the label.
    ///
    /// # Returns
    ///
    /// A copy of the label.
    pub fn get_text(&self) -> String {
        self.label.clone()
    }

    /// Returns whether the circle is drawn pressed (the mouse button is down on it).
    ///
    /// # Returns
    ///
    /// `true` while pressed.
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Checks this radio button as a click or Space would, and emits `Toggled { id, checked:
    /// true }`.  The core then clears the checked siblings.  Does nothing when already checked.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context used to emit `Toggled`.
    pub fn select(&mut self, ctx: &mut EventCtx) {
        if self.is_checked() {
            return;
        }

        self.set_checked(true);
        ctx.emit(Event::Toggled {
            id: self.get_id(),
            checked: true,
        });
    }

    /// The circle's rectangle: at the left edge, centered vertically.
    fn circle_bounds(&self, size: Size) -> Rect {
        let y = (size.h as i32 - CIRCLE_SIZE as i32) / 2;

        Rect::new(1, y.max(0), CIRCLE_SIZE, CIRCLE_SIZE)
    }

    /// Fills a disc one row at a time (the painter has no circle primitive).
    fn fill_disc(p: &mut dyn Painter, cx: i32, cy: i32, radius: i32, color: Color) {
        for dy in -radius..=radius {
            let half = (f64::from(radius * radius - dy * dy) + radius as f64)
                .sqrt()
                .floor() as i32;

            p.fill_rect(
                Rect::new(cx - half, cy + dy, (half * 2 + 1) as u32, 1),
                color,
            );
        }
    }

    /// Sets the pressed look, repainting only on a change.
    fn set_pressed(&mut self, pressed: bool) {
        if self.pressed != pressed {
            self.pressed = pressed;
            self.set_invalidated(true);
        }
    }
}

/// A column of labeled radio buttons built in one call.
///
/// ```rust
/// use pushrod_widgets::radio_button::RadioGroup;
///
/// let sizes = RadioGroup::new(["Small", "Medium", "Large"]).at(20, 40).selected(1);
/// ```
#[derive(Debug, Clone)]
pub struct RadioGroup {
    labels: Vec<String>,
    origin: (i32, i32),
    item: (u32, u32),
    gap: u32,
    selected: Option<usize>,
}

impl RadioGroup {
    /// Creates a group with one radio button per label.  The first is checked.
    ///
    /// Each button is 150 by 20 pixels with a 4 pixel gap, at the upper left-hand corner of
    /// the parent it is added to.
    ///
    /// # Arguments
    ///
    /// * `labels` - The text of each radio button, top to bottom.
    ///
    /// # Returns
    ///
    /// The new `RadioGroup`.
    pub fn new<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            origin: (0, 0),
            item: (150, 20),
            gap: 4,
            selected: Some(0),
        }
    }

    /// Sets the upper left-hand corner of the first button, relative to the parent.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The group, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.origin = (x, y);
        self
    }

    /// Sets the width and height of each button.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The group, for chaining.
    #[must_use]
    pub fn item_size(mut self, width: u32, height: u32) -> Self {
        self.item = (width, height);
        self
    }

    /// Sets the vertical gap between buttons.
    ///
    /// # Arguments
    ///
    /// * `gap` - The gap in pixels.
    ///
    /// # Returns
    ///
    /// The group, for chaining.
    #[must_use]
    pub fn gap(mut self, gap: u32) -> Self {
        self.gap = gap;
        self
    }

    /// Chooses which button starts checked.  An index past the last button checks none.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based position of the button to check.
    ///
    /// # Returns
    ///
    /// The group, for chaining.
    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// Starts with no button checked.
    ///
    /// # Returns
    ///
    /// The group, for chaining.
    #[must_use]
    pub fn none_selected(mut self) -> Self {
        self.selected = None;
        self
    }

    /// Adds the radio buttons as children of `parent`, so they form one group.
    ///
    /// # Arguments
    ///
    /// * `cache` - The widget tree to add to.
    /// * `parent` - The `Widget` that holds the group.
    ///
    /// # Returns
    ///
    /// The ID of each radio button, in label order.
    pub fn add_to(self, cache: &mut WidgetCache, parent: WidgetId) -> Vec<WidgetId> {
        let (x, y) = self.origin;
        let (width, height) = self.item;

        self.labels
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                let top = y + (i as u32 * (height + self.gap)) as i32;
                let radio = RadioButton::new(label)
                    .at(x, top)
                    .size(width, height)
                    .checked(self.selected == Some(i));

                cache.add_child(parent, Box::new(radio))
            })
            .collect()
    }
}
