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

//! # Widget
//!
//! A UI element that exists in a window, whether visible or otherwise, that consumes and
//! alternatively generates events.
//!
//! Every `Widget` owns a [`WidgetCore`], which stores the state that all `Widget`s share: the
//! ID, parent ID, origin, size, color, visibility, invalidation flag, and GPU texture.  The
//! `Widget` trait provides default implementations for all of the getters and setters against
//! the `WidgetCore`, so a new `Widget` only needs to implement three functions:
//!
//! ```rust,no_run
//! use pushrod::font::FontCache;
//! use pushrod::paint;
//! use pushrod::widget::{Widget, WidgetCore};
//! use sdl2::render::Canvas;
//! use sdl2::video::Window;
//!
//! pub struct MyWidget {
//!     core: WidgetCore,
//! }
//!
//! impl Widget for MyWidget {
//!     fn core(&self) -> &WidgetCore { &self.core }
//!     fn core_mut(&mut self) -> &mut WidgetCore { &mut self.core }
//!
//!     fn paint(&self, c: &mut Canvas<Window>, _fc: &mut FontCache) {
//!         paint::fill(c, self.get_color());
//!     }
//! }
//! ```
//!
//! If the compiler is happy, the `Widget` is complete: there are no lists of `Widget` types to
//! update elsewhere in the library.

use crate::event::{EventCtx, PushrodEvent};
use crate::font::FontCache;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::any::Any;

/// The ID of a `Widget`.  IDs are assigned by the `WidgetCache` in the order in which `Widget`s
/// are added, starting with the top-level window `Widget` at ID `0`.  IDs are stable for the
/// lifetime of the `WidgetCache`, which allows forms to be designed and stored by ID.
pub type WidgetId = u32;

/// The ID of the top-level `Widget` that represents the window itself.
pub const ROOT_WIDGET_ID: WidgetId = 0;

/// State that is shared by every `Widget`.  Store one of these in a `Widget` structure, and
/// return it from `Widget::core` and `Widget::core_mut`.
pub struct WidgetCore {
    pub(crate) id: WidgetId,
    pub(crate) parent: WidgetId,
    origin: Point,
    size: Size,
    color: Color,
    hidden: bool,
    invalidated: bool,
    pub(crate) texture: TextureStore,
}

impl WidgetCore {
    /// Creates a new `WidgetCore` given its `Point` of origin, relative to its parent, and its
    /// `Size`.  The default color is transparent white.
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            id: 0,
            parent: ROOT_WIDGET_ID,
            origin,
            size,
            color: Color::RGBA(255, 255, 255, 0),
            hidden: false,
            invalidated: true,
            texture: TextureStore::default(),
        }
    }
}

/// `Widget` is a drawable, interactive object that is presented in a `Window`.
///
/// `Widget`s are components that are used to display information to a user, as well as to provide
/// user-interactable objects.  These make up things like text messages, images, 3D graphics, and
/// so on.
///
/// Only `core`, `core_mut`, and `paint` must be implemented.  `handle_event` defaults to ignoring
/// all events, and every other function operates on the `WidgetCore`.
pub trait Widget: Any {
    /// Returns a reference to the shared `WidgetCore` state.
    fn core(&self) -> &WidgetCore;

    /// Returns a mutable reference to the shared `WidgetCore` state.
    fn core_mut(&mut self) -> &mut WidgetCore;

    /// Paints the `Widget` into its own texture.  The canvas passed in is already targeting the
    /// `Widget`'s texture, so drawing is always done from an origin of `x=0` and `y=0`, bounded by
    /// `get_size()`.  Use the functions in the [`paint`](crate::paint) module to compose the look
    /// of the `Widget` (fill, border, text, ...).
    ///
    /// This is only called when the `Widget` is invalidated.
    fn paint(&self, c: &mut Canvas<Window>, fc: &mut FontCache);

    /// Handles a `PushrodEvent` sent to this `Widget`.  Any events that the `Widget` generates as
    /// a result (such as `Clicked`) are queued with `ctx.emit()`, and delivered to the
    /// application's callbacks after event processing completes.
    ///
    /// The default implementation ignores all events.
    fn handle_event(&mut self, _event: &PushrodEvent, _ctx: &mut EventCtx) {}

    /// Returns the ID of the `Widget`, assigned by the `WidgetCache`.
    fn get_id(&self) -> WidgetId {
        self.core().id
    }

    /// Returns the ID of the parent of this `Widget`.  The top-level `Widget` is its own parent.
    fn get_parent(&self) -> WidgetId {
        self.core().parent
    }

    /// Gets the origin (upper left-hand) coordinate of the `Widget`, relative to its parent.
    fn get_origin(&self) -> Point {
        self.core().origin
    }

    /// Gets the `Size` (bounds) of the object in `width` and `height`.
    fn get_size(&self) -> Size {
        self.core().size
    }

    /// Gets the main (background) color of the `Widget`.
    fn get_color(&self) -> Color {
        self.core().color
    }

    /// Sets the origin point, relative to the parent `Widget`.
    fn set_origin(&mut self, point: Point) {
        self.core_mut().origin = point;
        self.set_invalidated(true);
    }

    /// Sets the size of the `Widget`.
    fn set_size(&mut self, size: Size) {
        self.core_mut().size = size;
        self.set_invalidated(true);
    }

    /// Sets the main (background) color of the `Widget`.
    fn set_color(&mut self, color: Color) {
        self.core_mut().color = color;
        self.set_invalidated(true);
    }

    /// Indicates whether or not the `Widget` (and all of its children) are hidden.  Hidden
    /// `Widget`s are not drawn, and do not receive mouse events.
    fn is_hidden(&self) -> bool {
        self.core().hidden
    }

    /// Hides or shows this `Widget` and all of its children.
    fn set_hidden(&mut self, state: bool) {
        self.core_mut().hidden = state;
        self.set_invalidated(true);
    }

    /// Indicates whether or not an object needs to be repainted.
    fn is_invalidated(&self) -> bool {
        self.core().invalidated
    }

    /// Sets the invalidation state of the object.  When invalidated, the `Widget` is repainted
    /// into its texture on the next frame, and the screen is refreshed.
    fn set_invalidated(&mut self, state: bool) {
        self.core_mut().invalidated = state;
    }
}
