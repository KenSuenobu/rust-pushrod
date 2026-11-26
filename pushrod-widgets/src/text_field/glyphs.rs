// TextField Accessory Glyphs
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

//! The built-in accessory pictures, drawn by hand from fills and lines so they look the same in
//! every font (the built-in font has none of these symbols).  They follow the `2.6-3` mockup:
//! a gray disc with a white cross (clear), a ring with a thick handle (search), and an almond
//! outline with a dark iris, struck through when the password is showing (reveal).

use pushrod::color::Color;
use pushrod::geometry::{Rect, point};
use pushrod::paint::Painter;

/// The disc behind the clear cross.
pub const CLEAR_DISC: Color = Color::RGB(190, 190, 190);

/// The default color of the search and reveal pictures.
pub const SOFT_INK: Color = Color::RGB(96, 96, 96);

/// Returns the half width of an ellipse of radii (`rx`, `ry`) at `dy` rows from its center.
fn half_width(rx: i32, ry: i32, dy: i32) -> i32 {
    if ry <= 0 {
        return rx;
    }

    let t = 1.0 - (dy as f64 / (ry as f64 + 0.5)).powi(2);

    (rx as f64 * t.max(0.0).sqrt() + 0.5).floor() as i32
}

/// Fills an ellipse.
fn disc(p: &mut dyn Painter, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
    for dy in -ry..=ry {
        let half = half_width(rx, ry, dy);

        p.fill_rect(
            Rect::new(cx - half, cy + dy, (half * 2 + 1) as u32, 1),
            color,
        );
    }
}

/// Draws the outline of an ellipse, `thickness` pixels wide.
fn ring(p: &mut dyn Painter, cx: i32, cy: i32, rx: i32, ry: i32, thickness: i32, color: Color) {
    let (irx, iry) = (rx - thickness, ry - thickness);

    for dy in -ry..=ry {
        let outer = half_width(rx, ry, dy);

        if iry < 0 || dy.abs() > iry {
            p.fill_rect(
                Rect::new(cx - outer, cy + dy, (outer * 2 + 1) as u32, 1),
                color,
            );
        } else {
            let inner = half_width(irx, iry, dy).min(outer - 1).max(0);
            let width = (outer - inner + 1) as u32;

            p.fill_rect(Rect::new(cx - outer, cy + dy, width, 1), color);
            p.fill_rect(Rect::new(cx + inner, cy + dy, width, 1), color);
        }
    }
}

/// Draws a line `thickness` pixels thick, by drawing it side by side.
fn thick_line(p: &mut dyn Painter, from: (i32, i32), to: (i32, i32), thickness: i32, color: Color) {
    for offset in 0..thickness {
        let shift = offset - thickness / 2;

        p.line(
            point(from.0 + shift, from.1),
            point(to.0 + shift, to.1),
            color,
        );
        p.line(
            point(from.0, from.1 + shift),
            point(to.0, to.1 + shift),
            color,
        );
    }
}

/// Draws the clear button: a light gray disc with a white cross.
///
/// # Arguments
///
/// * `p` - The painter.
/// * `area` - The accessory's square.
pub fn clear(p: &mut dyn Painter, area: Rect) {
    let (cx, cy) = center(area);
    let radius = (area.w.min(area.h) as i32 * 7 / 22).max(3);
    let arm = (radius / 2).max(1);

    disc(p, cx, cy, radius, radius, CLEAR_DISC);
    thick_line(
        p,
        (cx - arm, cy - arm),
        (cx + arm, cy + arm),
        2,
        Color::WHITE,
    );
    thick_line(
        p,
        (cx - arm, cy + arm),
        (cx + arm, cy - arm),
        2,
        Color::WHITE,
    );
}

/// Draws the search button: a ring with a thick handle running to the lower right.
///
/// # Arguments
///
/// * `p` - The painter.
/// * `area` - The accessory's square.
/// * `color` - The ink.
pub fn search(p: &mut dyn Painter, area: Rect, color: Color) {
    let (cx, cy) = center(area);
    let side = area.w.min(area.h) as i32;
    let radius = (side * 6 / 22).max(3);
    let (ox, oy) = (cx - radius / 3, cy - radius / 3);

    ring(p, ox, oy, radius, radius, 1, color);
    thick_line(
        p,
        (ox + radius * 7 / 10 + 1, oy + radius * 7 / 10 + 1),
        (ox + radius * 7 / 5 + 2, oy + radius * 7 / 5 + 2),
        3,
        color,
    );
}

/// Draws the reveal button: an eye, struck through by a diagonal when `struck` (the characters
/// are showing).
///
/// # Arguments
///
/// * `p` - The painter.
/// * `area` - The accessory's square.
/// * `color` - The ink.
/// * `struck` - `true` to cross the eye out.
pub fn eye(p: &mut dyn Painter, area: Rect, color: Color, struck: bool) {
    let (cx, cy) = center(area);
    let side = area.w.min(area.h) as i32;
    let (rx, ry) = ((side * 9 / 22).max(4), (side * 6 / 22).max(3));
    let iris = (side * 3 / 22).max(1);

    ring(p, cx, cy, rx, ry, 1, color);

    if struck {
        ring(p, cx, cy, iris, iris, 1, color);
        thick_line(p, (cx - rx, cy + ry + 2), (cx + rx, cy - ry - 2), 2, color);
    } else {
        disc(p, cx, cy, iris, iris, color);
    }
}

/// Returns the center of a square.
fn center(area: Rect) -> (i32, i32) {
    (area.x + area.w as i32 / 2, area.y + area.h as i32 / 2)
}
