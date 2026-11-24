// Text Widget
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

//! # TextWidget
//!
//! This is a simple system `Widget` that fills its background like the `BaseWidget`, and draws a
//! text message with a given font name, style, color, and message on top of it.

use pushrod::font::FontCache;
use pushrod::geometry::{Point, Size};
use pushrod::paint::{self, TextStyle};
use pushrod::widget::{Widget, WidgetCore};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::FontStyle;
use sdl2::video::Window;

pub use pushrod::paint::TextJustify;

/// A single string of text on a filled background.
pub struct TextWidget {
    core: WidgetCore,
    style: TextStyle,
    msg: String,
}

impl Widget for TextWidget {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the background, then draws the text.
    fn paint(&self, c: &mut Canvas<Window>, fc: &mut FontCache) {
        let size = self.get_size();

        paint::fill(c, self.get_color());
        paint::text(
            c,
            fc,
            Rect::new(0, 0, size.w, size.h),
            &self.style,
            self.style.font_color,
            &self.msg,
        );
    }
}

impl TextWidget {
    /// Creates a text widget at `origin` with the given `size`, font, style, color,
    /// justification, and text.
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
    ) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
            style: TextStyle::new(font_name, font_style, font_size, font_color, justification),
            msg,
        }
    }

    /// Sets the color of the text.
    pub fn set_font_color(&mut self, color: Color) {
        self.style.font_color = color;
        self.set_invalidated(true);
    }

    /// Sets the text to display.
    pub fn set_text(&mut self, msg: String) {
        self.msg = msg;
        self.set_invalidated(true);
    }

    /// Returns the color of the text.
    pub fn get_font_color(&self) -> Color {
        self.style.font_color
    }

    /// Returns the text being displayed.
    pub fn get_text(&self) -> String {
        self.msg.clone()
    }
}
