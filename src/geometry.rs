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

//! # Geometry
//!
//! These are geometric representations of shapes and coordinates in drawing space within
//! `Pushrod`.  All objects positioned within a window are drawn using these `Struct`s and
//! method calls.
//!
//! `X` and `Y` positions are from the upper left-hand corner of the widget.

use sdl2::rect::Rect;

/// A geometric struct representing `x` and `y` positional coordinates.
#[derive(Default, Copy, Clone)]
pub struct Point {
    /// Horizontal point from left to right in pixels.
    pub x: i32,

    /// Vertical point from top to bottom in pixels.
    pub y: i32,
}

impl Point {
    /// Constructor to create a new `Point` object.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Creates a new `Point` given `x` and `y` coordinates.
pub fn point(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

/// A geometric struct representing the size of an object in `w`idth and `h`eight.
#[derive(Default, Copy, Clone)]
pub struct Size {
    /// Width in pixels.
    pub w: u32,

    /// Height in pixels.
    pub h: u32,
}

impl Size {
    /// Constructor to create a new `Size` object.
    pub fn new(w: u32, h: u32) -> Self { Self { w, h } }

    /// Computes the amount of memory (in bytes) that a sized object takes in GPU RAM.
    /// This calculation is the `w`idth x `h`eight multiplied by 4 (4 bytes representing RGBA).
    pub fn get_memory_size(&self) -> u32 {
        self.w * self.h * 4
    }
}

/// Creates a new size object with its `w`idth and `h`eight constraints.
pub fn size(w: u32, h: u32) -> Size {
    Size::new(w, h)
}

/// Creates a new `Rect` object given `Point` and `Size` constraints.
pub fn make_rect(origin: Point, bounds: Size) -> Rect {
    Rect::new(origin.x, origin.y, bounds.w, bounds.h)
}

/// Creates a new `Rect` object given `x`, `y`, `w`, and `h` values.
pub fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
    make_rect(point(x, y), size(w, h))
}

/// Returns point of origin (`0x0`)
pub fn origin_point() -> Point {
    Point { x: 0, y: 0 }
}
