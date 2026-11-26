// CheckBox Widget
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

//! # CheckBox
//!
//! A square box followed by a label.  The box is empty when unchecked, holds a check mark when
//! checked, and holds a dash when it is *indeterminate* (a "mixed" state, for a box that
//! summarizes several items).
//!
//! Clicking the box **or its label**, or pressing Space while it has the keyboard focus, toggles
//! it and emits `Toggled { id, checked }`.  Activating an indeterminate box makes it checked.
//! While the mouse button is held down on it, the box is drawn solid black.
//!
//! The box is drawn by the widget itself: the core's `INDETERMINATE` overlay is a bar across the
//! whole widget, which would strike through the label, so the indeterminate state is kept here
//! and only `CHECKED` is mirrored into the widget's `VisualState`.

use pushrod::color::Color;
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, Size, point, size};
use pushrod::input::Key;
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore};

/// The width and height of the box, in pixels.
pub const BOX_SIZE: u32 = 14;

/// The gap between the box and the label, in pixels.
const GAP: i32 = 6;

/// A check box with a label and an optional indeterminate state.
///
/// See the [module documentation](self) for its behavior.
#[derive(Debug)]
pub struct CheckBox {
    core: WidgetCore,
    style: TextStyle,
    label: String,
    checked: bool,
    indeterminate: bool,
    pressed: bool,
}

impl Widget for CheckBox {
    /// Returns the check box's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the check box's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws the box (white inside, black frame), its mark, and the label.
    ///
    /// The mark is a check when checked, a dash when indeterminate, and nothing otherwise; while
    /// pressed the box is solid black.  Colors set with [`Widget::set_style`] for the current
    /// state override the label's color and the frame's color.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the check box.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let style = self.style();
        let frame = style.border_color.unwrap_or(Color::BLACK);
        let foreground = style.foreground.unwrap_or(self.style.font_color);
        let font = style.font.as_ref().unwrap_or(&self.style);
        let square = self.box_bounds(size);

        paint::fill(p, style.background.unwrap_or(self.get_color()));

        // Inside the box: white, or black while pressed, with the mark in the opposite color.
        let (inside, mark) = paint::pressed_colors(self.pressed, Color::WHITE, Color::BLACK);

        p.fill_rect(square, inside);
        p.rect(square, frame);

        if !self.pressed {
            if self.indeterminate {
                p.fill_rect(
                    Rect::new(
                        square.x + 3,
                        square.y + square.h as i32 / 2 - 1,
                        square.w - 6,
                        2,
                    ),
                    mark,
                );
            } else if self.checked {
                Self::paint_check(p, square, mark);
            }
        }

        let label_x = square.x + square.w as i32 + GAP;

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

    /// Feeds the click controller, toggles on Space, and re-emits the pointer events.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the check box.
    /// * `ctx` - The context used to emit `Toggled`.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the pointer events and the Space key press it handles;
    /// otherwise [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::KeyPressed {
            key: Key::Char(' '),
            repeat: false,
            ..
        } = event
        {
            self.toggle(ctx);
            return Propagation::Stop;
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

    /// Follows the click controller: shows the pressed look, and toggles on a click.
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
                self.toggle(ctx);
            }
            ControllerEvent::ClickCancelled => self.set_pressed(false),
            _ => {}
        }

        Propagation::Continue
    }
}

impl CheckBox {
    /// Creates an unchecked check box with a label.
    ///
    /// It is 150 by 20 pixels at the upper left-hand corner of its parent, can take keyboard
    /// focus, and has a transparent background until set with `set_color`.
    ///
    /// ```rust
    /// use pushrod_widgets::check_box::CheckBox;
    ///
    /// let send = CheckBox::new("Send email").at(20, 20).checked(true);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `label` - The text after the box.
    ///
    /// # Returns
    ///
    /// The new `CheckBox`.
    pub fn new(label: impl Into<String>) -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(150, 20));

        // Space arrives as a key press.  Set the mask first: adding a controller ORs in its own.
        core.set_event_mask(EventMask::KEYBOARD);
        core.add_controller(ClickController::new());

        let mut check_box = Self {
            core,
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            label: label.into(),
            checked: false,
            indeterminate: false,
            pressed: false,
        };

        check_box.set_behavior(Behavior::FOCUSABLE);
        check_box
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
    /// The check box, for chaining.
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
    /// The check box, for chaining.
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
    /// The check box, for chaining.
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
    /// The check box, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
        self
    }

    /// Starts the check box checked or unchecked.
    ///
    /// # Arguments
    ///
    /// * `checked` - The starting state.
    ///
    /// # Returns
    ///
    /// The check box, for chaining.
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.set_checked(checked);
        self
    }

    /// Starts the check box in the indeterminate (mixed) state.
    ///
    /// # Returns
    ///
    /// The check box, for chaining.
    #[must_use]
    pub fn indeterminate(mut self) -> Self {
        self.set_indeterminate(true);
        self
    }

    /// Checks or unchecks the box and clears the indeterminate state.  Emits no event.
    ///
    /// # Arguments
    ///
    /// * `checked` - The new state.
    pub fn set_checked(&mut self, checked: bool) {
        if self.checked != checked || self.indeterminate {
            self.checked = checked;
            self.indeterminate = false;
            self.mirror_checked();
            self.set_invalidated(true);
        }
    }

    /// Returns whether the box is checked.  An indeterminate box is not checked.
    ///
    /// # Returns
    ///
    /// `true` when checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Enters or leaves the indeterminate state.  Entering it unchecks the box.  Emits no event.
    ///
    /// # Arguments
    ///
    /// * `indeterminate` - `true` to show the mixed dash.
    pub fn set_indeterminate(&mut self, indeterminate: bool) {
        if self.indeterminate != indeterminate {
            self.indeterminate = indeterminate;

            if indeterminate {
                self.checked = false;
                self.mirror_checked();
            }

            self.set_invalidated(true);
        }
    }

    /// Returns whether the box is in the indeterminate state.
    ///
    /// # Returns
    ///
    /// `true` while it shows the mixed dash.
    pub fn is_indeterminate(&self) -> bool {
        self.indeterminate
    }

    /// Sets the label, and invalidates the check box.
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

    /// Returns whether the box is drawn pressed (the mouse button is down on it).
    ///
    /// # Returns
    ///
    /// `true` while pressed.
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Toggles the box as a click or Space would, and emits `Toggled { id, checked }`: an
    /// indeterminate box becomes checked; otherwise the state flips.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context used to emit `Toggled`.
    pub fn toggle(&mut self, ctx: &mut EventCtx) {
        let next = self.indeterminate || !self.checked;

        self.set_checked(next);
        ctx.emit(Event::Toggled {
            id: self.get_id(),
            checked: self.checked,
        });
    }

    /// The box's rectangle: at the left edge, centered vertically.
    fn box_bounds(&self, size: Size) -> Rect {
        let y = (size.h as i32 - BOX_SIZE as i32) / 2;

        Rect::new(1, y.max(0), BOX_SIZE, BOX_SIZE)
    }

    /// Draws a check mark (a short stroke down, a long stroke up) inside `square`.
    fn paint_check(p: &mut dyn Painter, square: Rect, color: Color) {
        let (x, y) = (square.x, square.y);

        for offset in 0..2 {
            p.line(
                point(x + 3, y + 7 + offset),
                point(x + 5, y + 9 + offset),
                color,
            );
            p.line(
                point(x + 5, y + 9 + offset),
                point(x + 10, y + 4 + offset),
                color,
            );
        }
    }

    /// Mirrors `checked` into the widget's visual state.
    fn mirror_checked(&mut self) {
        let mut visual = self.state();

        visual.set(VisualState::CHECKED, self.checked);
        self.set_state(visual);
    }

    /// Sets the pressed look, repainting only on a change.
    fn set_pressed(&mut self, pressed: bool) {
        if self.pressed != pressed {
            self.pressed = pressed;
            self.set_invalidated(true);
        }
    }
}
