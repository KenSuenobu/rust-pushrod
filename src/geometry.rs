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

/// A rectangle: an upper left-hand corner and a size, in pixels.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Rect {
    /// Left edge in pixels.
    pub x: i32,

    /// Top edge in pixels.
    pub y: i32,

    /// Width in pixels.
    pub w: u32,

    /// Height in pixels.
    pub h: u32,
}

impl Rect {
    /// Creates a `Rect`.
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
    /// The new `Rect`.
    pub fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    /// Returns `true` if a point lies inside the `Rect`.
    ///
    /// # Arguments
    ///
    /// * `x` - The horizontal coordinate in pixels.
    /// * `y` - The vertical coordinate in pixels.
    ///
    /// # Returns
    ///
    /// `true` if the point is inside; the left and top edges are inside, the right and bottom edges
    /// are not.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        let (x, y) = (i64::from(x), i64::from(y));

        x >= i64::from(self.x)
            && x < i64::from(self.x) + i64::from(self.w)
            && y >= i64::from(self.y)
            && y < i64::from(self.y) + i64::from(self.h)
    }

    /// Returns the area two `Rect`s have in common.
    ///
    /// # Arguments
    ///
    /// * `other` - The `Rect` to intersect with.
    ///
    /// # Returns
    ///
    /// The overlapping `Rect`, or `None` if they do not overlap (or the overlap is empty).
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = i64::from(self.x).max(i64::from(other.x));
        let top = i64::from(self.y).max(i64::from(other.y));
        let right =
            (i64::from(self.x) + i64::from(self.w)).min(i64::from(other.x) + i64::from(other.w));
        let bottom =
            (i64::from(self.y) + i64::from(self.h)).min(i64::from(other.y) + i64::from(other.h));

        if right <= left || bottom <= top {
            return None;
        }

        Some(Rect::new(
            left as i32,
            top as i32,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
    }
}

/// Creates a rectangle from an origin and a size.
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

/// Creates a rectangle from its position and size.
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
    Rect::new(x, y, w, h)
}

/// Returns the point `(0, 0)`.
///
/// # Returns
///
/// A `Point` at the origin.
pub fn origin_point() -> Point {
    Point { x: 0, y: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_rects_intersect() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, -5, 10, 10);

        assert_eq!(a.intersection(&b), Some(Rect::new(5, 0, 5, 5)));
        assert_eq!(b.intersection(&a), a.intersection(&b));
    }

    #[test]
    fn touching_or_disjoint_rects_do_not_intersect() {
        let a = Rect::new(0, 0, 10, 10);

        assert_eq!(a.intersection(&Rect::new(10, 0, 5, 5)), None);
        assert_eq!(a.intersection(&Rect::new(50, 50, 5, 5)), None);
        assert_eq!(a.intersection(&Rect::new(2, 2, 0, 5)), None);
    }

    #[test]
    fn extreme_rects_do_not_overflow() {
        let a = Rect::new(i32::MAX - 1, 0, u32::MAX, 10);

        assert_eq!(a.intersection(&a), Some(a).filter(|r| r.w > 0));
    }

    #[test]
    fn contains_includes_left_top_and_excludes_right_bottom() {
        let r = Rect::new(2, 3, 4, 5);

        assert!(r.contains(2, 3));
        assert!(r.contains(5, 7));
        assert!(!r.contains(6, 3));
        assert!(!r.contains(2, 8));
        assert!(!r.contains(1, 3));
    }
}
