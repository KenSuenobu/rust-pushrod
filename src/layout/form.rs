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

//! # Form layout
//!
//! [`FormLayout`] places a container's children as "Label: field" rows.  The labels sit in a
//! column as wide as the widest label, aligned to its right edge (the Mac style, the default) or
//! its left edge; the fields fill the rest of the width.  A full-width row (a check box, a group
//! box) spans both columns.
//!
//! When the container is too narrow for the label column and the fields' minimum width, every
//! label moves above its field.
//!
//! Clicking a label focuses its field: the cache asks the layout through
//! [`Layout::focus_target`].
//!
//! Children that are in no row are placed after the rows, as full-width rows, in child order.
//!
//! # Examples
//!
//! ```rust
//! use pushrod::backend::headless::HeadlessBackend;
//! use pushrod::cache::WidgetCache;
//! use pushrod::geometry::{Rect, size};
//! use pushrod::layout::FormLayout;
//! use pushrod::root_widget::RootWidget;
//!
//! let mut cache = WidgetCache::new();
//!
//! cache.add(Box::new(RootWidget::new(size(300, 100))));
//!
//! let name_label = cache.add_child(0, Box::new(RootWidget::new(size(40, 16))));
//! let name = cache.add_child(0, Box::new(RootWidget::new(size(100, 24))));
//! let email_label = cache.add_child(0, Box::new(RootWidget::new(size(60, 16))));
//! let email = cache.add_child(0, Box::new(RootWidget::new(size(100, 24))));
//! let mut form = FormLayout::new().spacing(6).column_spacing(8);
//!
//! form.add_row(name_label, name).add_row(email_label, email);
//! cache.get_mut(0).unwrap().set_layout(Box::new(form));
//! cache.draw_loop(&mut HeadlessBackend::new(size(300, 100)));
//!
//! // Labels are right-aligned in a 60 pixel column; fields fill the rest.
//! assert_eq!(cache.get(name_label).unwrap().get_origin().x, 20);
//! assert_eq!(cache.get(name).unwrap().get_origin().x, 68);
//! assert_eq!(cache.get(email).unwrap().get_size(), size(232, 24));
//! ```

use super::place_in_slot;
use super::{Align, Alignment, ChildInfo, Layout, LayoutParams, Margin, SizeHint, UNBOUNDED};
use crate::geometry::{Rect, Size};
use crate::widget::WidgetId;
use std::collections::{HashMap, HashSet};

/// One row of a [`FormLayout`], as the application added it.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FormRow {
    /// A label in the label column and a field that fills the rest of the row.
    Pair {
        /// The label.
        label: WidgetId,
        /// The field the label names.
        field: WidgetId,
    },
    /// A child that spans the whole row.
    Full(WidgetId),
}

/// A row with its visible children found.
#[derive(Debug, Copy, Clone)]
enum Resolved<'a> {
    /// A label row; either side may be hidden.
    Pair {
        /// The label, if visible.
        label: Option<&'a ChildInfo>,
        /// The field, if visible.
        field: Option<&'a ChildInfo>,
    },
    /// A full-width row.
    Full(&'a ChildInfo),
}

/// A layout of aligned "Label: field" rows.
///
/// Build it with [`FormLayout::new`] and the chained setters, add rows with
/// [`FormLayout::add_row`] and [`FormLayout::add_full_row`], then give it to the container with
/// [`Widget::set_layout`](crate::widget::Widget::set_layout).  The label and field are children
/// of that container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormLayout {
    /// The rows, top to bottom.
    rows: Vec<FormRow>,
    /// Pixels between rows.
    spacing: u32,
    /// Pixels between the label column and the fields.
    column_spacing: u32,
    /// Space kept inside the container's edges.
    margins: Margin,
    /// Where labels sit in the label column.
    label_align: Align,
}

impl FormLayout {
    /// Creates an empty form with no spacing, no margins, and right-aligned labels.
    ///
    /// # Returns
    ///
    /// The layout.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            spacing: 0,
            column_spacing: 0,
            margins: Margin::default(),
            label_align: Align::End,
        }
    }

    /// Sets the space between rows.
    ///
    /// # Arguments
    ///
    /// * `spacing` - The gap in pixels.
    ///
    /// # Returns
    ///
    /// The layout.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the space between the label column and the fields.
    ///
    /// # Arguments
    ///
    /// * `spacing` - The gap in pixels.
    ///
    /// # Returns
    ///
    /// The layout.
    pub fn column_spacing(mut self, spacing: u32) -> Self {
        self.column_spacing = spacing;
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
        self.margins = margins;
        self
    }

    /// Sets where labels sit in the label column.
    ///
    /// # Arguments
    ///
    /// * `align` - [`Align::End`] (the default) right-aligns labels next to their fields, the
    ///   Mac style; [`Align::Start`] left-aligns them.  `Center` centers them and `Fill` makes
    ///   each label as wide as the column.  Labels above their fields (a narrow form) are always
    ///   left-aligned.
    ///
    /// # Returns
    ///
    /// The layout.
    pub fn label_align(mut self, align: Align) -> Self {
        self.label_align = align;
        self
    }

    /// Adds a row with a label and the field it names.
    ///
    /// # Arguments
    ///
    /// * `label` - The label, a child of the container.
    /// * `field` - The field, a child of the container.  Clicking the label focuses it.
    ///
    /// # Returns
    ///
    /// The layout, so rows can be chained.  A row whose label and field are the same `Widget` is
    /// added as a full-width row; a `Widget` already in a row is placed by its first row only.
    pub fn add_row(&mut self, label: WidgetId, field: WidgetId) -> &mut Self {
        if label == field {
            return self.add_full_row(label);
        }

        self.rows.push(FormRow::Pair { label, field });
        self
    }

    /// Adds a row that spans both columns, such as a check box or a group box.
    ///
    /// # Arguments
    ///
    /// * `id` - The child, filled to the width of the form up to its maximum.
    ///
    /// # Returns
    ///
    /// The layout, so rows can be chained.
    pub fn add_full_row(&mut self, id: WidgetId) -> &mut Self {
        self.rows.push(FormRow::Full(id));
        self
    }

    /// Returns the field a label names.
    ///
    /// # Arguments
    ///
    /// * `label` - The label's ID.
    ///
    /// # Returns
    ///
    /// The field of the first row with that label, or `None` if it is not a label.
    pub fn field_for(&self, label: WidgetId) -> Option<WidgetId> {
        self.rows.iter().find_map(|row| match *row {
            FormRow::Pair { label: l, field } if l == label => Some(field),
            _ => None,
        })
    }

    /// Matches the rows to the visible children.  Children in no row become full-width rows
    /// after the others; a child in more than one row keeps its first.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children.
    ///
    /// # Returns
    ///
    /// The rows that have at least one visible child, top to bottom.
    fn resolve<'a>(&self, children: &'a [ChildInfo]) -> Vec<Resolved<'a>> {
        let by_id: HashMap<WidgetId, &ChildInfo> = children.iter().map(|c| (c.id, c)).collect();
        let mut used = HashSet::new();
        let mut take = |id: WidgetId| {
            if used.insert(id) {
                by_id.get(&id).copied()
            } else {
                None
            }
        };
        let mut rows: Vec<Resolved> = Vec::with_capacity(self.rows.len());

        for row in &self.rows {
            match *row {
                FormRow::Pair { label, field } => {
                    let (label, field) = (take(label), take(field));

                    if label.is_some() || field.is_some() {
                        rows.push(Resolved::Pair { label, field });
                    }
                }
                FormRow::Full(id) => {
                    if let Some(child) = take(id) {
                        rows.push(Resolved::Full(child));
                    }
                }
            }
        }

        rows.extend(
            children
                .iter()
                .filter(|c| take(c.id).is_some())
                .map(Resolved::Full),
        );
        rows
    }

    /// Returns the width of the label column: the widest visible label's preferred width, with
    /// its margin.
    ///
    /// # Arguments
    ///
    /// * `rows` - The resolved rows.
    ///
    /// # Returns
    ///
    /// The width in pixels; `0` with no labels.
    fn label_column(rows: &[Resolved]) -> u32 {
        rows.iter()
            .filter_map(|row| match row {
                Resolved::Pair { label, .. } => label.map(|l| outer(l, |h| h.preferred).w),
                Resolved::Full(_) => None,
            })
            .max()
            .unwrap_or(0)
    }

    /// Returns the widest minimum width of the visible fields, with their margins.
    ///
    /// # Arguments
    ///
    /// * `rows` - The resolved rows.
    ///
    /// # Returns
    ///
    /// The width in pixels; `0` with no fields.
    fn field_min(rows: &[Resolved]) -> u32 {
        rows.iter()
            .filter_map(|row| match row {
                Resolved::Pair { field, .. } => field.map(|f| outer(f, |h| h.min).w),
                Resolved::Full(_) => None,
            })
            .max()
            .unwrap_or(0)
    }

    /// Returns whether labels go above their fields: the width inside the margins is less than
    /// the label column, the column spacing, and the widest field minimum.
    ///
    /// # Arguments
    ///
    /// * `rows` - The resolved rows.
    /// * `inner_w` - The width inside the margins.
    ///
    /// # Returns
    ///
    /// `true` for the narrow (stacked) arrangement.
    fn is_narrow(&self, rows: &[Resolved], inner_w: u32) -> bool {
        let label_col = Self::label_column(rows);

        label_col > 0
            && inner_w
                < label_col
                    .saturating_add(self.column_spacing)
                    .saturating_add(Self::field_min(rows))
    }

    /// Returns the height of one row.
    ///
    /// # Arguments
    ///
    /// * `row` - The row.
    /// * `narrow` - `true` if the label goes above the field.
    /// * `pick` - Which size of each hint to use: minimum or preferred.
    ///
    /// # Returns
    ///
    /// The row's height, margins included.
    fn row_height(row: &Resolved, narrow: bool, pick: fn(&SizeHint) -> Size) -> u32 {
        match row {
            Resolved::Full(child) => outer(child, pick).h,
            Resolved::Pair { label, field } => {
                let label_h = label.map_or(0, |l| outer(l, pick).h);
                let field_h = field.map_or(0, |f| outer(f, pick).h);

                if narrow {
                    label_h.saturating_add(field_h)
                } else {
                    label_h.max(field_h)
                }
            }
        }
    }

    /// Returns the height of all rows with spacing and margins.
    ///
    /// # Arguments
    ///
    /// * `rows` - The resolved rows.
    /// * `narrow` - `true` if labels go above their fields.
    /// * `pick` - Which size of each hint to use: minimum or preferred.
    ///
    /// # Returns
    ///
    /// The height in pixels.
    fn total_height(&self, rows: &[Resolved], narrow: bool, pick: fn(&SizeHint) -> Size) -> u32 {
        let gaps = self
            .spacing
            .saturating_mul(rows.len().saturating_sub(1) as u32);

        rows.iter()
            .map(|row| Self::row_height(row, narrow, pick))
            .fold(
                self.margins.total().h.saturating_add(gaps),
                u32::saturating_add,
            )
    }

    /// Places one row's children.
    ///
    /// # Arguments
    ///
    /// * `row` - The row.
    /// * `area` - The row's area: the form's inner width at the row's top, with the row's height.
    /// * `label_col` - The label column's width.
    /// * `narrow` - `true` if the label goes above the field.
    /// * `out` - Receives each child's rectangle.
    fn place_row(
        &self,
        row: &Resolved,
        area: Rect,
        label_col: u32,
        narrow: bool,
        out: &mut HashMap<WidgetId, Rect>,
    ) {
        match *row {
            Resolved::Full(child) => {
                let slot = Rect::new(
                    area.x,
                    area.y,
                    area.w.max(outer(child, |h| h.min).w),
                    area.h,
                );

                out.insert(
                    child.id,
                    place_in_slot(slot, &aligned(child, Align::Fill, None)),
                );
            }
            Resolved::Pair { label, field } if narrow => {
                let label_h = label.map_or(0, |l| outer(l, |h| h.preferred).h);

                if let Some(l) = label {
                    let slot = Rect::new(area.x, area.y, area.w, label_h);

                    out.insert(l.id, place_in_slot(slot, &aligned(l, Align::Start, None)));
                }

                if let Some(f) = field {
                    let y = area.y.saturating_add(label_h as i32);
                    let w = area.w.max(outer(f, |h| h.min).w);
                    let slot = Rect::new(area.x, y, w, area.h.saturating_sub(label_h));

                    out.insert(f.id, place_in_slot(slot, &aligned(f, Align::Fill, None)));
                }
            }
            Resolved::Pair { label, field } => {
                if let Some(l) = label {
                    let slot = Rect::new(area.x, area.y, label_col, area.h);
                    let child = aligned(l, self.label_align, Some(Align::Center));

                    out.insert(l.id, place_in_slot(slot, &child));
                }

                if let Some(f) = field {
                    let offset = label_col.saturating_add(self.column_spacing);
                    let w = area.w.saturating_sub(offset).max(outer(f, |h| h.min).w);
                    let slot = Rect::new(area.x.saturating_add(offset as i32), area.y, w, area.h);

                    out.insert(f.id, place_in_slot(slot, &aligned(f, Align::Fill, None)));
                }
            }
        }
    }
}

impl Default for FormLayout {
    /// Creates an empty form; the same as [`FormLayout::new`].
    ///
    /// # Returns
    ///
    /// The layout.
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for FormLayout {
    /// Reports the size the form needs.  The minimum width fits the labels above their fields;
    /// the preferred width fits them beside.  Heights are for the arrangement the container's
    /// current width gives.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children.
    /// * `available` - The container's current size; its width picks the arrangement.
    ///
    /// # Returns
    ///
    /// The container's hint.
    fn measure(&self, children: &[ChildInfo], available: Size) -> SizeHint {
        let rows = self.resolve(children);
        let margins = self.margins.total();
        let narrow = self.is_narrow(&rows, available.w.saturating_sub(margins.w));
        let label_col = Self::label_column(&rows);
        let widest = |pick: fn(&SizeHint) -> Size, wide: bool| {
            rows.iter()
                .map(|row| match row {
                    Resolved::Full(child) => outer(child, pick).w,
                    Resolved::Pair { label, field } => {
                        let field_w = field.map_or(0, |f| outer(f, pick).w);

                        if wide {
                            label_col
                                .saturating_add(self.column_spacing)
                                .saturating_add(field_w)
                        } else {
                            label.map_or(0, |l| outer(l, pick).w).max(field_w)
                        }
                    }
                })
                .max()
                .unwrap_or(0)
                .saturating_add(margins.w)
        };

        SizeHint::new(
            Size::new(
                widest(|h| h.min, false),
                self.total_height(&rows, narrow, |h| h.min),
            ),
            Size::new(
                widest(|h| h.preferred, true),
                self.total_height(&rows, narrow, |h| h.preferred),
            ),
            Size::new(UNBOUNDED, UNBOUNDED),
        )
    }

    /// Places the rows top to bottom: labels in their column and fields filling the rest, or
    /// labels above fields when the form is narrow.
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
        let rows = self.resolve(children);
        let inner_w = bounds.w.saturating_sub(self.margins.total().w);
        let narrow = self.is_narrow(&rows, inner_w);
        let label_col = Self::label_column(&rows);
        let x = bounds.x.saturating_add(self.margins.left as i32);
        let mut y = bounds.y.saturating_add(self.margins.top as i32);
        let mut placed = HashMap::with_capacity(children.len());

        for row in &rows {
            let height = Self::row_height(row, narrow, |h| h.preferred);

            self.place_row(
                row,
                Rect::new(x, y, inner_w, height),
                label_col,
                narrow,
                &mut placed,
            );
            y = y
                .saturating_add(height as i32)
                .saturating_add(self.spacing as i32);
        }

        children
            .iter()
            .map(|c| placed.get(&c.id).copied().unwrap_or_default())
            .collect()
    }

    /// Returns the field a clicked label names, so the click focuses the field.
    ///
    /// # Arguments
    ///
    /// * `child` - The child that was clicked.
    ///
    /// # Returns
    ///
    /// The field, if `child` is a label.
    fn focus_target(&self, child: WidgetId) -> Option<WidgetId> {
        self.field_for(child)
    }
}

/// Returns a child's hint size with its margin added.
///
/// # Arguments
///
/// * `child` - The child.
/// * `pick` - Which size of the hint to use.
///
/// # Returns
///
/// The size, margin included.
fn outer(child: &ChildInfo, pick: fn(&SizeHint) -> Size) -> Size {
    let size = pick(&child.hint);
    let margin = child.params.margin.total();

    Size::new(
        size.w.saturating_add(margin.w),
        size.h.saturating_add(margin.h),
    )
}

/// Returns a copy of a child with the alignment its slot should use.
///
/// # Arguments
///
/// * `child` - The child.
/// * `horizontal` - The horizontal alignment to use.
/// * `vertical` - The vertical alignment to use; `None` keeps the child's own.
///
/// # Returns
///
/// The adjusted child.
fn aligned(child: &ChildInfo, horizontal: Align, vertical: Option<Align>) -> ChildInfo {
    let vertical = vertical.unwrap_or(child.params.alignment.vertical);

    ChildInfo {
        params: LayoutParams {
            alignment: Alignment::new(horizontal, vertical),
            ..child.params
        },
        ..*child
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::size;

    /// Builds a child with a minimum and preferred size and no maximum.
    ///
    /// # Arguments
    ///
    /// * `id` - The child's ID.
    /// * `min` - The minimum size.
    /// * `preferred` - The preferred size.
    ///
    /// # Returns
    ///
    /// The child info.
    fn child(id: WidgetId, min: Size, preferred: Size) -> ChildInfo {
        ChildInfo {
            id,
            hint: SizeHint::new(min, preferred, size(UNBOUNDED, UNBOUNDED)),
            params: LayoutParams::default(),
        }
    }

    /// Builds two rows: labels 1 (40 wide) and 3 (60 wide) for fields 2 and 4 (minimum 80).
    ///
    /// # Returns
    ///
    /// The layout and its children.
    fn two_rows() -> (FormLayout, Vec<ChildInfo>) {
        let mut form = FormLayout::new().spacing(4).column_spacing(10);

        form.add_row(1, 2).add_row(3, 4);

        let children = vec![
            child(1, size(40, 16), size(40, 16)),
            child(2, size(80, 24), size(100, 24)),
            child(3, size(60, 16), size(60, 16)),
            child(4, size(80, 24), size(100, 24)),
        ];

        (form, children)
    }

    #[test]
    fn labels_align_right_in_a_column_as_wide_as_the_widest() {
        let (form, children) = two_rows();
        let rects = form.arrange(&children, Rect::new(0, 0, 300, 100));

        assert_eq!(rects[0], Rect::new(20, 4, 40, 16));
        assert_eq!(rects[2], Rect::new(0, 32, 60, 16));
        assert_eq!(rects[1].x, 70);
        assert_eq!(rects[3].x, 70);
    }

    #[test]
    fn left_aligned_labels_start_at_the_edge() {
        let (form, children) = two_rows();
        let rects = form
            .label_align(Align::Start)
            .arrange(&children, Rect::new(0, 0, 300, 100));

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[2].x, 0);
    }

    #[test]
    fn fields_stretch_to_the_width() {
        let (form, children) = two_rows();
        let narrow = form.arrange(&children, Rect::new(0, 0, 200, 100));
        let wide = form.arrange(&children, Rect::new(0, 0, 400, 100));

        assert_eq!(narrow[1], Rect::new(70, 0, 130, 24));
        assert_eq!(wide[1], Rect::new(70, 0, 330, 24));
        assert_eq!(wide[3], Rect::new(70, 28, 330, 24));
    }

    #[test]
    fn narrow_forms_put_labels_above_fields() {
        let (form, children) = two_rows();

        // 60 + 10 + 80 = 150 is needed side by side.
        let rects = form.arrange(&children, Rect::new(0, 0, 149, 200));

        assert_eq!(rects[0], Rect::new(0, 0, 40, 16));
        assert_eq!(rects[1], Rect::new(0, 16, 149, 24));
        assert_eq!(rects[2], Rect::new(0, 44, 60, 16));
        assert_eq!(rects[3], Rect::new(0, 60, 149, 24));

        let side_by_side = form.arrange(&children, Rect::new(0, 0, 150, 200));

        assert_eq!(side_by_side[1].x, 70);
    }

    #[test]
    fn full_rows_span_both_columns() {
        let (mut form, mut children) = two_rows();

        form.add_full_row(5);
        children.push(child(5, size(20, 20), size(90, 20)));

        let rects = form.arrange(&children, Rect::new(0, 0, 300, 100));

        assert_eq!(rects[4], Rect::new(0, 56, 300, 20));
    }

    #[test]
    fn children_in_no_row_follow_as_full_rows() {
        let mut form = FormLayout::new();

        form.add_row(1, 2);

        let children = vec![
            child(9, size(10, 10), size(10, 10)),
            child(1, size(10, 10), size(10, 10)),
            child(2, size(10, 10), size(10, 10)),
        ];
        let rects = form.arrange(&children, Rect::new(0, 0, 100, 100));

        assert_eq!(rects[0], Rect::new(0, 10, 100, 10));
    }

    #[test]
    fn hidden_labels_keep_their_fields_in_the_field_column() {
        let (form, children) = two_rows();
        let without_first_label: Vec<ChildInfo> = children[1..].to_vec();
        let rects = form.arrange(&without_first_label, Rect::new(0, 0, 300, 100));

        assert_eq!(rects[0].x, 70);
        assert_eq!(rects.len(), 3);
    }

    #[test]
    fn repeated_children_keep_their_first_row() {
        let mut form = FormLayout::new();

        form.add_row(1, 2).add_full_row(2).add_row(3, 3);

        let children = vec![
            child(1, size(10, 10), size(10, 10)),
            child(2, size(10, 10), size(10, 10)),
            child(3, size(10, 10), size(10, 10)),
        ];
        let rects = form.arrange(&children, Rect::new(0, 0, 100, 100));

        assert_eq!(rects[1], Rect::new(10, 0, 90, 10));
        assert_eq!(rects[2], Rect::new(0, 10, 100, 10));
        assert_eq!(form.field_for(3), None);
    }

    #[test]
    fn labels_name_their_fields() {
        let (form, _) = two_rows();

        assert_eq!(form.focus_target(1), Some(2));
        assert_eq!(form.focus_target(3), Some(4));
        assert_eq!(form.focus_target(2), None);
    }

    #[test]
    fn measure_reports_narrow_minimum_and_wide_preference() {
        let (form, children) = two_rows();
        let wide = form
            .clone()
            .margins(Margin::all(5))
            .measure(&children, size(400, 0));
        let narrow = form.measure(&children, size(100, 0));

        assert_eq!(wide.min, size(80 + 10, 24 + 4 + 24 + 10));
        assert_eq!(wide.preferred, size(60 + 10 + 100 + 10, 24 + 4 + 24 + 10));
        assert_eq!(narrow.min.h, 16 + 24 + 4 + 16 + 24);
        assert_eq!(narrow.preferred.h, narrow.min.h);
    }

    #[test]
    fn empty_forms_measure_their_margins() {
        let form = FormLayout::default().margins(Margin::all(3));

        assert_eq!(form.measure(&[], size(0, 0)).preferred, size(6, 6));
        assert!(form.arrange(&[], Rect::new(0, 0, 10, 10)).is_empty());
    }
}
