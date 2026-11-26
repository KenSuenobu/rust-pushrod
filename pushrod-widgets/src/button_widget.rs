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

//! # ButtonWidget
//!
//! This is a simple system `Widget` that fills its background, displays a text message within its
//! bounds, and draws a border around its frame.  It generates a `Clicked { id, button, count }` event when
//! the left mouse button is pressed and released within its bounds.
//!
//! The button follows the same behavior as a GEM button on the Atari ST: pressing the mouse
//! button selects (inverts) it; dragging out of the bounds deselects it, and dragging back in
//! selects it again.  `Clicked` is only generated if the mouse button is released while the
//! button is still selected.

use pushrod::color::Color;
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::event::{Event, EventCtx, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::widget::{VisualState, Widget, WidgetCore};
use std::cell::Cell;

/// A push button with a text label.
///
/// Pressing the left mouse button selects (inverts) the button; dragging out of its bounds
/// deselects it and dragging back in selects it again.  Releasing the mouse button while the
/// button is selected emits `Clicked { id, button, count }`.
pub struct ButtonWidget {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
    border_width: u8,
    selected: bool,
    truncate: Truncate,
    truncated: Cell<bool>,
}

impl Widget for ButtonWidget {
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
        let (background, foreground) = if self.selected {
            (Color::BLACK, Color::WHITE)
        } else {
            (self.get_color(), self.style.font_color)
        };
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
            style.border_width.unwrap_or(1),
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
    /// - `ClickPressed`: selects (inverts) the button.
    /// - `ClickArmed`: selects or deselects it as the pointer re-enters or leaves while pressed.
    /// - `Clicked`: emits `Clicked { id, button, count }`, then deselects.
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
            ControllerEvent::ClickPressed => self.set_selected(true),
            ControllerEvent::ClickArmed(armed) => self.set_selected(*armed),
            ControllerEvent::Clicked { button, count } => {
                ctx.emit(Event::Clicked {
                    id: self.get_id(),
                    button: *button,
                    count: *count,
                });
                self.set_selected(false);
            }
            ControllerEvent::ClickCancelled => self.set_selected(false),
            _ => {}
        }

        Propagation::Continue
    }
}

impl ButtonWidget {
    /// Creates a button.  Its background is transparent until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent `Widget`.
    /// * `size` - The width and height in pixels.
    /// * `font` - The font handle: a registered family, size, and style.  `Font::default()` is
    ///   the engine's default font, which needs no files.
    /// * `font_color` - The label color.
    /// * `justification` - The horizontal alignment of the label.
    /// * `msg` - The label.
    /// * `border_width` - Horizontal inset of the label from the edges, in pixels (the frame itself is
    ///   always one pixel).
    ///
    /// # Returns
    ///
    /// The new `ButtonWidget`.
    pub fn new(
        origin: Point,
        size: Size,
        font: Font,
        font_color: Color,
        justification: TextJustify,
        msg: String,
        border_width: u8,
    ) -> Self {
        let mut core = WidgetCore::new(origin, size);

        // The click controller needs presses and releases to click, and hover to follow the
        // pointer in and out of the button's bounds while pressed.
        core.add_controller(ClickController::new());

        Self {
            core,
            style: TextStyle::new(font, font_color, justification),
            msg,
            border_width,
            selected: false,
            truncate: Truncate::End,
            truncated: Cell::new(false),
        }
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
}
