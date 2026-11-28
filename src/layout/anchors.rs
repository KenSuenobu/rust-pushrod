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

//! # Anchors (autoresizing)
//!
//! [`Anchors`] say how a child follows its container when the container is resized: keep a
//! margin to an edge, or stretch.  Absolute positioning stays the default (no anchors, nothing
//! moves), and there is no constraint solver: each axis is worked out on its own from how much
//! the container grew or shrank.
//!
//! Anchors apply to containers **without** a layout manager; a container with a layout is
//! arranged by the layout instead.  The [`WidgetCache`](crate::cache::WidgetCache) applies them
//! once per frame, before layout and painting.

use crate::geometry::Rect;
use bitflags::bitflags;

bitflags! {
    /// Which edges of a child follow its container, and whether it stretches.
    ///
    /// With no flags set, the child keeps its place (absolute positioning).  On each axis:
    ///
    /// * the far edge only (`RIGHT`, `BOTTOM`) keeps the margin to that edge, so the child moves;
    /// * both edges (`LEFT | RIGHT`, `TOP | BOTTOM`), or `STRETCH_WIDTH`/`STRETCH_HEIGHT`, keep
    ///   both margins, so the child grows or shrinks;
    /// * the near edge only, or nothing, leaves the child where it is.
    ///
    /// The values are stored in form files (the `anchors` field).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::geometry::Rect;
    /// use pushrod::layout::Anchors;
    ///
    /// // A status bar: sticks to the bottom and stretches across.
    /// let anchors = Anchors::LEFT | Anchors::RIGHT | Anchors::BOTTOM;
    ///
    /// // The container grew by 100 x 50.
    /// let rect = anchors.apply(Rect::new(0, 170, 200, 30), 100, 50);
    ///
    /// assert_eq!(rect, Rect::new(0, 220, 300, 30));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Anchors: u8 {
        /// Keep the margin to the container's left edge.
        const LEFT = 1 << 0;
        /// Keep the margin to the container's right edge.
        const RIGHT = 1 << 1;
        /// Keep the margin to the container's top edge.
        const TOP = 1 << 2;
        /// Keep the margin to the container's bottom edge.
        const BOTTOM = 1 << 3;
        /// Grow and shrink in width with the container.
        const STRETCH_WIDTH = 1 << 4;
        /// Grow and shrink in height with the container.
        const STRETCH_HEIGHT = 1 << 5;
    }
}

/// Applies one axis of the anchors.
///
/// # Arguments
///
/// * `start` - The child's position on the axis.
/// * `length` - The child's length on the axis.
/// * `delta` - How much the container's length changed.
/// * `near` - Whether the near edge (left or top) is anchored.
/// * `far` - Whether the far edge (right or bottom) is anchored.
/// * `stretch` - Whether the child stretches.
///
/// # Returns
///
/// The new `(start, length)`; the length never goes below zero.
fn axis(start: i32, length: u32, delta: i64, near: bool, far: bool, stretch: bool) -> (i32, u32) {
    let clamp = |v: i64| v.clamp(i64::from(i32::MIN), i64::from(i32::MAX));

    if stretch || (near && far) {
        let grown = (i64::from(length) + delta).clamp(0, i64::from(u32::MAX));

        (start, grown as u32)
    } else if far {
        (clamp(i64::from(start) + delta) as i32, length)
    } else {
        (start, length)
    }
}

impl Anchors {
    /// Works out where a child goes when its container changes size.
    ///
    /// # Arguments
    ///
    /// * `rect` - The child's rectangle before, relative to the container.
    /// * `dw` - How much the container's width changed (negative if it shrank).
    /// * `dh` - How much the container's height changed.
    ///
    /// # Returns
    ///
    /// The child's new rectangle; unchanged if no anchor applies.
    pub fn apply(self, rect: Rect, dw: i64, dh: i64) -> Rect {
        let (x, w) = axis(
            rect.x,
            rect.w,
            dw,
            self.contains(Anchors::LEFT),
            self.contains(Anchors::RIGHT),
            self.contains(Anchors::STRETCH_WIDTH),
        );
        let (y, h) = axis(
            rect.y,
            rect.h,
            dh,
            self.contains(Anchors::TOP),
            self.contains(Anchors::BOTTOM),
            self.contains(Anchors::STRETCH_HEIGHT),
        );

        Rect::new(x, y, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keeps_the_place() {
        let rect = Rect::new(5, 6, 70, 80);

        assert_eq!(Anchors::default().apply(rect, 50, -20), rect);
        assert_eq!((Anchors::LEFT | Anchors::TOP).apply(rect, 50, -20), rect);
    }

    #[test]
    fn far_edges_follow_the_container() {
        let rect = Rect::new(5, 6, 70, 80);

        assert_eq!(
            (Anchors::RIGHT | Anchors::BOTTOM).apply(rect, 50, -20),
            Rect::new(55, -14, 70, 80)
        );
    }

    #[test]
    fn both_edges_or_stretch_resize() {
        let rect = Rect::new(5, 6, 70, 80);

        assert_eq!(
            (Anchors::LEFT | Anchors::RIGHT | Anchors::TOP | Anchors::BOTTOM).apply(rect, 10, 20),
            Rect::new(5, 6, 80, 100)
        );
        assert_eq!(
            (Anchors::STRETCH_WIDTH | Anchors::BOTTOM).apply(rect, 10, 20),
            Rect::new(5, 26, 80, 80)
        );
        assert_eq!(
            Anchors::STRETCH_HEIGHT.apply(rect, 10, 20),
            Rect::new(5, 6, 70, 100)
        );
    }

    #[test]
    fn size_never_goes_negative() {
        let rect = Rect::new(0, 0, 30, 30);

        assert_eq!(
            Anchors::STRETCH_WIDTH.apply(rect, -100, 0),
            Rect::new(0, 0, 0, 30)
        );
    }
}
