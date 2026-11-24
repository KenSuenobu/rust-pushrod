// Fonts and Text
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

//! # Fonts
//!
//! The `FontCache` loads TrueType fonts through SDL2_ttf and renders text into GPU textures for
//! the [`paint::text`](crate::paint::text) helper.  The `Engine` owns one `FontCache` and passes
//! it to every `Widget::paint` call.

use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::ttf::{FontStyle, Sdl2TtfContext};
use sdl2::video::Window;
use std::path::Path;

/// Loads fonts and renders text into textures.
///
/// The `Engine` owns one and passes it to [`Widget::paint`](crate::widget::Widget::paint).
pub struct FontCache {
    ttf_context: Sdl2TtfContext,
}

impl FontCache {
    /// Initializes SDL2_ttf and creates a font cache.
    ///
    /// # Returns
    ///
    /// The new `FontCache`.
    ///
    /// # Panics
    ///
    /// Panics if SDL2_ttf cannot be initialized.
    pub fn new() -> Self {
        Self {
            ttf_context: sdl2::ttf::init().map_err(|e| e.to_string()).unwrap(),
        }
    }

    /// Renders text into a new texture.
    ///
    /// The text is drawn with antialiasing and wrapped at `width` pixels.  The caller owns the
    /// returned texture and must destroy it when done (`unsafe { texture.destroy() }`), because
    /// `unsafe_textures` textures aren't freed when dropped.
    ///
    /// # Arguments
    ///
    /// * `c` - The canvas whose renderer creates the texture.
    /// * `font_name` - Path to the TrueType font file.
    /// * `font_size` - The font size in points.
    /// * `font_style` - The font style (normal, bold, italic, ...).
    /// * `font_color` - The text color.
    /// * `text` - The text to render.
    /// * `width` - The width in pixels at which to wrap the text.
    ///
    /// # Returns
    ///
    /// The texture, and its width and height in pixels.
    ///
    /// # Panics
    ///
    /// Panics if the font file cannot be loaded or the text cannot be rendered.
    #[allow(clippy::too_many_arguments)]
    pub fn render_text(
        &mut self,
        c: &mut Canvas<Window>,
        font_name: &str,
        font_size: u16,
        font_style: FontStyle,
        font_color: Color,
        text: &str,
        width: u32,
    ) -> (Texture, u32, u32) {
        let texture_creator = c.texture_creator();
        let mut font = self
            .ttf_context
            .load_font(Path::new(font_name), font_size)
            .unwrap();

        font.set_style(font_style);

        let surface = font
            .render(text)
            .blended_wrapped(font_color, width)
            .map_err(|e| e.to_string())
            .unwrap();
        let font_texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();

        let TextureQuery { width, height, .. } = font_texture.query();

        (font_texture, width, height)
    }
}

impl Default for FontCache {
    /// Creates a font cache; the same as [`FontCache::new`].
    ///
    /// # Returns
    ///
    /// The new `FontCache`.
    ///
    /// # Panics
    ///
    /// Panics if SDL2_ttf cannot be initialized.
    fn default() -> Self {
        Self::new()
    }
}
