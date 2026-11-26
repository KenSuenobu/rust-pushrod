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

//! # Row and column layouts
//!
//! [`Row`] places its children left to right; [`Column`] places them top to bottom.  Both keep
//! `spacing` pixels between children and `margins` inside their edges, and align children across
//! the other axis.
//!
//! Along the main axis, each child gets its preferred size.  Extra space goes to children whose
//! [`LayoutParams::stretch`](super::LayoutParams::stretch) is above zero, in proportion to it; the
//! others keep their preferred size.  When there is too little space, children shrink toward
//! their minimum.  A child is never made smaller than its minimum or larger than its maximum.
//!
//! A [`Spacer`] is an empty `Widget` that takes up space: `Spacer::stretch()` pushes the
//! children after it to the end, for example to right-align OK and Cancel buttons.
//!
//! # Examples
//!
//! A button row with OK and Cancel at the right:
//!
//! ```rust
//! use pushrod::backend::headless::HeadlessBackend;
//! use pushrod::cache::WidgetCache;
//! use pushrod::geometry::{point, size};
//! use pushrod::layout::{Row, Spacer};
//! use pushrod::root_widget::RootWidget;
//!
//! let mut cache = WidgetCache::new();
//!
//! cache.add(Box::new(RootWidget::new(size(300, 40))));
//! cache.add_child(0, Box::new(Spacer::stretch()));
//!
//! let ok = cache.add_child(0, Box::new(RootWidget::new(size(80, 24))));
//! let cancel = cache.add_child(0, Box::new(RootWidget::new(size(80, 24))));
//!
//! cache.get_mut(0).unwrap().set_layout(Box::new(Row::new().spacing(10)));
//! cache.draw_loop(&mut HeadlessBackend::new(size(300, 40)));
//!
//! assert_eq!(cache.get(ok).unwrap().get_origin(), point(130, 0));
//! assert_eq!(cache.get(cancel).unwrap().get_origin(), point(220, 0));
//! ```

use super::{Align, ChildInfo, Layout, LayoutParams, Margin, SizeHint, UNBOUNDED, place_in_slot};
use crate::geometry::{Point, Rect, Size};
use crate::paint::Painter;
use crate::widget::{Widget, WidgetCore};

/// The direction a linear layout places its children in.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Axis {
    /// Left to right.
    Horizontal,
    /// Top to bottom.
    Vertical,
}

impl Axis {
    /// Returns the main-axis and cross-axis lengths of a size.
    ///
    /// # Arguments
    ///
    /// * `size` - The size to split.
    ///
    /// # Returns
    ///
    /// `(main, cross)`.
    fn split(self, size: Size) -> (u32, u32) {
        match self {
            Axis::Horizontal => (size.w, size.h),
            Axis::Vertical => (size.h, size.w),
        }
    }

    /// Builds a size from main-axis and cross-axis lengths.
    ///
    /// # Arguments
    ///
    /// * `main` - The length along the axis.
    /// * `cross` - The length across the axis.
    ///
    /// # Returns
    ///
    /// The size.
    fn join(self, main: u32, cross: u32) -> Size {
        match self {
            Axis::Horizontal => Size::new(main, cross),
            Axis::Vertical => Size::new(cross, main),
        }
    }
}

/// The shared engine behind [`Row`] and [`Column`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Linear {
    /// The direction children are placed in.
    axis: Axis,
    /// Pixels between neighboring children.
    spacing: u32,
    /// Space kept inside the container's edges.
    margins: Margin,
    /// Cross-axis alignment for every child; `None` uses each child's own alignment.
    cross: Option<Align>,
}

impl Linear {
    /// Creates a linear layout with no spacing, no margins, and each child's own alignment.
    ///
    /// # Arguments
    ///
    /// * `axis` - The direction children are placed in.
    ///
    /// # Returns
    ///
    /// The layout.
    fn new(axis: Axis) -> Self {
        Self {
            axis,
            spacing: 0,
            margins: Margin::default(),
            cross: None,
        }
    }

    /// Returns the total spacing between `count` children.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of children.
    ///
    /// # Returns
    ///
    /// `spacing` times the number of gaps.
    fn gaps(&self, count: usize) -> u32 {
        self.spacing.saturating_mul(count.saturating_sub(1) as u32)
    }

    /// Returns a child's hint along both axes, with its margin added.
    ///
    /// # Arguments
    ///
    /// * `child` - The child.
    ///
    /// # Returns
    ///
    /// `(min, preferred, max)` as `(main, cross)` pairs.
    fn outer(&self, child: &ChildInfo) -> [(u32, u32); 3] {
        let margin = child.params.margin.total();
        let grow = |s: Size| {
            let (main, cross) = self.axis.split(s);
            let (m_main, m_cross) = self.axis.split(margin);

            (main.saturating_add(m_main), cross.saturating_add(m_cross))
        };

        [
            grow(child.hint.min),
            grow(child.hint.preferred),
            grow(child.hint.max),
        ]
    }

    /// Reports the size the container needs.  See [`Layout::measure`].
    fn measure(&self, children: &[ChildInfo]) -> SizeHint {
        let (m_main, m_cross) = self.axis.split(self.margins.total());
        let fixed = m_main.saturating_add(self.gaps(children.len()));
        let (mut min, mut pref, mut max) = ((fixed, 0), (fixed, 0), (fixed, 0));

        for child in children {
            let [c_min, c_pref, c_max] = self.outer(child);

            min = (min.0.saturating_add(c_min.0), min.1.max(c_min.1));
            pref = (pref.0.saturating_add(c_pref.0), pref.1.max(c_pref.1));
            max.0 = max.0.saturating_add(c_max.0);
        }

        SizeHint::new(
            self.axis.join(min.0, min.1.saturating_add(m_cross)),
            self.axis.join(pref.0, pref.1.saturating_add(m_cross)),
            self.axis.join(max.0, UNBOUNDED),
        )
    }

    /// Places the children.  See [`Layout::arrange`].
    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
        let (main_len, cross_len) = self.axis.split(Size::new(bounds.w, bounds.h));
        let (m_main, m_cross) = self.axis.split(self.margins.total());
        let room = main_len
            .saturating_sub(m_main)
            .saturating_sub(self.gaps(children.len()));
        let cross_room = cross_len.saturating_sub(m_cross);
        let outers: Vec<[(u32, u32); 3]> = children.iter().map(|c| self.outer(c)).collect();
        let lengths = distribute(
            room,
            &outers
                .iter()
                .zip(children)
                .map(|(o, c)| Share {
                    min: o[0].0,
                    preferred: o[1].0,
                    max: o[2].0,
                    stretch: c.params.stretch,
                })
                .collect::<Vec<_>>(),
        );
        let (start_main, start_cross) = match self.axis {
            Axis::Horizontal => (
                bounds.x.saturating_add(self.margins.left as i32),
                bounds.y.saturating_add(self.margins.top as i32),
            ),
            Axis::Vertical => (
                bounds.y.saturating_add(self.margins.top as i32),
                bounds.x.saturating_add(self.margins.left as i32),
            ),
        };
        let mut position = start_main;

        children
            .iter()
            .zip(lengths)
            .zip(&outers)
            .map(|((child, length), outer)| {
                // A slot is never narrower than the child's minimum across the axis.
                let cross_room = cross_room.max(outer[0].1);
                let slot = match self.axis {
                    Axis::Horizontal => Rect::new(position, start_cross, length, cross_room),
                    Axis::Vertical => Rect::new(start_cross, position, cross_room, length),
                };

                position = position
                    .saturating_add(length as i32)
                    .saturating_add(self.spacing as i32);
                place_in_slot(slot, &self.slot_child(child))
            })
            .collect()
    }

    /// Returns the child as the slot should treat it: filling the main axis (its length was
    /// already chosen) and using the layout's cross-axis alignment, if one was set.
    ///
    /// # Arguments
    ///
    /// * `child` - The child.
    ///
    /// # Returns
    ///
    /// A copy of the child with adjusted alignment.
    fn slot_child(&self, child: &ChildInfo) -> ChildInfo {
        let mut alignment = child.params.alignment;
        let (main, cross) = match self.axis {
            Axis::Horizontal => (&mut alignment.horizontal, &mut alignment.vertical),
            Axis::Vertical => (&mut alignment.vertical, &mut alignment.horizontal),
        };

        *main = Align::Fill;

        if let Some(align) = self.cross {
            *cross = align;
        }

        ChildInfo {
            params: LayoutParams {
                alignment,
                ..child.params
            },
            ..*child
        }
    }
}

/// One child's lengths along the main axis, margin included.
#[derive(Debug, Copy, Clone)]
struct Share {
    /// The smallest length.
    min: u32,
    /// The wanted length.
    preferred: u32,
    /// The largest length.
    max: u32,
    /// Its share of extra space.
    stretch: u32,
}

/// Divides the main-axis room between children.
///
/// With room to spare, children start at their preferred length and those with stretch share
/// the extra in proportion to it, up to their maximum.  Without enough room, children shrink in
/// proportion to how far each can shrink, down to their minimum.
///
/// # Arguments
///
/// * `room` - The main-axis length available to children, after margins and spacing.
/// * `shares` - Each child's lengths and stretch.
///
/// # Returns
///
/// One length per child.
fn distribute(room: u32, shares: &[Share]) -> Vec<u32> {
    let mut lengths: Vec<u32> = shares.iter().map(|s| s.preferred).collect();
    let wanted: u64 = lengths.iter().map(|&l| l as u64).sum();
    let room = room as u64;

    if room >= wanted {
        grow(&mut lengths, shares, room - wanted);
    } else {
        shrink(&mut lengths, shares, wanted - room);
    }

    lengths
}

/// Gives extra length to the children with stretch, in proportion to it, up to their maximum.
/// Length a child cannot take passes to the others.
///
/// # Arguments
///
/// * `lengths` - The children's lengths, changed in place.
/// * `shares` - Each child's limits and stretch.
/// * `extra` - The length to give out.
fn grow(lengths: &mut [u32], shares: &[Share], mut extra: u64) {
    while extra > 0 {
        let open: Vec<usize> = (0..shares.len())
            .filter(|&i| shares[i].stretch > 0 && lengths[i] < shares[i].max)
            .collect();
        let total: u64 = open.iter().map(|&i| shares[i].stretch as u64).sum();

        if total == 0 {
            return;
        }

        let mut given = 0;

        for &i in &open {
            let share = extra * shares[i].stretch as u64 / total;
            let take = share.min((shares[i].max - lengths[i]) as u64);

            lengths[i] += take as u32;
            given += take;
        }

        if given == 0 {
            // Rounding left less than one pixel per share: hand it out one pixel at a time.
            for &i in open.iter().take(extra as usize) {
                lengths[i] += 1;
                given += 1;
            }
        }

        extra -= given;
    }
}

/// Takes length away from the children, in proportion to how far each can shrink, down to
/// their minimum.  If every child is at its minimum, the children overflow the container.
///
/// # Arguments
///
/// * `lengths` - The children's lengths, changed in place.
/// * `shares` - Each child's limits.
/// * `deficit` - The length to take away.
fn shrink(lengths: &mut [u32], shares: &[Share], mut deficit: u64) {
    while deficit > 0 {
        let open: Vec<usize> = (0..shares.len())
            .filter(|&i| lengths[i] > shares[i].min)
            .collect();
        let total: u64 = open
            .iter()
            .map(|&i| (lengths[i] - shares[i].min) as u64)
            .sum();

        if total == 0 {
            return;
        }

        let mut taken = 0;

        for &i in &open {
            let slack = (lengths[i] - shares[i].min) as u64;
            let take = (deficit * slack / total).min(slack);

            lengths[i] -= take as u32;
            taken += take;
        }

        if taken == 0 {
            for &i in open.iter().take(deficit as usize) {
                lengths[i] -= 1;
                taken += 1;
            }
        }

        deficit -= taken;
    }
}

/// Defines the builder functions shared by [`Row`] and [`Column`].
macro_rules! linear_builders {
    ($name:ident, $axis:expr, $what:literal) => {
        impl $name {
            #[doc = concat!("Creates a ", $what, " with no spacing, no margins, and each child's own alignment.")]
            ///
            /// # Returns
            ///
            /// The layout.
            pub fn new() -> Self {
                Self(Linear::new($axis))
            }

            /// Sets the space between neighboring children.
            ///
            /// # Arguments
            ///
            /// * `spacing` - The gap in pixels.
            ///
            /// # Returns
            ///
            /// The layout.
            pub fn spacing(mut self, spacing: u32) -> Self {
                self.0.spacing = spacing;
                self
            }

            /// Sets the space kept inside the container's edges.
            ///
            /// # Arguments
            ///
            /// * `margins` - The space on each side, such as [`Margin::all`].
            ///
            /// # Returns
            ///
            /// The layout.
            pub fn margins(mut self, margins: Margin) -> Self {
                self.0.margins = margins;
                self
            }

            /// Aligns every child across the main axis, in place of each child's own
            /// [`LayoutParams::alignment`].
            ///
            /// # Arguments
            ///
            /// * `align` - Where children sit across the axis; [`Align::Fill`] stretches them up
            ///   to their maximum.
            ///
            /// # Returns
            ///
            /// The layout.
            pub fn align(mut self, align: Align) -> Self {
                self.0.cross = Some(align);
                self
            }
        }

        impl Default for $name {
            /// Creates the layout with no spacing, no margins, and each child's own alignment; the
            /// same as `new`.
            ///
            /// # Returns
            ///
            /// The layout.
            fn default() -> Self {
                Self::new()
            }
        }

        impl Layout for $name {
            /// Reports the children's sizes added along the main axis, with spacing and margins,
            /// and the largest child across it.
            ///
            /// # Arguments
            ///
            /// * `children` - The visible children.
            /// * `_available` - The container's size; not used.
            ///
            /// # Returns
            ///
            /// The container's hint.
            fn measure(&self, children: &[ChildInfo], _available: Size) -> SizeHint {
                self.0.measure(children)
            }

            /// Places the children one after another, sharing extra space by stretch.
            ///
            /// # Arguments
            ///
            /// * `children` - The visible children.
            /// * `bounds` - The container's area.
            ///
            /// # Returns
            ///
            /// One rectangle per child.
            fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
                self.0.arrange(children, bounds)
            }
        }
    };
}

/// A layout that places its children left to right.
///
/// Children are vertically placed by their own alignment, or by [`Row::align`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Row(Linear);

/// A layout that places its children top to bottom.
///
/// Children are horizontally placed by their own alignment, or by [`Column::align`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Column(Linear);

linear_builders!(Row, Axis::Horizontal, "row");
linear_builders!(Column, Axis::Vertical, "column");

/// An empty, invisible `Widget` that takes up space in a [`Row`] or [`Column`].
///
/// [`Spacer::stretch`] takes all the extra space (shared with other stretching children), which
/// pushes the children after it to the right or bottom.
#[derive(Debug)]
pub struct Spacer {
    core: WidgetCore,
}

impl Spacer {
    /// Creates a spacer that takes the extra space in its row or column.
    ///
    /// # Returns
    ///
    /// A spacer with a minimum and preferred size of zero, no maximum, and a stretch of `1`.
    pub fn stretch() -> Self {
        let mut spacer = Self {
            core: WidgetCore::new(Point::new(0, 0), Size::new(0, 0)),
        };

        spacer.set_layout_params(LayoutParams {
            stretch: 1,
            ..LayoutParams::default()
        });
        spacer
    }
}

impl Widget for Spacer {
    /// Returns the spacer's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the spacer's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing; a spacer is invisible.
    ///
    /// # Arguments
    ///
    /// * `_p` - The painter; not used.
    fn paint(&self, _p: &mut dyn Painter) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::size;

    /// Builds a child.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum size.
    /// * `preferred` - The preferred size.
    /// * `max` - The maximum size.
    /// * `stretch` - The share of extra space.
    ///
    /// # Returns
    ///
    /// The child info.
    fn child(min: Size, preferred: Size, max: Size, stretch: u32) -> ChildInfo {
        ChildInfo {
            id: 1,
            hint: SizeHint::new(min, preferred, max),
            params: LayoutParams {
                stretch,
                ..LayoutParams::default()
            },
        }
    }

    /// Builds a child with a fixed preferred size and no maximum.
    fn fixed(w: u32, h: u32, stretch: u32) -> ChildInfo {
        child(size(w, h), size(w, h), size(UNBOUNDED, UNBOUNDED), stretch)
    }

    #[test]
    fn row_places_children_left_to_right_with_spacing_and_margins() {
        let row = Row::new().spacing(5).margins(Margin::all(10));
        let rects = row.arrange(
            &[fixed(20, 10, 0), fixed(30, 15, 0)],
            Rect::new(0, 0, 200, 50),
        );

        assert_eq!(rects[0], Rect::new(10, 10, 20, 10));
        assert_eq!(rects[1], Rect::new(35, 10, 30, 15));
    }

    #[test]
    fn column_places_children_top_to_bottom() {
        let column = Column::new().spacing(4);
        let rects = column.arrange(
            &[fixed(20, 10, 0), fixed(30, 15, 0)],
            Rect::new(0, 0, 100, 100),
        );

        assert_eq!(rects[0], Rect::new(0, 0, 20, 10));
        assert_eq!(rects[1], Rect::new(0, 14, 30, 15));
    }

    #[test]
    fn stretch_shares_extra_space_in_proportion() {
        let rects = Row::new().arrange(
            &[fixed(10, 10, 1), fixed(10, 10, 0), fixed(10, 10, 3)],
            Rect::new(0, 0, 110, 10),
        );

        assert_eq!(rects[0].w, 30);
        assert_eq!(rects[1].w, 10);
        assert_eq!(rects[2].w, 70);
        assert_eq!(rects[2].x, 40);
    }

    #[test]
    fn stretch_stops_at_the_maximum_and_passes_the_rest_on() {
        let capped = child(size(10, 10), size(10, 10), size(20, 10), 1);
        let rects = Row::new().arrange(&[capped, fixed(10, 10, 1)], Rect::new(0, 0, 100, 10));

        assert_eq!(rects[0].w, 20);
        assert_eq!(rects[1].w, 80);
    }

    #[test]
    fn rounding_leftovers_are_not_lost() {
        let rects = Row::new().arrange(
            &[fixed(0, 10, 1), fixed(0, 10, 1), fixed(0, 10, 1)],
            Rect::new(0, 0, 10, 10),
        );
        let total: u32 = rects.iter().map(|r| r.w).sum();

        assert_eq!(total, 10);
    }

    #[test]
    fn too_little_room_shrinks_toward_the_minimum() {
        let a = child(size(10, 10), size(50, 10), size(UNBOUNDED, 10), 0);
        let b = child(size(40, 10), size(50, 10), size(UNBOUNDED, 10), 0);
        let rects = Row::new().arrange(&[a, b], Rect::new(0, 0, 60, 10));

        assert_eq!(rects[0].w + rects[1].w, 60);
        assert!(rects[1].w >= 40);
        assert!(rects[0].w >= 10);
        assert_eq!(rects[1].x, rects[0].w as i32);

        let tiny = Row::new().arrange(&[a, b], Rect::new(0, 0, 5, 10));

        assert_eq!((tiny[0].w, tiny[1].w), (10, 40));
    }

    #[test]
    fn cross_alignment_overrides_the_childs_own() {
        let rects = Row::new()
            .align(Align::Center)
            .arrange(&[fixed(10, 10, 0)], Rect::new(0, 0, 100, 30));
        let filled = Column::new()
            .align(Align::Fill)
            .arrange(&[fixed(10, 10, 0)], Rect::new(0, 0, 100, 30));

        assert_eq!(rects[0], Rect::new(0, 10, 10, 10));
        assert_eq!(filled[0], Rect::new(0, 0, 100, 10));
    }

    #[test]
    fn cross_fill_stops_at_the_maximum() {
        let capped = child(size(10, 10), size(10, 10), size(UNBOUNDED, 20), 0);
        let rects = Row::new()
            .align(Align::Fill)
            .arrange(&[capped], Rect::new(0, 0, 100, 30));

        assert_eq!(rects[0].h, 20);
    }

    #[test]
    fn child_margins_are_kept() {
        let mut c = fixed(10, 10, 0);

        c.params.margin = Margin::all(2);

        let rects = Row::new().arrange(&[c, fixed(10, 10, 0)], Rect::new(0, 0, 100, 30));

        assert_eq!(rects[0], Rect::new(2, 2, 10, 10));
        assert_eq!(rects[1].x, 14);
    }

    #[test]
    fn measure_adds_along_and_takes_the_largest_across() {
        let row = Row::new().spacing(5).margins(Margin::all(10));
        let a = child(size(5, 5), size(20, 10), size(40, 10), 0);
        let b = child(size(5, 8), size(30, 15), size(60, 15), 0);
        let hint = row.measure(&[a, b], size(0, 0));

        assert_eq!(hint.min, size(5 + 5 + 5 + 20, 8 + 20));
        assert_eq!(hint.preferred, size(20 + 30 + 5 + 20, 15 + 20));
        assert_eq!(hint.max, size(40 + 60 + 5 + 20, UNBOUNDED));

        let column = Column::new().measure(&[a, b], size(0, 0));

        assert_eq!(column.preferred, size(30, 25));
    }

    #[test]
    fn empty_layouts_measure_their_margins() {
        let hint = Column::new()
            .spacing(8)
            .margins(Margin::all(3))
            .measure(&[], size(0, 0));

        assert_eq!(hint.preferred, size(6, 6));
        assert!(
            Row::default()
                .arrange(&[], Rect::new(0, 0, 10, 10))
                .is_empty()
        );
    }

    #[test]
    fn spacer_stretches_and_has_no_size() {
        let spacer = Spacer::stretch();

        assert_eq!(spacer.layout_params().stretch, 1);
        assert_eq!(spacer.get_size(), size(0, 0));
    }
}
