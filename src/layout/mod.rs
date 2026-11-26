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

//! # Layout
//!
//! Layout managers place a container's children so applications do not hard-code positions.  A
//! window can be resized, text can be translated into longer words, and users can pick larger
//! fonts; a layout re-places everything.
//!
//! The pieces:
//!
//! * [`SizeHint`] - what a `Widget` reports: its minimum, preferred, and maximum size
//!   ([`Widget::size_hint`](crate::widget::Widget::size_hint)).
//! * [`LayoutParams`] - what a `Widget` asks of its container: how much extra space it takes
//!   (`stretch`), how it sits in its slot (`alignment`), and its `margin`.
//! * [`Layout`] - the trait a layout manager implements.  [`Layout::measure`] reports the size a
//!   container needs; [`Layout::arrange`] returns a rectangle for each child.
//!
//! Give any container a layout with [`Widget::set_layout`](crate::widget::Widget::set_layout).
//! Without one, a container keeps absolute positioning.  Layout runs once per frame, before
//! painting, and only for containers that changed; see [`WidgetCache::draw_loop`](crate::cache::WidgetCache::draw_loop).
//!
//! # Examples
//!
//! A layout that stacks its children in a column, written by an application:
//!
//! ```rust
//! use pushrod::geometry::{Rect, Size, size};
//! use pushrod::layout::{ChildInfo, Layout, SizeHint};
//!
//! #[derive(Debug)]
//! struct Stack;
//!
//! impl Layout for Stack {
//!     fn measure(&self, children: &[ChildInfo], _available: Size) -> SizeHint {
//!         let w = children.iter().map(|c| c.hint.preferred.w).max().unwrap_or(0);
//!         let h = children.iter().map(|c| c.hint.preferred.h).sum();
//!
//!         SizeHint::fixed(size(w, h))
//!     }
//!
//!     fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
//!         let mut y = bounds.y;
//!
//!         children
//!             .iter()
//!             .map(|c| {
//!                 let r = Rect::new(bounds.x, y, c.hint.preferred.w, c.hint.preferred.h);
//!                 y += c.hint.preferred.h as i32;
//!                 r
//!             })
//!             .collect()
//!     }
//! }
//! ```

use crate::geometry::{Rect, Size};
use crate::widget::WidgetId;

/// A size with no upper bound, used as a [`SizeHint`] maximum.
pub const UNBOUNDED: u32 = u32::MAX;

/// The smallest, preferred, and largest size of a `Widget`.
///
/// A layout never makes a `Widget` smaller than `min` or larger than `max`, and gives it
/// `preferred` when there is room and the `Widget` does not stretch.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SizeHint {
    /// The smallest size the `Widget` is usable at.
    pub min: Size,
    /// The size the `Widget` wants.
    pub preferred: Size,
    /// The largest size the `Widget` can use; [`UNBOUNDED`] in a dimension means no limit.
    pub max: Size,
}

impl SizeHint {
    /// Creates a hint.
    ///
    /// # Arguments
    ///
    /// * `min` - The smallest size.
    /// * `preferred` - The wanted size.
    /// * `max` - The largest size.
    ///
    /// # Returns
    ///
    /// The hint, with `preferred` raised to at least `min` and lowered to at most `max`, so the
    /// three sizes are always ordered.
    pub fn new(min: Size, preferred: Size, max: Size) -> Self {
        let max = Size::new(max.w.max(min.w), max.h.max(min.h));
        let preferred = Size::new(
            preferred.w.clamp(min.w, max.w),
            preferred.h.clamp(min.h, max.h),
        );

        Self {
            min,
            preferred,
            max,
        }
    }

    /// Creates a hint for a `Widget` that wants one size but may be given more space.
    ///
    /// # Arguments
    ///
    /// * `size` - The minimum and preferred size.
    ///
    /// # Returns
    ///
    /// A hint whose minimum and preferred sizes are `size` and whose maximum is unbounded.
    pub fn fixed(size: Size) -> Self {
        Self::new(size, size, Size::new(UNBOUNDED, UNBOUNDED))
    }

    /// Limits a size to the hint.
    ///
    /// # Arguments
    ///
    /// * `size` - The size to limit.
    ///
    /// # Returns
    ///
    /// `size` raised to at least `min` and lowered to at most `max`, per dimension.
    pub fn clamp(&self, size: Size) -> Size {
        Size::new(
            size.w.clamp(self.min.w, self.max.w),
            size.h.clamp(self.min.h, self.max.h),
        )
    }
}

impl Default for SizeHint {
    /// Creates a hint for an empty `Widget`.
    ///
    /// # Returns
    ///
    /// Minimum and preferred sizes of zero, and an unbounded maximum.
    fn default() -> Self {
        Self::fixed(Size::new(0, 0))
    }
}

/// How a `Widget` sits in the space its layout gives it along one axis.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Align {
    /// At the start of the slot: the left or the top.
    #[default]
    Start,
    /// Centered in the slot.
    Center,
    /// At the end of the slot: the right or the bottom.
    End,
    /// Stretched to fill the slot, up to the `Widget`'s maximum size.
    Fill,
}

/// How a `Widget` sits in its slot, horizontally and vertically.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Alignment {
    /// Placement from left to right.
    pub horizontal: Align,
    /// Placement from top to bottom.
    pub vertical: Align,
}

impl Alignment {
    /// Creates an alignment.
    ///
    /// # Arguments
    ///
    /// * `horizontal` - Placement from left to right.
    /// * `vertical` - Placement from top to bottom.
    ///
    /// # Returns
    ///
    /// The alignment.
    pub fn new(horizontal: Align, vertical: Align) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

/// Empty space kept around a `Widget`, in pixels.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Margin {
    /// Space to the left.
    pub left: u32,
    /// Space above.
    pub top: u32,
    /// Space to the right.
    pub right: u32,
    /// Space below.
    pub bottom: u32,
}

impl Margin {
    /// Creates a margin of the same width on every side.
    ///
    /// # Arguments
    ///
    /// * `all` - The space on each side in pixels.
    ///
    /// # Returns
    ///
    /// The margin.
    pub fn all(all: u32) -> Self {
        Self {
            left: all,
            top: all,
            right: all,
            bottom: all,
        }
    }

    /// Returns the space the margin takes.
    ///
    /// # Returns
    ///
    /// The total horizontal (left plus right) and vertical (top plus bottom) space.
    pub fn total(&self) -> Size {
        Size::new(
            self.left.saturating_add(self.right),
            self.top.saturating_add(self.bottom),
        )
    }
}

/// What a `Widget` asks of the layout of its container.  A new `Widget` has the defaults:
/// no stretch, start alignment, and no margin.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct LayoutParams {
    /// A share of any extra space; `0` keeps the preferred size.  Children with stretch divide
    /// the extra space in proportion to these numbers.
    pub stretch: u32,
    /// How the `Widget` sits in its slot.
    pub alignment: Alignment,
    /// Space kept around the `Widget`.
    pub margin: Margin,
}

/// What a [`Layout`] is told about one child.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChildInfo {
    /// The child's ID.
    pub id: WidgetId,
    /// The child's size hint, not including its margin.
    pub hint: SizeHint,
    /// The child's layout parameters.
    pub params: LayoutParams,
}

/// A layout manager: measures a container's children and places them.
///
/// Applications implement it to add their own managers.  Only visible children are passed in.
/// A layout must be a pure function of its arguments, so it can be run any number of times.
pub trait Layout {
    /// Reports the size the container needs to hold its children.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children, in order.
    /// * `available` - The container's current size.
    ///
    /// # Returns
    ///
    /// The container's [`SizeHint`].
    fn measure(&self, children: &[ChildInfo], available: Size) -> SizeHint;

    /// Places the children inside the container.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children, in order.
    /// * `bounds` - The container's area: its origin is `(0, 0)` and its size is the container's
    ///   size.
    ///
    /// # Returns
    ///
    /// One rectangle per child, in the same order, relative to the container's upper left-hand
    /// corner.  A missing rectangle leaves its child where it is.
    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect>;
}

/// Positions a `Widget` inside a slot, as its [`LayoutParams`] ask.  Layout managers call it so
/// every manager treats margin and alignment the same way.
///
/// # Arguments
///
/// * `slot` - The space the layout gives the child, including its margin.
/// * `child` - The child's hint and parameters.
///
/// # Returns
///
/// The child's rectangle: the slot minus the margin, sized to the preferred size (or filled, for
/// [`Align::Fill`]) within the hint's limits, then aligned in what remains.
pub fn place_in_slot(slot: Rect, child: &ChildInfo) -> Rect {
    let margin = child.params.margin;
    let inner_w = slot.w.saturating_sub(margin.total().w);
    let inner_h = slot.h.saturating_sub(margin.total().h);
    let (x, w) = align_axis(
        slot.x.saturating_add(margin.left as i32),
        inner_w,
        child.hint.preferred.w,
        child.hint.max.w,
        child.params.alignment.horizontal,
    );
    let (y, h) = align_axis(
        slot.y.saturating_add(margin.top as i32),
        inner_h,
        child.hint.preferred.h,
        child.hint.max.h,
        child.params.alignment.vertical,
    );

    Rect::new(x, y, w, h)
}

/// Aligns one dimension of a child in its slot.
///
/// # Arguments
///
/// * `start` - The start of the slot, after the margin.
/// * `room` - The length of the slot, after the margin.
/// * `preferred` - The child's preferred length.
/// * `max` - The child's maximum length.
/// * `align` - Where the child sits.
///
/// # Returns
///
/// The start and length of the child.
fn align_axis(start: i32, room: u32, preferred: u32, max: u32, align: Align) -> (i32, u32) {
    let length = match align {
        Align::Fill => room.min(max),
        _ => preferred.min(room),
    };
    let spare = (room - length) as i32;

    match align {
        Align::Start | Align::Fill => (start, length),
        Align::Center => (start + spare / 2, length),
        Align::End => (start + spare, length),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::size;

    /// Builds a child with a preferred size and parameters.
    ///
    /// # Arguments
    ///
    /// * `preferred` - The preferred size.
    /// * `params` - The layout parameters.
    ///
    /// # Returns
    ///
    /// The child info.
    fn child(preferred: Size, params: LayoutParams) -> ChildInfo {
        ChildInfo {
            id: 1,
            hint: SizeHint::new(size(10, 10), preferred, size(100, 50)),
            params,
        }
    }

    #[test]
    fn hint_keeps_its_sizes_ordered() {
        let hint = SizeHint::new(size(10, 10), size(5, 500), size(8, 20));

        assert_eq!(hint.max, size(10, 20));
        assert_eq!(hint.preferred, size(10, 20));
    }

    #[test]
    fn fixed_hint_is_unbounded_above() {
        let hint = SizeHint::fixed(size(30, 20));

        assert_eq!(hint.min, size(30, 20));
        assert_eq!(hint.preferred, size(30, 20));
        assert_eq!(hint.max, size(UNBOUNDED, UNBOUNDED));
        assert_eq!(hint.clamp(size(5, 500)), size(30, 500));
    }

    #[test]
    fn margin_total_adds_both_sides() {
        assert_eq!(Margin::all(3).total(), size(6, 6));
        assert_eq!(Margin::default().total(), size(0, 0));
    }

    #[test]
    fn slot_alignment_places_the_child() {
        let slot = Rect::new(0, 0, 80, 40);
        let mut params = LayoutParams::default();
        let start = place_in_slot(slot, &child(size(20, 10), params));

        params.alignment = Alignment::new(Align::Center, Align::End);

        let centered = place_in_slot(slot, &child(size(20, 10), params));

        assert_eq!(start, Rect::new(0, 0, 20, 10));
        assert_eq!(centered, Rect::new(30, 30, 20, 10));
    }

    #[test]
    fn slot_fill_stops_at_the_maximum() {
        let params = LayoutParams {
            alignment: Alignment::new(Align::Fill, Align::Fill),
            ..LayoutParams::default()
        };
        let placed = place_in_slot(Rect::new(0, 0, 500, 500), &child(size(20, 10), params));

        assert_eq!(placed, Rect::new(0, 0, 100, 50));
    }

    #[test]
    fn slot_margin_is_kept_and_never_underflows() {
        let params = LayoutParams {
            margin: Margin::all(5),
            ..LayoutParams::default()
        };
        let placed = place_in_slot(Rect::new(10, 10, 100, 100), &child(size(20, 10), params));
        let tiny = place_in_slot(Rect::new(0, 0, 4, 4), &child(size(20, 10), params));

        assert_eq!(placed, Rect::new(15, 15, 20, 10));
        assert_eq!((tiny.w, tiny.h), (0, 0));
    }
}
