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
//! bounds, and draws a border around its frame.  It generates a `Clicked(id, clicks)` event when
//! the left mouse button is pressed and released within its bounds.
//!
//! The button follows the same behavior as a GEM button on the Atari ST: pressing the mouse
//! button selects (inverts) it; dragging out of the bounds deselects it, and dragging back in
//! selects it again.  `Clicked` is only generated if the mouse button is released while the
//! button is still selected.

use pushrod::event::{EventCtx, PushrodEvent};
use pushrod::font::FontCache;
use pushrod::geometry::{Point, Size};
use pushrod::paint::{self, TextJustify, TextStyle};
use pushrod::widget::{Widget, WidgetCore};
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

/// A push button with a text label.
///
/// Pressing the left mouse button selects (inverts) the button; dragging out of its bounds
/// deselects it and dragging back in selects it again.  Releasing the mouse button while the
/// button is selected emits `Clicked(id, clicks)`.
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
    /// * `c` - The canvas, targeting the button's texture.
    /// * `fc` - The font cache used to render the label.
    fn paint(&self, c: &mut Canvas<Window>, fc: &mut FontCache) {
        let size = self.get_size();
        let (background, foreground) = if self.selected {
            (Color::BLACK, Color::WHITE)
        } else {
            (self.get_color(), self.style.font_color)
        };
        let inset = self.border_width as u32 + 1;

        paint::fill(c, background);

        if size.w > inset * 2 {
            paint::text(
                c,
                fc,
                Rect::new(inset as i32, 0, size.w - inset * 2, size.h),
                &self.style,
                foreground,
                &self.msg,
            );
        }

        paint::border(c, size, Color::BLACK, 1);
    }

    /// Tracks the left mouse button and emits `Clicked`.
    ///
    /// - Left button pressed: the button is pressed and selected.
    /// - Pointer leaves or re-enters while pressed (`BoundsChange`): deselects or reselects it.
    /// - Left button released: if still selected, emits `Clicked(id, clicks)`; then the button is
    ///   released and deselected.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the button.
    /// * `ctx` - The context used to emit `Clicked`.
    fn handle_event(&mut self, event: &PushrodEvent, ctx: &mut EventCtx) {
        match event {
            PushrodEvent::SystemEvent(
                _,
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                },
            ) => {
                self.pressed = true;
                self.set_selected(true);
            }

            PushrodEvent::SystemEvent(
                _,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    clicks,
                    ..
                },
            ) => {
                if self.pressed && self.selected {
                    ctx.emit(PushrodEvent::Clicked(self.get_id(), *clicks));
                }

                self.pressed = false;
                self.set_selected(false);
            }

            // While pressed, the button follows the mouse pointer in and out of its bounds.
            PushrodEvent::BoundsChange(_, entered) if self.pressed => {
                self.set_selected(*entered == self.get_id());
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
        Self {
            core: WidgetCore::new(origin, size),
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
