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

use crate::base_widget::BaseWidget;
use crate::box_widget::BoxWidget;
use crate::button_widget::ButtonWidget;
use crate::event::PushrodEvent;
use crate::geometry::{Point, Size};
use crate::image_widget::ImageWidget;
use crate::text_widget::TextWidget;
use crate::texture::TextureStore;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::any::Any;

/// `Widget` is a drawable, interactive object that is presented in a `Window`.
pub trait Widget {
    /// Returns the pure form of the object for casting, if required.
    fn as_any(&self) -> &dyn Any;

    /// Gets the origin (upper left-hand) coordinate of the `Widget`.
    fn get_origin(&self) -> &Point;

    /// Gets the size (bounds) of the object in width and height.
    fn get_size(&self) -> &Size;

    /// Gets the main color of the `Widget`.
    fn get_color(&self) -> Color;

    /// Sets the origin point.
    fn set_origin(&mut self, point: Point);

    /// Sets the size of the `Widget`.  Sub-`Widget` objects must also be resized.
    fn set_size(&mut self, size: Size);

    /// Sets the invalidation state of the object.  When invalidated, it indicates to the
    /// `Engine` that it needs to be refreshed in the main screen, or not.  Note, any `Widget`s that
    /// introduce multiple sub-`Widget`s into their drawing stack also need to be set invalidated
    /// when the top-level is set.  Otherwise, you may exhibit unwanted or odd behavior.
    fn set_invalidated(&mut self, state: bool);

    /// Sets the main color of the `Widget`.
    fn set_color(&mut self, color: Color);

    /// Indicates whether or not an object needs to be redrawn.  This only needs to be implemented
    /// on the top-level `Widget`, unless a sub-`Widget` object changes state based on a timer or
    /// other event.
    fn is_invalidated(&self) -> bool;

    /// Retrieves the `TextureStore` for the `Widget`.  The `TextureStore` is the object's
    /// drawing GPU texture that gets blitted to the screen.  Only update the texture if
    /// invalidated.
    fn get_texture(&mut self) -> &mut TextureStore;

    /// Function that retrieves an event from SDL2, and generates an optional `PushrodEvent` as a
    /// result of the event.
    fn handle_event(&self, event: PushrodEvent) -> Option<&[PushrodEvent]>;

    /// Copies the image stored inside the `Widget` to the screen.  If the `Widget` doesn't
    /// actually _draw_ anything, it can return `None`.  Otherwise, it returns a reference
    /// to the stored `Texture`.
    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture>;
}

/// System Widgets.
pub enum SystemWidget {
    /// Stores a `BaseWidget`, one of the simplest drawing `Widget` objects.
    Base(Box<BaseWidget>),

    /// Stores a `BoxWidget`, an object that contains `BaseWidget` and draws a border with a width and color.
    Box(Box<BoxWidget>),

    /// Stores a `ButtonWidget`, an object that can be interacted with, accepting button clicks.
    Button(Box<ButtonWidget>),

    /// Stores an `ImageWidget`, an object that displays an image.
    Image(Box<ImageWidget>),

    /// Stores a `TextWidget`, an object that draws text within the limits of a widget's bounds.
    Text(Box<TextWidget>),

    /// Stores a custom `Widget`.
    Custom(Box<dyn Widget>),
}
