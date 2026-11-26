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

//! # High-DPI scaling
//!
//! Turns the logical shapes a `Widget` paints into the physical pixels the `SDL2` painter
//! draws, so a high-DPI window gets sharp lines and text instead of a stretched 1x image.  Every
//! function here is plain arithmetic, with no `SDL2` calls, so it is tested without a display.
//!
//! * One-pixel lines and borders become [`stroke_width`] whole physical pixels thick.
//! * Text is rasterized at [`scaled_font_size`], the font size times the scale.
//! * The window's scale is [`window_scale`]: drawable pixels over window pixels.

use pushrod::geometry::{Point, Rect, Size, sanitize_scale, stroke_width, to_physical};

/// Returns the scale factor of a window: how many drawable (physical) pixels it has per window
/// (logical) pixel, horizontally.
///
/// # Arguments
///
/// * `window` - The window size in logical pixels, as `SDL_GetWindowSize` reports it.
/// * `drawable` - The renderer's output size in physical pixels.
///
/// # Returns
///
/// The scale factor; `1.0` if either width is zero.
pub fn window_scale(window: Size, drawable: Size) -> f32 {
    if window.w == 0 || drawable.w == 0 {
        return 1.0;
    }

    sanitize_scale(drawable.w as f32 / window.w as f32)
}

/// Returns the point size to rasterize a font at, so its glyphs have physical resolution.
///
/// # Arguments
///
/// * `size` - The font size in points, as the `Widget` asked for it.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The size times the scale, rounded, at least `1` and at most `u16::MAX`.
pub fn scaled_font_size(size: u16, scale: f32) -> u16 {
    let scaled = (size as f64 * sanitize_scale(scale) as f64).round();

    scaled.clamp(1.0, u16::MAX as f64) as u16
}

/// Splits a rectangle outline into the four filled bars that draw it, each `thickness` physical
/// pixels wide, inside `bounds`.
///
/// # Arguments
///
/// * `bounds` - The outline's outer edge, in physical pixels.
/// * `thickness` - The width of each side, in physical pixels.
///
/// # Returns
///
/// The bars: top, bottom, left, right.  A rectangle too small to have a hole is returned whole;
/// an empty one gives no bars.
pub fn outline_bars(bounds: Rect, thickness: u32) -> Vec<Rect> {
    if bounds.w == 0 || bounds.h == 0 {
        return Vec::new();
    }

    let t = thickness.max(1);

    if bounds.w <= t * 2 || bounds.h <= t * 2 {
        return vec![bounds];
    }

    let right = bounds.x + (bounds.w - t) as i32;
    let bottom = bounds.y + (bounds.h - t) as i32;
    let inner = bounds.h - t * 2;
    let top = bounds.y + t as i32;

    vec![
        Rect::new(bounds.x, bounds.y, bounds.w, t),
        Rect::new(bounds.x, bottom, bounds.w, t),
        Rect::new(bounds.x, top, t, inner),
        Rect::new(right, top, t, inner),
    ]
}

/// Returns the physical rectangle that draws a horizontal or vertical one-pixel line.
///
/// The line covers the logical pixels from `from` to `to`, both ends included, and is
/// [`stroke_width`] physical pixels thick.
///
/// # Arguments
///
/// * `from` - One end, in logical pixels.
/// * `to` - The other end, in logical pixels.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The rectangle, or `None` for a diagonal line (see [`diagonal_offsets`]).
pub fn straight_line(from: Point, to: Point, scale: f32) -> Option<Rect> {
    if from.x != to.x && from.y != to.y {
        return None;
    }

    let x = from.x.min(to.x);
    let y = from.y.min(to.y);
    let covered = Rect::new(x, y, from.x.abs_diff(to.x) + 1, from.y.abs_diff(to.y) + 1);
    let mut physical = to_physical(covered, scale);
    let t = stroke_width(scale);

    if from.y == to.y {
        physical.h = t;
    } else {
        physical.w = t;
    }

    Some(physical)
}

/// Returns the offsets of the parallel one-pixel lines that draw a thick diagonal line: the line
/// is repeated `stroke_width` times, shifted across its main direction.
///
/// # Arguments
///
/// * `from` - One end, in physical pixels.
/// * `to` - The other end, in physical pixels.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// One `(dx, dy)` offset per repeat; `(0, 0)` first.
pub fn diagonal_offsets(from: Point, to: Point, scale: f32) -> Vec<Point> {
    let mostly_horizontal = from.x.abs_diff(to.x) >= from.y.abs_diff(to.y);

    (0..stroke_width(scale) as i32)
        .map(|k| {
            if mostly_horizontal {
                Point::new(0, k)
            } else {
                Point::new(k, 0)
            }
        })
        .collect()
}

/// Returns the physical rectangles of a checkerboard's first-color cells, one per logical pixel,
/// so a dither looks the same at every scale.
///
/// # Arguments
///
/// * `bounds` - The dithered area, in logical pixels.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The cells whose logical `(x + y)` is even, relative to `bounds`.
pub fn dither_cells(bounds: Rect, scale: f32) -> Vec<Rect> {
    (0..bounds.h as i32)
        .flat_map(|y| {
            (0..bounds.w as i32)
                .filter(move |x| (x + y) % 2 == 0)
                .map(move |x| to_physical(Rect::new(bounds.x + x, bounds.y + y, 1, 1), scale))
        })
        .collect()
}

/// Converts the source and destination of a texture copy to physical pixels, with the same size
/// so the copy is never stretched, and the source kept inside the texture.
///
/// # Arguments
///
/// * `source` - The part of the `Widget`'s texture to copy, in logical pixels.
/// * `dest` - Where it goes in the window, in logical pixels.
/// * `texture` - The texture's size, in physical pixels.
/// * `scale` - Physical pixels per logical pixel.
///
/// # Returns
///
/// The physical `(source, dest)` pair.
pub fn blit_rects(source: Rect, dest: Rect, texture: Size, scale: f32) -> (Rect, Rect) {
    let mut source = to_physical(source, scale);
    let mut dest = to_physical(dest, scale);
    let room_w = texture.w.saturating_sub(source.x.max(0) as u32);
    let room_h = texture.h.saturating_sub(source.y.max(0) as u32);
    let w = dest.w.min(room_w);
    let h = dest.h.min(room_h);

    source.w = w;
    source.h = h;
    dest.w = w;
    dest.h = h;

    (source, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pushrod::geometry::{point, size};

    #[test]
    fn window_scale_is_drawable_over_window() {
        assert_eq!(window_scale(size(400, 300), size(800, 600)), 2.0);
        assert_eq!(window_scale(size(400, 300), size(600, 450)), 1.5);
        assert_eq!(window_scale(size(400, 300), size(400, 300)), 1.0);
        assert_eq!(window_scale(size(0, 0), size(800, 600)), 1.0);
    }

    #[test]
    fn fonts_grow_with_the_scale() {
        assert_eq!(scaled_font_size(14, 1.0), 14);
        assert_eq!(scaled_font_size(14, 1.5), 21);
        assert_eq!(scaled_font_size(14, 2.0), 28);
        assert_eq!(scaled_font_size(0, 2.0), 1);
        assert_eq!(scaled_font_size(u16::MAX, 2.0), u16::MAX);
    }

    #[test]
    fn outlines_have_whole_pixel_sides() {
        let bars = outline_bars(Rect::new(0, 0, 20, 10), 2);

        assert_eq!(
            bars,
            vec![
                Rect::new(0, 0, 20, 2),
                Rect::new(0, 8, 20, 2),
                Rect::new(0, 2, 2, 6),
                Rect::new(18, 2, 2, 6),
            ]
        );
        assert_eq!(
            outline_bars(Rect::new(1, 1, 3, 3), 2),
            vec![Rect::new(1, 1, 3, 3)]
        );
        assert!(outline_bars(Rect::new(0, 0, 0, 5), 1).is_empty());
    }

    #[test]
    fn straight_lines_are_stroke_thick() {
        // A 1x line from (2, 5) to (6, 5) covers five logical pixels.
        assert_eq!(
            straight_line(point(2, 5), point(6, 5), 2.0),
            Some(Rect::new(4, 10, 10, 2))
        );
        assert_eq!(
            straight_line(point(3, 9), point(3, 1), 1.5),
            Some(Rect::new(5, 2, 2, 13))
        );
        assert_eq!(
            straight_line(point(0, 0), point(4, 0), 1.0),
            Some(Rect::new(0, 0, 5, 1))
        );
        assert_eq!(straight_line(point(0, 0), point(3, 3), 2.0), None);
    }

    #[test]
    fn diagonals_repeat_across_their_direction() {
        assert_eq!(
            diagonal_offsets(point(0, 0), point(10, 2), 2.0),
            vec![point(0, 0), point(0, 1)]
        );
        assert_eq!(
            diagonal_offsets(point(0, 0), point(2, 10), 2.0),
            vec![point(0, 0), point(1, 0)]
        );
        assert_eq!(diagonal_offsets(point(0, 0), point(5, 5), 1.0).len(), 1);
    }

    #[test]
    fn blits_are_never_stretched_and_stay_inside_the_texture() {
        // A 3x3 widget at (1, 1) and 1.5x: its texture is 5x5 physical pixels.
        let texture = size(5, 5);
        let (source, dest) = blit_rects(Rect::new(0, 0, 3, 3), Rect::new(1, 1, 3, 3), texture, 1.5);

        assert_eq!((source.w, source.h), (dest.w, dest.h));
        assert!(source.x as u32 + source.w <= texture.w);
        assert_eq!(
            blit_rects(
                Rect::new(2, 0, 8, 4),
                Rect::new(12, 10, 8, 4),
                size(20, 8),
                2.0
            ),
            (Rect::new(4, 0, 16, 8), Rect::new(24, 20, 16, 8))
        );
    }

    #[test]
    fn dither_cells_follow_logical_pixels() {
        let cells = dither_cells(Rect::new(0, 0, 2, 2), 2.0);

        assert_eq!(cells, vec![Rect::new(0, 0, 2, 2), Rect::new(2, 2, 2, 2)]);
    }
}
