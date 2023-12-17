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

use sdl2::ttf::FontStyle;
use crate::base_widget::BaseWidget;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;

pub enum TextJustify {
    Left,
    Center,
    Right,
}

pub struct TextWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    base_widget: BaseWidget,
    font_name: String,
    font_style: FontStyle,
    font_size: u32,
    justification: TextJustify,
    msg: String,
}
