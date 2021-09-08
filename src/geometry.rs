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

use sdl2::rect::Rect;

/// Provides a definition of a point of origin: contains the `x` and `y` coordinates of
/// an object.
#[derive(Default, Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Provides a definition of the size of an object: contains the `w` and `h` coordinates of
/// an object.
#[derive(Default, Copy, Clone)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            w: width,
            h: height,
        }
    }

    /// Shortcut to determine the size (in bytes) that a texture has allocated.
    pub fn get_memory_size(&self) -> u32 {
        self.w * self.h * 4
    }
}

// Helper method to create a rect bounds for `SDL2`
pub fn make_rect(origin: Point, bounds: Size) -> Rect {
    Rect::new(origin.x, origin.y, bounds.w, bounds.h)
}

// Makes an origin point.
pub fn make_origin() -> Point {
    Point { x: 0, y: 0 }
}
