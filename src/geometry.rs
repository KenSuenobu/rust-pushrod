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
//!
//! ## Logical and physical pixels
//!
//! Every position and size in Pushrod is in *logical* pixels.  On a high-DPI display a backend
//! draws at *physical* resolution: one logical pixel covers `scale` physical pixels (2 at 2x).
//! [`to_physical`], [`to_physical_size`] and [`stroke_width`] do the conversion, so text and
//! lines stay sharp at any scale factor.
//!
//! ```rust
//! use pushrod::geometry::{Rect, size, stroke_width, to_physical, to_physical_size};
//!
//! assert_eq!(to_physical(Rect::new(10, 10, 100, 20), 2.0), Rect::new(20, 20, 200, 40));
//! assert_eq!(to_physical_size(size(3, 3), 1.5), size(5, 5));
//! assert_eq!(stroke_width(1.5), 2);
//! ```

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

/// The smallest scale factor accepted; smaller (or non-finite) values are treated as `1.0`.
pub const MIN_SCALE_FACTOR: f32 = 0.25;

/// Returns a usable scale factor: `scale` itself, or `1.0` if it is not finite or is below
/// [`MIN_SCALE_FACTOR`].
///
/// # Arguments
///
/// * `scale` - Physical pixels per logical pixel, as a backend reports it.
///
/// # Returns
///
/// The scale factor to draw with.
pub fn sanitize_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale >= MIN_SCALE_FACTOR {
        scale
    } else {
        1.0
    }
}

/// Converts one logical coordinate to physical pixels, rounding to the nearest pixel.
///
/// # Arguments
///
/// * `value` - The logical coordinate.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The physical coordinate.
pub fn to_physical_coord(value: i32, scale: f32) -> i32 {
    (value as f64 * sanitize_scale(scale) as f64).round() as i32
}

/// Converts a logical point to physical pixels.
///
/// # Arguments
///
/// * `p` - The logical point.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The physical point.
pub fn to_physical_point(p: Point, scale: f32) -> Point {
    Point::new(to_physical_coord(p.x, scale), to_physical_coord(p.y, scale))
}

/// Converts a logical rectangle to physical pixels.
///
/// Each edge is rounded on its own, so rectangles that touch in logical pixels still touch in
/// physical pixels, with no gap or overlap, at any scale factor.
///
/// # Arguments
///
/// * `r` - The logical rectangle.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The physical rectangle.
pub fn to_physical(r: Rect, scale: f32) -> Rect {
    let x0 = to_physical_coord(r.x, scale);
    let y0 = to_physical_coord(r.y, scale);
    let x1 = to_physical_coord(r.x.saturating_add_unsigned(r.w), scale);
    let y1 = to_physical_coord(r.y.saturating_add_unsigned(r.h), scale);

    Rect::new(x0, y0, x1.abs_diff(x0), y1.abs_diff(y0))
}

/// Converts a logical size to physical pixels, rounding up so the result covers every physical
/// pixel the logical area touches.  A backend sizes each `Widget`'s texture with it.
///
/// # Arguments
///
/// * `s` - The logical size.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The physical size.
pub fn to_physical_size(s: Size, scale: f32) -> Size {
    let scale = sanitize_scale(scale) as f64;

    Size::new(
        (s.w as f64 * scale).ceil() as u32,
        (s.h as f64 * scale).ceil() as u32,
    )
}

/// Converts a physical length back to logical pixels, rounding up so the logical length always
/// holds the physical one.  Used to report text sizes measured at physical resolution.
///
/// # Arguments
///
/// * `value` - The physical length.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The logical length.
pub fn to_logical_length(value: u32, scale: f32) -> u32 {
    (value as f64 / sanitize_scale(scale) as f64).ceil() as u32
}

/// Returns how many whole physical pixels a one-pixel line or border covers.
///
/// # Arguments
///
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The scale rounded to the nearest whole pixel, at least `1`: `1` at 1x, `2` at 1.5x and 2x,
/// `3` at 3x.
pub fn stroke_width(scale: f32) -> u32 {
    (sanitize_scale(scale).round() as u32).max(1)
}

/// Converts a scale factor to a whole percentage, as engine events carry it.
///
/// # Arguments
///
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The scale in percent: `100` for 1x, `150` for 1.5x.
pub fn scale_percent(scale: f32) -> u32 {
    (sanitize_scale(scale) * 100.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_x_is_the_identity() {
        let r = Rect::new(-3, 7, 41, 9);

        assert_eq!(to_physical(r, 1.0), r);
        assert_eq!(to_physical_size(size(41, 9), 1.0), size(41, 9));
        assert_eq!(to_physical_point(point(-3, 7), 1.0), point(-3, 7));
        assert_eq!(to_logical_length(41, 1.0), 41);
    }

    #[test]
    fn two_x_doubles_everything() {
        assert_eq!(
            to_physical(Rect::new(5, 6, 10, 20), 2.0),
            Rect::new(10, 12, 20, 40)
        );
        assert_eq!(to_physical_size(size(10, 20), 2.0), size(20, 40));
        assert_eq!(to_logical_length(41, 2.0), 21);
    }

    #[test]
    fn touching_rects_still_touch_at_fractional_scales() {
        for scale in [1.25, 1.5, 1.75, 2.5] {
            let left = to_physical(Rect::new(0, 0, 3, 1), scale);
            let right = to_physical(Rect::new(3, 0, 3, 1), scale);

            assert_eq!(left.x + left.w as i32, right.x, "scale {scale}");
        }
    }

    #[test]
    fn physical_sizes_cover_any_placement() {
        let scale = 1.5;
        let s = size(7, 3);
        let texture = to_physical_size(s, scale);

        for x in 0..8 {
            let placed = to_physical(Rect::new(x, x, s.w, s.h), scale);

            assert!(placed.w <= texture.w && placed.h <= texture.h);
        }
    }

    #[test]
    fn strokes_are_whole_pixels() {
        assert_eq!(stroke_width(1.0), 1);
        assert_eq!(stroke_width(1.5), 2);
        assert_eq!(stroke_width(2.0), 2);
        assert_eq!(stroke_width(3.0), 3);
        assert_eq!(stroke_width(0.5), 1);
    }

    #[test]
    fn bad_scales_fall_back_to_one() {
        assert_eq!(sanitize_scale(f32::NAN), 1.0);
        assert_eq!(sanitize_scale(0.0), 1.0);
        assert_eq!(sanitize_scale(-2.0), 1.0);
        assert_eq!(sanitize_scale(f32::INFINITY), 1.0);
        assert_eq!(scale_percent(1.5), 150);
        assert_eq!(scale_percent(f32::NAN), 100);
    }

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
