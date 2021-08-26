// Widget
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

use std::any::Any;
use crate::image_widget::ImageWidget;
use crate::button_widget::ButtonWidget;
use crate::geometry::{Size, Point};
use crate::base_widget::BaseWidget;
use crate::texture::TextureStore;
use sdl2::render::{Texture, Canvas};
use sdl2::video::Window;

/// Widget is a drawable, interactive object that is presented in a Window.
pub trait Widget {
    /// Returns the pure form of the object for casting, if required.
    fn as_any(&self) -> &dyn Any;

    /// Gets the origin (upper left-hand) coordinate of the Widget.
    fn get_origin(&self) -> &Point;

    /// Gets the size (bounds) of the object in width and height.
    fn get_size(&self) -> &Size;

    /// Sets the origin point.
    fn set_origin(&mut self, point: Point);

    /// Sets the size of the Widget.
    fn set_size(&mut self, size: Size);

    /// Sets the invalidation state of the object.  When invalidated, it indicates to the
    /// `Engine` that it needs to be refreshed in the main screen, or not.
    fn set_invalidated(&mut self, state: bool);

    /// Indicates whether or not an object needs to be redrawn.
    fn is_invalidated(&self) -> bool;

    /// Retrieves the `TextureStore` for the Widget.  The `TextureStore` is the object's
    /// drawing GPU texture that gets blitted to the screen.  Only update the texture if
    /// invalidated.
    fn get_texture(&mut self) -> &mut TextureStore;

    /// Blits the image stored inside the `Widget` to the screen.  If the `Widget` doesn't
    /// actually _draw_ anything, it can return `None`.  Otherwise, it returns a reference
    /// to the stored `Texture`.
    fn draw(&mut self, _c: &mut Canvas<Window>) -> Option<&Texture>;
}

/// System Widgets.
pub enum SystemWidget {
    /// Stores a `BaseWidget`, one of the simplest drawing `Widget` objects.
    Base(Box<BaseWidget>),

    /// Stores a `ButtonWidget`, an object that can be interacted with, accepting button clicks.
    Button(Box<ButtonWidget>),

    /// Stores an `ImageWidget`, an object that displays an image.
    Image(Box<ImageWidget>),

    /// Stores a custom `Widget`.
    Custom(Box<dyn Widget>),
}
