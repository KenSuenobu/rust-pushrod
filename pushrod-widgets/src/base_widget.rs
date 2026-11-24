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
//! implementation that fills its background with the desired color.

use pushrod::font::FontCache;
use pushrod::geometry::{Point, Size};
use pushrod::paint;
use pushrod::widget::{Widget, WidgetCore};
use sdl2::render::Canvas;
use sdl2::video::Window;

/// BaseWidget structure containing only the shared `WidgetCore` state.
pub struct BaseWidget {
    core: WidgetCore,
}

impl Widget for BaseWidget {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Fills the background with the `Widget`'s color.
    fn paint(&self, c: &mut Canvas<Window>, _fc: &mut FontCache) {
        paint::fill(c, self.get_color());
    }
}

impl BaseWidget {
    /// Creates a new `BaseWidget` given the `Point` of origin and its `Size`.
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            core: WidgetCore::new(origin, size),
        }
    }
}
