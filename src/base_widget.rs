// Base Widget
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

//! # BaseWidget
//!
//! This is a default widget that can be used as a top-level canvas.  It is a basic `Widget`
//! implementation that draws a background, and fills it in with the desired color.

use crate::event::PushrodEvent;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::any::Any;

/// BaseWidget structure containing the point of origin, size, base color, an invalidation flag,
/// and texture store for drawing.
pub struct BaseWidget {
    origin: Point,
    size: Size,
    base_color: Color,
    invalidated: bool,
    texture: TextureStore,
}

impl Widget for BaseWidget {
    /// Returns self object as an `Any` object.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Returns the `Point` of origin for the `Widget`.
    fn get_origin(&self) -> &Point {
        &self.origin
    }

    /// Returns the `Size` of the `Widget`.
    fn get_size(&self) -> &Size {
        &self.size
    }

    /// Returns the `Color` of the `Widget`.
    fn get_color(&self) -> Color {
        self.base_color
    }

    /// Sets the `Point` of origin for this `Widget`.
    fn set_origin(&mut self, point: Point) {
        self.origin = point;
    }

    /// Sets the `Size` of the `Widget`, invalidating the store so that the texture is rebuilt, and
    /// the object is re-drawn on the screen.
    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.set_invalidated(true);
    }

    /// Sets the display invalidation flag.  If set to `true`, the `Texture` is re-blitted to the
    /// screen.
    fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
    }

    /// Sets the `Color` of the `Widget`, invalidating the object in the process for redrawing.
    fn set_color(&mut self, color: Color) {
        self.base_color = color;
        self.set_invalidated(true);
    }

    /// Returns a flag indicating whether or not the `Widget` needs to be redrawn.
    fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    /// Returns the `TextureStore` of the `Widget`.
    fn get_texture(&mut self) -> &mut TextureStore {
        &mut self.texture
    }

    /// Handles any `PushrodEvent` objects.  Since this is a `BaseWidget`, it does not handle or
    /// generate any events.
    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]> { None }

    /// Draws the object.
    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        if self.invalidated {
            self.texture.create_or_resize_texture(c, self.size);

            let base_color = self.base_color;
            let size = self.size;

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture.set_draw_color(base_color);
                texture.clear();
            })
            .unwrap();
        }

        self.texture.get_optional_ref()
    }
}

impl BaseWidget {
    /// Creates a new `BaseWidget` given the `Point` of origin and its `Size`.
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            origin,
            size,
            base_color: Color::RGBA(255, 255, 255, 0),
            invalidated: true,
            texture: TextureStore::default(),
        }
    }
}
