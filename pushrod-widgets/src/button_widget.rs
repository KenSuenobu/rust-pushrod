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

use pushrod::color::{Color, FontStyle};
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Point, Rect, Size};
use pushrod::paint::{self, Painter, TextJustify, TextStyle};
use pushrod::widget::{Widget, WidgetCore};
use sdl2::mouse::MouseButton;

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
    pressed: bool,
    selected: bool,
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
    /// When selected, the colors are inverted (black background, white text).
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the button.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();
        let (background, foreground) = if self.selected {
            (Color::BLACK, Color::WHITE)
        } else {
            (self.get_color(), self.style.font_color)
        };
        let inset = self.border_width as u32 + 1;

        paint::fill(p, background);

        if size.w > inset * 2 {
            paint::text(
                p,
                Rect::new(inset as i32, 0, size.w - inset * 2, size.h),
                &self.style,
                foreground,
                &self.msg,
            );
        }

        paint::border(p, size, Color::BLACK, 1);
    }

    /// Tracks the left mouse button and emits `Clicked`.
    ///
    /// - Left button pressed (`PointerPressed`): the button is pressed and selected.
    /// - Pointer leaves or re-enters while pressed (`PointerExited`, `PointerEntered`): deselects
    ///   or reselects it.
    /// - Left button released (`PointerReleased`): if still selected, emits
    ///   `Clicked { id, button, count }`; then the button is released and deselected.
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
        self.track(event, ctx);

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
}

impl ButtonWidget {
    /// Updates the pressed and selected state for `event`, emitting `Clicked` on a click.
    fn track(&mut self, event: &Event, ctx: &mut EventCtx) {
        match event {
            Event::PointerPressed {
                button: MouseButton::Left,
                ..
            } => {
                self.pressed = true;
                self.set_selected(true);
            }

            Event::PointerReleased {
                button: MouseButton::Left,
                count,
                ..
            } => {
                if self.pressed && self.selected {
                    ctx.emit(Event::Clicked {
                        id: self.get_id(),
                        button: MouseButton::Left,
                        count: *count,
                    });
                }

                self.pressed = false;
                self.set_selected(false);
            }

            // While pressed, the button follows the mouse pointer in and out of its bounds.
            Event::PointerEntered { id } if self.pressed && *id == self.get_id() => {
                self.set_selected(true);
            }

            Event::PointerExited { id } if self.pressed && *id == self.get_id() => {
                self.set_selected(false);
            }

            _ => {}
        }
    }
}

impl ButtonWidget {
    /// Creates a button.  Its background is transparent until set with `set_color`.
    ///
    /// # Arguments
    ///
    /// * `origin` - The upper left-hand corner, relative to the parent `Widget`.
    /// * `size` - The width and height in pixels.
    /// * `font_name` - Path to the TrueType font file.
    /// * `font_style` - The font style (normal, bold, italic, ...).
    /// * `font_size` - The font size in points.
    /// * `font_color` - The label color.
    /// * `justification` - The horizontal alignment of the label.
    /// * `msg` - The label.
    /// * `border_width` - Horizontal inset of the label from the edges, in pixels (the frame itself is
    ///   always one pixel).
    ///
    /// # Returns
    ///
    /// The new `ButtonWidget`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: Point,
        size: Size,
        font_name: String,
        font_style: FontStyle,
        font_size: u16,
        font_color: Color,
        justification: TextJustify,
        msg: String,
        border_width: u8,
    ) -> Self {
        let mut core = WidgetCore::new(origin, size);

        // A button needs presses and releases to click, and hover to follow the pointer in and
        // out of its bounds while pressed.
        core.set_event_mask(EventMask::POINTER_BUTTONS | EventMask::HOVER);

        Self {
            core,
            style: TextStyle::new(font_name, font_style, font_size, font_color, justification),
            msg,
            border_width,
            pressed: false,
            selected: false,
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
            self.selected = state;
            self.set_invalidated(true);
        }
    }
}
