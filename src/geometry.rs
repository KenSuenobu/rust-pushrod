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

/// A position in pixels: `x` from left to right, `y` from top to bottom.
///
/// `Widget` origins are `Point`s relative to the parent `Widget`'s upper left-hand corner.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Point {
    /// Horizontal point from left to right in pixels.
    pub x: i32,

    /// Vertical point from top to bottom in pixels.
    pub y: i32,
}

impl Point {
    /// Creates a `Point`.
    ///
    /// # Arguments
    ///
    /// * `x` - The horizontal position in pixels, from the left.
    /// * `y` - The vertical position in pixels, from the top.
    ///
    /// # Returns
    ///
    /// The new `Point`.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Creates a `Point`; shorthand for [`Point::new`].
///
/// # Arguments
///
/// * `x` - The horizontal position in pixels, from the left.
/// * `y` - The vertical position in pixels, from the top.
///
/// # Returns
///
/// The new `Point`.
pub fn point(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

/// A width and height in pixels.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Size {
    /// Width in pixels.
    pub w: u32,

    /// Height in pixels.
    pub h: u32,
}

impl Size {
    /// Creates a `Size`.
    ///
    /// # Arguments
    ///
    /// * `w` - The width in pixels.
    /// * `h` - The height in pixels.
    ///
    /// # Returns
    ///
    /// The new `Size`.
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }

    /// Returns the GPU memory a texture of this size uses.
    ///
    /// # Returns
    ///
    /// The size in bytes: width times height times 4 (one byte each for red, green, blue, and alpha).
    pub fn get_memory_size(&self) -> u32 {
        self.w * self.h * 4
    }
}

/// Creates a `Size`; shorthand for [`Size::new`].
///
/// # Arguments
///
/// * `w` - The width in pixels.
/// * `h` - The height in pixels.
///
/// # Returns
///
/// The new `Size`.
pub fn size(w: u32, h: u32) -> Size {
    Size::new(w, h)
}

/// Creates an `SDL2` rectangle from an origin and a size.
///
/// # Arguments
///
/// * `origin` - The upper left-hand corner.
/// * `bounds` - The width and height.
///
/// # Returns
///
/// The rectangle.
pub fn make_rect(origin: Point, bounds: Size) -> Rect {
    Rect::new(origin.x, origin.y, bounds.w, bounds.h)
}

/// Creates an `SDL2` rectangle from its position and size.
///
/// # Arguments
///
/// * `x` - The left edge in pixels.
/// * `y` - The top edge in pixels.
/// * `w` - The width in pixels.
/// * `h` - The height in pixels.
///
/// # Returns
///
/// The rectangle.
pub fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
    make_rect(point(x, y), size(w, h))
}

/// Returns the point `(0, 0)`.
///
/// # Returns
///
/// A `Point` at the origin.
pub fn origin_point() -> Point {
    Point { x: 0, y: 0 }
}
