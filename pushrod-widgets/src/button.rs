// Button Widget
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

//! # Button
//!
//! This is a simple system `Widget` that fills its background, displays a text message within its
//! bounds, and draws a border around its frame.  It generates a `Clicked { id, button, count }` event when
//! the left mouse button is pressed and released within its bounds.
//!
//! The button follows the same behavior as a GEM button on the Atari ST: pressing the mouse
//! button selects (inverts) it; dragging out of the bounds deselects it, and dragging back in
//! selects it again.  `Clicked` is only generated if the mouse button is released while the
//! button is still selected.
//!
//! # Behavior flags
//!
//! A button's other behaviors come from the core's [`Behavior`] flags, set with the builders below
//! or `Widget::set_behavior`:
//!
//! | Flag (builder) | Look | When activated | Emits |
//! |---|---|---|---|
//! | none | normal | release inside the button | `Clicked` |
//! | `DEFAULT_ACTION` ([`Button::default_action`]) | 2 px border ring | also by Return | `Clicked` |
//! | `CANCEL_ACTION` ([`Button::cancel_action`]) | normal | also by Escape | `Clicked` |
//! | `TOGGLE` ([`Button::toggle`]) | inverted while checked | each activation flips it | `Toggled` |
//! | `ACTIVATE_ON_PRESS` ([`Button::activate_on_press`]) | normal | on press, not on release | as above |
//! | `DISMISSES_DIALOG` ([`Button::dismisses_dialog`]) | normal | as above, then asks to close | also `EngineEvent::DialogDismissRequested` |

use pushrod::color::Color;
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::event::{EngineEvent, Event, EventCtx, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::MouseButton;
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore};
use std::cell::Cell;

/// A push button with a text label.
///
/// Pressing the left mouse button selects (inverts) the button; dragging out of its bounds
/// deselects it and dragging back in selects it again.  Releasing the mouse button while the
/// button is selected emits `Clicked { id, button, count }`.
#[derive(Debug)]
pub struct Button {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
    border_width: u8,
    selected: bool,
    checked: bool,
    truncate: Truncate,
    truncated: Cell<bool>,
}

impl Widget for Button {
    /// Returns the button's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the button's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws the button: a fill, the label inset by the border width, and a one-pixel frame.
    ///
    /// When selected, the colors are inverted (black background, white text).  Colors, border and
    /// font set with [`Widget::set_style`] for the current state override these defaults.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the button.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let style = self.style();
        let (background, foreground) = paint::pressed_colors(
            self.selected || self.checked,
            self.get_color(),
            self.style.font_color,
        );
        let (background, foreground) = (
            style.background.unwrap_or(background),
            style.foreground.unwrap_or(foreground),
        );
        let font = style.font.as_ref().unwrap_or(&self.style);
        let inset = self.border_width as u32 + 1;

        paint::fill(p, background);

        if size.w > inset * 2 {
            let drawn = paint::text(
                p,
                Rect::new(inset as i32, 0, size.w - inset * 2, size.h),
                font,
                foreground,
                &self.msg,
                self.truncate,
            );

            self.truncated.set(drawn.truncated);
        } else {
            self.truncated.set(false);
        }

        paint::border(
            p,
            size,
            style.border_color.unwrap_or(Color::BLACK),
            style.border_width.unwrap_or(self.ring_width()),
        );
    }

    /// Feeds the button's [`ClickController`] and re-emits the pointer events.
    ///
    /// The controller does the press, track, and release tracking and reports through
    /// [`Widget::on_controller_event`] (see below); the button only reacts to the results.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the button.
    /// * `ctx` - The context used to emit `Clicked`.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the pointer events the button handles, so they do not bubble to its
    /// parents; otherwise [`Propagation::Continue`].
    ///
    /// After the button has updated itself, it also re-emits `PointerPressed`, `PointerReleased`,
    /// `PointerEntered` and `PointerExited` so the application's `on_press`, `on_release` and
    /// `on_hover` callbacks run (each is emitted after `Clicked`, when there is one).
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        // Return or Escape activated this button (it is the form's default or cancel button).
        if let Event::Activated { id } = event {
            if *id == self.get_id() {
                self.activate(ctx, MouseButton::Left, 1);
            }

            return Propagation::Continue;
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

    /// Updates the selected state from the click controller, emitting `Clicked` on a click.
    ///
    /// - `ClickPressed`: selects (inverts) the button; with `ACTIVATE_ON_PRESS`, also activates it.
    /// - `ClickArmed`: selects or deselects it as the pointer re-enters or leaves while pressed.
    /// - `Clicked`: activates the button (a toggle flips and emits `Toggled`; any other button emits `Clicked`) unless it activates on press,
    ///   then deselects.
    /// - `ClickCancelled`: deselects.
    ///
    /// # Arguments
    ///
    /// * `event` - What the controller reported.
    /// * `ctx` - The context used to emit `Clicked`.
    ///
    /// # Returns
    ///
    /// Always [`Propagation::Continue`]; `handle_event` decides what stops.
    fn on_controller_event(&mut self, event: &ControllerEvent, ctx: &mut EventCtx) -> Propagation {
        match event {
            ControllerEvent::ClickPressed => {
                self.set_selected(true);

                if self.behavior().contains(Behavior::ACTIVATE_ON_PRESS) {
                    self.activate(ctx, MouseButton::Left, 1);
                }
            }
            ControllerEvent::ClickArmed(armed) => self.set_selected(*armed),
            ControllerEvent::Clicked { button, count } => {
                if !self.behavior().contains(Behavior::ACTIVATE_ON_PRESS) {
                    self.activate(ctx, *button, *count);
                }

                self.set_selected(false);
            }
            ControllerEvent::ClickCancelled => self.set_selected(false),
            _ => {}
        }

        Propagation::Continue
    }
}

impl Button {
    /// Creates a button with a label and sensible defaults.
    ///
    /// The button is 100 by 24 pixels at the upper left-hand corner of its parent, with the
    /// engine's default font, a black centered label, and a two pixel label inset.  Its background
    /// is transparent until set with `set_color`.  Change the defaults with the chained setters:
    ///
    /// ```rust
    /// use pushrod_widgets::button::Button;
    ///
    /// let ok = Button::new("OK").at(20, 60).size(120, 24);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `label` - The button's label.
    ///
    /// # Returns
    ///
    /// The new `Button`.
    pub fn new(label: impl Into<String>) -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(100, 24));

        // The click controller needs presses and releases to click, and hover to follow the
        // pointer in and out of the button's bounds while pressed.
        core.add_controller(ClickController::new());

        Self {
            core,
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Center),
            msg: label.into(),
            border_width: 2,
            selected: false,
            checked: false,
            truncate: Truncate::End,
            truncated: Cell::new(false),
        }
    }

    /// Makes this the form's default button: drawn with a 2 px border ring, and activated by
    /// Return (sets [`Behavior::DEFAULT_ACTION`]).
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn default_action(self) -> Self {
        self.with_behavior(Behavior::DEFAULT_ACTION)
    }

    /// Makes this the form's cancel button: activated by Escape (sets
    /// [`Behavior::CANCEL_ACTION`]).
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn cancel_action(self) -> Self {
        self.with_behavior(Behavior::CANCEL_ACTION)
    }

    /// Makes this a toggle button: each activation flips it between checked (drawn inverted) and
    /// unchecked, and emits `Toggled { id, checked }` instead of `Clicked` (sets
    /// [`Behavior::TOGGLE`]).
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn toggle(self) -> Self {
        self.with_behavior(Behavior::TOGGLE)
    }

    /// Makes the button activate when the mouse button goes down, not when it is released (sets
    /// [`Behavior::ACTIVATE_ON_PRESS`]).
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn activate_on_press(self) -> Self {
        self.with_behavior(Behavior::ACTIVATE_ON_PRESS)
    }

    /// Makes activating the button ask the enclosing modal dialog to close: it also emits
    /// `EngineEvent::DialogDismissRequested { id }` (sets [`Behavior::DISMISSES_DIALOG`]).
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn dismisses_dialog(self) -> Self {
        self.with_behavior(Behavior::DISMISSES_DIALOG)
    }

    /// Returns whether a toggle button is checked.
    ///
    /// # Returns
    ///
    /// `true` while the button is checked; always `false` for a button that is not a toggle.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Checks or unchecks a toggle button without emitting `Toggled`, and invalidates it.
    ///
    /// # Arguments
    ///
    /// * `checked` - The new state.
    pub fn set_checked(&mut self, checked: bool) {
        if self.checked != checked {
            let mut visual = self.state();

            visual.set(VisualState::CHECKED, checked);
            self.checked = checked;
            self.set_state(visual);
            self.set_invalidated(true);
        }
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
    /// The button, for chaining.
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
    /// The button, for chaining.
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
    /// The button, for chaining.
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
    /// The button, for chaining.
    #[must_use]
    pub fn font_color(mut self, color: Color) -> Self {
        self.style.font_color = color;
        self
    }

    /// Sets the label's horizontal alignment.
    ///
    /// # Arguments
    ///
    /// * `justification` - The alignment.
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn justify(mut self, justification: TextJustify) -> Self {
        self.style.justification = justification;
        self
    }

    /// Sets the label's horizontal inset.
    ///
    /// # Arguments
    ///
    /// * `width` - The inset in pixels.
    ///
    /// # Returns
    ///
    /// The button, for chaining.
    #[must_use]
    pub fn border_width(mut self, width: u8) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the label's horizontal inset, and invalidates the button.
    ///
    /// # Arguments
    ///
    /// * `width` - The inset in pixels.
    pub fn set_border_width(&mut self, width: u8) {
        self.border_width = width;
        self.set_invalidated(true);
    }

    /// Returns the label's horizontal inset.
    ///
    /// # Returns
    ///
    /// The inset in pixels.
    pub fn get_border_width(&self) -> u8 {
        self.border_width
    }

    /// Sets the label, and invalidates the button.
    ///
    /// # Arguments
    ///
    /// * `msg` - The new label.
    pub fn set_text(&mut self, msg: String) {
        self.msg = msg;
        self.set_invalidated(true);
    }

    /// Sets how a label that doesn't fit is shortened, and invalidates the button.
    ///
    /// # Arguments
    ///
    /// * `truncate` - The mode; the default is [`Truncate::End`].
    pub fn set_truncate(&mut self, truncate: Truncate) {
        if self.truncate != truncate {
            self.truncate = truncate;
            self.set_invalidated(true);
        }
    }

    /// Returns how a label that doesn't fit is shortened.
    ///
    /// # Returns
    ///
    /// The truncation mode.
    pub fn get_truncate(&self) -> Truncate {
        self.truncate
    }

    /// Returns whether the label was shortened the last time the button painted, so a tooltip can
    /// show the full label.
    ///
    /// # Returns
    ///
    /// `true` if the last paint drew a shortened label; `false` before the first paint.
    pub fn is_truncated(&self) -> bool {
        self.truncated.get()
    }

    /// Returns the label.
    ///
    /// # Returns
    ///
    /// A copy of the label.
    pub fn get_text(&self) -> String {
        self.msg.clone()
    }

    /// Returns whether the button is drawn selected (pressed, with the pointer inside).
    ///
    /// # Returns
    ///
    /// `true` while the button is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, state: bool) {
        if self.selected != state {
            let mut visual = self.state();

            visual.set(VisualState::PRESSED, state);
            self.selected = state;
            self.set_state(visual);
            self.set_invalidated(true);
        }
    }

    /// Adds a behavior flag to the ones already set.
    fn with_behavior(mut self, flag: Behavior) -> Self {
        let behavior = self.behavior() | flag;

        self.set_behavior(behavior);
        self
    }

    /// The border width used when the style sets none: 2 px for a default button, else 1 px.
    fn ring_width(&self) -> u8 {
        if self.behavior().contains(Behavior::DEFAULT_ACTION) {
            2
        } else {
            1
        }
    }

    /// Performs the button's action: flips a toggle button and emits `Toggled`, or emits
    /// `Clicked`; then, for a dialog-dismissing button, emits `DialogDismissRequested`.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context used to emit events.
    /// * `button` - The mouse button reported in `Clicked`.
    /// * `count` - The click count reported in `Clicked`.
    fn activate(&mut self, ctx: &mut EventCtx, button: MouseButton, count: u8) {
        let id = self.get_id();

        if self.behavior().contains(Behavior::TOGGLE) {
            self.set_checked(!self.checked);
            ctx.emit(Event::Toggled {
                id,
                checked: self.checked,
            });
        } else {
            ctx.emit(Event::Clicked { id, button, count });
        }

        if self.behavior().contains(Behavior::DISMISSES_DIALOG) {
            ctx.emit(Event::Engine(EngineEvent::DialogDismissRequested { id }));
        }
    }
}
