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

//! # RootWidget
//!
//! The top-level `Widget` (ID `0`) that the `Engine` creates for every window.  It fills the
//! window with its color (white by default) and is the parent of every other `Widget`.
//!
//! It lives in the core so the engine doesn't depend on any widget library.  Change the window
//! background with `ui.get_mut::<RootWidget>(ROOT_WIDGET_ID).unwrap().set_color(..)`, or
//! [`Engine::set_background`](crate::engine::Engine::set_background).

use crate::accessibility::{AccessInfo, Role};
use crate::color::Color;
use crate::geometry::{Point, Size};
use crate::paint::{self, Painter};
use crate::widget::{Widget, WidgetCore};

/// The window's root `Widget`: a flat fill of its color.
///
/// The `Engine` creates one as `Widget` `0` for every window.  It is the parent of every other
/// `Widget`.
#[derive(Debug)]
pub struct RootWidget {
    core: WidgetCore,
}

impl Widget for RootWidget {
    /// Returns the root `Widget`'s shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the root `Widget`'s shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Describes the root `Widget` as the application window.
    ///
    /// # Returns
    ///
    /// A [`Role::Window`] node.
    fn accessibility(&self) -> AccessInfo {
        AccessInfo::of(self, Role::Window)
    }

    /// Fills the window with the root `Widget`'s color.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the root `Widget`.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.get_color());
    }
}

impl RootWidget {
    /// Creates a root `Widget` covering a window, filled with white.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the window in pixels.
    ///
    /// # Returns
    ///
    /// The new `RootWidget`.
    pub fn new(size: Size) -> Self {
        let mut root = Self {
            core: WidgetCore::new(Point::new(0, 0), size),
        };
        root.set_color(Color::RGBA(255, 255, 255, 255));
        root
    }
}
