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

use std::path::Path;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::ttf::{FontStyle, Sdl2TtfContext};
use sdl2::video::Window;

pub struct FontCache {
    ttf_context: Sdl2TtfContext,
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            ttf_context: sdl2::ttf::init().map_err(|e| e.to_string()).unwrap(),
        }
    }

    pub fn render_text(
        &mut self,
        c: &mut Canvas<Window>,
        font_name: String,
        font_size: u16,
        font_style: FontStyle,
        font_color: Color,
        text: String,
        width: u32,
    ) -> (Texture, u32, u32) {
        let texture_creator = c.texture_creator();
        let mut font = self.ttf_context
            .load_font(Path::new(&font_name), font_size)
            .unwrap();

        font.set_style(font_style);

        let surface = font
            .render(&text)
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
    fn default() -> Self {
        Self::new()
    }
}
