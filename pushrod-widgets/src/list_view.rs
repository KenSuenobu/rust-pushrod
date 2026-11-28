// ListView Widget
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

//! # ListView
//!
//! A scrolling list of text rows backed by a [`ListModel`], with single, multiple, or range
//! selection, keyboard navigation, type-ahead, and activation.
//!
//! It is a [composite](pushrod::composite): a small pool of recycled row `Widget`s and a
//! vertical scroll bar.  Only the rows that fit are ever created, whatever the model's length, so
//! a list of 10,000 (or 10,000,000) rows costs the same as a list of ten.
//!
//! ```text
//!   ListView (composite)
//!    ├── "row:0" … "row:n"  ListRow    one per visible row; rebound to the model as it scrolls
//!    └── "scroll"           ScrollBar  shown only when the model has more rows than fit
//! ```
//!
//! * **Model:** a [`ListModel`] gives the length and each row's text, and reports changes as
//!   ranges ([`ModelChange::Inserted`], [`Removed`](ModelChange::Removed),
//!   [`Changed`](ModelChange::Changed), [`Reset`](ModelChange::Reset)), so the view adjusts its
//!   selection and scroll position and repaints only what moved.  [`VecModel`] is a ready-made
//!   model of strings.
//! * **Selection:** [`SelectionMode::Single`] keeps one row; [`SelectionMode::Range`] keeps a
//!   contiguous run (Shift-click or Shift+arrow extends it); [`SelectionMode::Multiple`] also
//!   allows Ctrl-click (Command-click on macOS) to toggle single rows.
//! * **Keyboard:** Up, Down, Page Up, Page Down, Home, and End move the selection; typing
//!   letters jumps to the next row that starts with them; Return activates; Ctrl+A selects all
//!   (not in single mode).
//! * **Mouse:** a click selects, a double-click activates, and the wheel scrolls three rows.
//!
//! It emits `SelectionChanged { id }` when the selection changes (read it with
//! [`ListView::selected_rows`]), `Activated { id }` for a double-click or Return, and
//! `ViewportChanged { id, offset }` (`offset.y` is the first visible row) when the list scrolls.
//! Changing the selection or scroll position from code emits nothing.
//!
//! ```mermaid
//! flowchart LR
//!   M[ListModel] -- ModelChange ranges --> V[ListView]
//!   V -- rebinds only visible rows --> R[row pool]
//!   U[click / key / wheel] --> V
//!   V -- SelectionChanged / Activated --> A[application callbacks]
//! ```

use crate::scroll_bar::{DEFAULT_THICKNESS, ScrollBar};
use pushrod::accessibility::{AccessActions, AccessInfo, Role};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::error::Result;
use pushrod::event::{Event, EventCtx, EventMask, EventPhase, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Point, Rect, point, size};
use pushrod::input::{Key, Modifiers, MouseButton};
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::properties::{Property, core_properties, set_core_property, track};
use pushrod::registry::PropertyValue;
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};
use std::any::Any;
use std::collections::BTreeSet;
use std::fmt;
use std::ops::Range;
use std::time::{Duration, Instant};

/// The default height of a row, in pixels.
pub const DEFAULT_ROW_HEIGHT: u32 = 20;

/// How many rows the wheel scrolls for one notch.
pub const WHEEL_ROWS: i32 = 3;

/// The default time after which type-ahead starts a new search.
pub const TYPE_AHEAD_TIMEOUT: Duration = Duration::from_millis(1000);

/// The space between a row's left edge and its text, in pixels.
const ROW_PADDING: i32 = 4;

/// A change to a [`ListModel`], reported as a range of rows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelChange {
    /// Rows were inserted; the range is where the new rows are now.
    Inserted(Range<usize>),
    /// Rows were removed; the range is where they were.
    Removed(Range<usize>),
    /// The text of existing rows changed.
    Changed(Range<usize>),
    /// Everything changed: the view forgets its selection and scrolls back to the top.
    Reset,
}

/// The data behind a [`ListView`].
///
/// The view asks for only the rows it shows, so `item` must be cheap.  A model that changes
/// reports each change from [`ListModel::take_changes`]; the view reads them before it next
/// draws.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::list_view::{ListModel, ModelChange};
///
/// /// The numbers from 1 to a million, without storing them.
/// struct Counting;
///
/// impl ListModel for Counting {
///     fn len(&self) -> usize {
///         1_000_000
///     }
///
///     fn item(&self, index: usize) -> String {
///         format!("Item {}", index + 1)
///     }
/// }
///
/// assert_eq!(Counting.item(41), "Item 42");
/// ```
pub trait ListModel: Any {
    /// Returns the number of rows.
    ///
    /// # Returns
    ///
    /// The row count.
    fn len(&self) -> usize;

    /// Returns the text of a row.
    ///
    /// # Arguments
    ///
    /// * `index` - The row, from `0` to `len() - 1`.
    ///
    /// # Returns
    ///
    /// The row's text; empty for an index past the end.
    fn item(&self, index: usize) -> String;

    /// Returns whether the model has no rows.
    ///
    /// # Returns
    ///
    /// `true` if `len()` is `0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Hands over the changes made since the last call.
    ///
    /// # Returns
    ///
    /// The changes, in the order they happened; none for a model that never changes.
    fn take_changes(&mut self) -> Vec<ModelChange> {
        Vec::new()
    }
}

/// A [`ListModel`] of strings that reports its own changes.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::list_view::{ListModel, ModelChange, VecModel};
///
/// let mut model = VecModel::new(["Ada", "Grace"]);
///
/// model.insert(1, "Alan");
///
/// assert_eq!(model.len(), 3);
/// assert_eq!(model.take_changes(), vec![ModelChange::Inserted(1..2)]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct VecModel {
    items: Vec<String>,
    changes: Vec<ModelChange>,
}

impl VecModel {
    /// Creates a model from some rows.
    ///
    /// # Arguments
    ///
    /// * `items` - The rows' text.
    ///
    /// # Returns
    ///
    /// The new model, with no changes pending.
    pub fn new<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            changes: Vec::new(),
        }
    }

    /// Adds a row at the end.
    ///
    /// # Arguments
    ///
    /// * `text` - The row's text.
    pub fn push(&mut self, text: impl Into<String>) {
        self.insert(self.items.len(), text);
    }

    /// Inserts a row.
    ///
    /// # Arguments
    ///
    /// * `index` - Where the row goes; past the end appends it.
    /// * `text` - The row's text.
    pub fn insert(&mut self, index: usize, text: impl Into<String>) {
        let index = index.min(self.items.len());

        self.items.insert(index, text.into());
        self.changes.push(ModelChange::Inserted(index..index + 1));
    }

    /// Removes a range of rows.
    ///
    /// # Arguments
    ///
    /// * `range` - The rows to remove, clamped to the model.
    pub fn remove(&mut self, range: Range<usize>) {
        let end = range.end.min(self.items.len());
        let start = range.start.min(end);

        if start < end {
            self.items.drain(start..end);
            self.changes.push(ModelChange::Removed(start..end));
        }
    }

    /// Replaces the text of a row.
    ///
    /// # Arguments
    ///
    /// * `index` - The row; nothing happens past the end.
    /// * `text` - The new text.
    pub fn set(&mut self, index: usize, text: impl Into<String>) {
        if let Some(item) = self.items.get_mut(index) {
            *item = text.into();
            self.changes.push(ModelChange::Changed(index..index + 1));
        }
    }

    /// Replaces every row.
    ///
    /// # Arguments
    ///
    /// * `items` - The new rows' text.
    pub fn reset<I, S>(&mut self, items: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.items = items.into_iter().map(Into::into).collect();
        self.changes.clear();
        self.changes.push(ModelChange::Reset);
    }
}

impl ListModel for VecModel {
    /// Returns the number of rows.
    ///
    /// # Returns
    ///
    /// The row count.
    fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the text of a row.
    ///
    /// # Arguments
    ///
    /// * `index` - The row.
    ///
    /// # Returns
    ///
    /// The text, or an empty string past the end.
    fn item(&self, index: usize) -> String {
        self.items.get(index).cloned().unwrap_or_default()
    }

    /// Hands over the changes made since the last call.
    ///
    /// # Returns
    ///
    /// The changes, oldest first.
    fn take_changes(&mut self) -> Vec<ModelChange> {
        std::mem::take(&mut self.changes)
    }
}

/// How many rows can be selected, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum SelectionMode {
    /// At most one row.
    #[default]
    Single,
    /// A contiguous run of rows: click, then Shift-click or Shift+arrow to extend.
    Range,
    /// Any set of rows: Ctrl-click toggles one, Shift-click extends a run.
    Multiple,
}

/// The look of one row, returned by a [`ListView::row_style`] callback.  Unset fields keep the
/// list's defaults.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RowStyle {
    /// The font, or `None` for the default font.
    pub font: Option<Font>,
    /// The text color, or `None` for black.
    pub color: Option<Color>,
    /// The background, or `None` for white.
    pub background: Option<Color>,
}

/// A list view's parts.  Get it with [`TypedParts::typed_parts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListViewParts {
    /// The pool of row widgets (`ListRow`), top to bottom.  The first
    /// [`ListView::visible_rows`] show rows; the rest are hidden.
    pub rows: Vec<WidgetId>,
    /// The vertical scroll bar: a `ScrollBar`.
    pub scroll: WidgetId,
}

/// One recycled row: a rectangle of text that the list rebinds as it scrolls.
#[derive(Debug)]
pub struct ListRow {
    core: WidgetCore,
    text: String,
    index: Option<usize>,
    selected: bool,
    cursor: bool,
    font: TextStyle,
    row_color: Option<Color>,
    row_background: Option<Color>,
}

impl ListRow {
    /// Creates an empty row.
    ///
    /// # Returns
    ///
    /// The new row.
    fn new() -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(100, DEFAULT_ROW_HEIGHT));

        core.set_event_mask(EventMask::POINTER_BUTTONS);

        Self {
            core,
            text: String::new(),
            index: None,
            selected: false,
            cursor: false,
            font: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            row_color: None,
            row_background: None,
        }
    }

    /// Returns the model row this widget shows.
    ///
    /// # Returns
    ///
    /// The row index, or `None` while the widget is not showing a row.
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    /// Returns the text shown.
    ///
    /// # Returns
    ///
    /// The row's text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns whether the row is selected.
    ///
    /// # Returns
    ///
    /// `true` if it is drawn selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Points the row at a model row, repainting only if something visible changed.
    ///
    /// # Arguments
    ///
    /// * `index` - The model row.
    /// * `text` - Its text.
    /// * `selected` - Whether it is selected.
    /// * `cursor` - Whether it has the keyboard cursor.
    /// * `style` - The row's own style.
    fn bind(&mut self, index: usize, text: String, selected: bool, cursor: bool, style: &RowStyle) {
        let mut font = self.font.clone();

        if let Some(f) = &style.font {
            font.font = f.clone();
        } else {
            font.font = Font::default();
        }

        let changed = self.index != Some(index)
            || self.text != text
            || self.selected != selected
            || self.cursor != cursor
            || self.row_color != style.color
            || self.row_background != style.background
            || self.font.font != font.font;

        self.index = Some(index);
        self.text = text;
        self.selected = selected;
        self.cursor = cursor;
        self.row_color = style.color;
        self.row_background = style.background;
        self.font = font;

        if changed {
            self.set_invalidated(true);
        }
    }

    /// Detaches the row from the model (it is hidden).
    fn unbind(&mut self) {
        if self.index.take().is_some() {
            self.text.clear();
            self.selected = false;
            self.cursor = false;
            self.set_invalidated(true);
        }
    }
}

impl Widget for ListRow {
    /// Returns the row's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the row's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws the row: white with black text, inverted when selected, with a gray inner border
    /// on the keyboard cursor.  Colors set with `set_style`, or by a `row_style` callback,
    /// replace the defaults for an unselected row.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the row.
    fn paint(&self, p: &mut dyn Painter) {
        let bounds = self.get_size();
        let style = self.style();
        let (background, foreground) = if self.selected {
            (Color::BLACK, Color::WHITE)
        } else {
            (
                style
                    .background
                    .or(self.row_background)
                    .unwrap_or(Color::WHITE),
                style
                    .foreground
                    .or(self.row_color)
                    .unwrap_or(self.font.font_color),
            )
        };
        let font = style.font.as_ref().unwrap_or(&self.font);

        paint::fill(p, background);
        paint::text(
            p,
            Rect::new(
                ROW_PADDING,
                0,
                bounds.w.saturating_sub(ROW_PADDING as u32 * 2),
                bounds.h,
            ),
            font,
            foreground,
            &self.text,
            Truncate::End,
        );

        if self.cursor {
            let ring = if self.selected {
                Color::WHITE
            } else {
                Color::RGB(128, 128, 128)
            };

            paint::border(p, bounds, ring, 1);
        }
    }
}

/// A scrolling list of rows from a [`ListModel`].
///
/// See the [module documentation](self) for its behavior.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::list_view::{ListView, SelectionMode, VecModel};
///
/// let model = VecModel::new((1..=10_000).map(|n| format!("Row {n}")));
/// let list = ListView::new(model)
///     .at(10, 10)
///     .size(200, 160)
///     .selection_mode(SelectionMode::Multiple);
///
/// assert_eq!(list.row_count(), 10_000);
/// assert!(list.selected_rows().is_empty());
/// ```
pub struct ListView {
    core: WidgetCore,
    parts: PartList,
    model: Box<dyn ListModel>,
    row_style: Option<Box<dyn Fn(usize) -> RowStyle>>,
    mode: SelectionMode,
    row_height: u32,
    /// The number of rows the pool was built with.
    pool: usize,
    /// The row count as of the last sync.
    len: usize,
    /// The first visible row.
    top: usize,
    /// The row that has the keyboard cursor.
    cursor: Option<usize>,
    /// The row a Shift extension starts from.
    anchor: Option<usize>,
    selected: BTreeSet<usize>,
    /// The selection changed in a way the application has not been told about.
    selection_unreported: bool,
    scroll_moved: bool,
    reported_top: usize,
    /// The model was reached through `model_mut`, so it may have changes to read.
    model_touched: bool,
    search: String,
    searched_at: Option<Instant>,
    search_timeout: Duration,
}

impl fmt::Debug for ListView {
    /// Formats the list's state; the model and the style callback are not shown.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatter's result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListView")
            .field("core", &self.core)
            .field("rows", &self.len)
            .field("top", &self.top)
            .field("mode", &self.mode)
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl Widget for ListView {
    /// Returns the list's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the list's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Lists the widget's properties: the core properties, then `row_height`.
    ///
    /// # Returns
    ///
    /// The properties.
    fn properties(&self) -> Vec<Property> {
        let mut list = core_properties(self);

        list.push(Property::int("row_height", i64::from(self.row_height)));
        list
    }

    /// Sets a property by name; `row_height`, or else a core property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed.
    ///
    /// # Errors
    ///
    /// [`Error::Property`](pushrod::error::Error::Property) for an unknown or read-only
    /// property, or a value of the wrong kind.
    fn set_property(&mut self, name: &str, value: &PropertyValue) -> Result<bool> {
        match name {
            "row_height" => match value {
                PropertyValue::Int(n) if (1..=i64::from(u16::MAX)).contains(n) => {
                    track(self, |w| w.set_row_height(*n as u32))
                }
                _ => set_core_property(self, name, value),
            },
            _ => set_core_property(self, name, value),
        }
    }

    /// Fills the background white behind the rows.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the list.
    fn paint(&self, p: &mut dyn Painter) {
        let size = self.get_size();

        paint::fill(p, self.style().background.unwrap_or(Color::WHITE));
        paint::border(p, size, Color::BLACK, 1);
    }

    /// Describes the list for assistive technology: a group whose value is the selected rows.
    ///
    /// # Returns
    ///
    /// The role, name, value, and actions.
    fn accessibility(&self) -> AccessInfo {
        let value = self
            .selected
            .iter()
            .map(|&i| self.model.item(i))
            .collect::<Vec<_>>()
            .join(", ");

        AccessInfo::of(self, Role::Group)
            .with_value(value)
            .with_actions(AccessActions::FOCUS)
    }

    /// Handles clicks on rows, the wheel, keys, and focus changes.
    ///
    /// # Arguments
    ///
    /// * `event` - The event, from the list or bubbling up from a row.
    /// * `ctx` - The context used to emit events and read the modifier keys.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for what the list handled.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        match event {
            Event::PointerPressed {
                id,
                button: MouseButton::Left,
                count,
                ..
            } if ctx.phase() == EventPhase::Bubble || *id == self.get_id() => {
                let Some(row) = self.row_for_part(*id) else {
                    return Propagation::Continue;
                };

                let before = self.selected.clone();

                self.click_row(row, ctx.modifiers());
                self.report(&before, ctx);

                if *count >= 2 {
                    ctx.emit(Event::Activated { id: self.get_id() });
                }

                Propagation::Stop
            }
            Event::Scrolled { delta, .. } => {
                let target = self.top as i64 - i64::from(delta.y) * i64::from(WHEEL_ROWS);

                if self.scroll_top(target, ctx) {
                    Propagation::Stop
                } else {
                    Propagation::Continue
                }
            }
            Event::FocusGained { .. } | Event::FocusLost { .. } => {
                self.request_arrange_parts();
                Propagation::Continue
            }
            Event::KeyPressed { key, modifiers, .. } if ctx.phase() == EventPhase::Target => {
                self.handle_key(*key, *modifiers, ctx)
            }
            _ => Propagation::Continue,
        }
    }

    /// Returns the list as a composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    /// Returns the list as a mutable composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for ListView {
    /// Creates the row pool, then the scroll bar (so it is drawn over the rows).
    ///
    /// # Arguments
    ///
    /// * `parts` - Adds the parts.
    fn build(&mut self, parts: &mut PartsBuilder) {
        for i in 0..self.pool {
            parts.add(format!("row:{i}"), ListRow::new());
        }

        parts.add("scroll", ScrollBar::vertical());
    }

    /// Returns the list's parts.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list(&self) -> &PartList {
        &self.parts
    }

    /// Returns the list's parts for modification.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    /// Reads model changes, follows the scroll bar, and rebinds the row pool to the rows now
    /// in view; emits `SelectionChanged` if a model change altered the selection and
    /// `ViewportChanged` if the first visible row moved.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the rows and the scroll bar.
    /// * `ctx` - Queues the events.
    fn sync_parts(&mut self, cache: &mut WidgetCache, ctx: &mut EventCtx) {
        if self.parts.is_empty() {
            return;
        }

        let before = self.selected.clone();

        self.model_touched = false;

        for change in self.model.take_changes() {
            self.apply_change(change);
        }

        self.len = self.model.len();
        self.selected.retain(|&i| i < self.len);
        self.cursor = self.cursor.filter(|&i| i < self.len);
        self.anchor = self.anchor.filter(|&i| i < self.len);

        let parts = self.typed_parts();

        if std::mem::take(&mut self.scroll_moved)
            && let Some(bar) = cache.get_as::<ScrollBar>(parts.scroll)
        {
            let max = self.max_top();

            self.top = (bar.get_value() * max as f64).round() as usize;
        }

        self.top = self.top.min(self.max_top());

        let focused = cache.focused() == Some(self.get_id());
        let visible = self.visible_rows();
        let shown = self.len.saturating_sub(self.top).min(visible);

        for (slot, &id) in parts.rows.iter().enumerate() {
            let Some(row) = cache.get_as_mut::<ListRow>(id) else {
                continue;
            };

            if slot < shown {
                let index = self.top + slot;
                let style = self
                    .row_style
                    .as_ref()
                    .map(|f| f(index))
                    .unwrap_or_default();

                row.bind(
                    index,
                    self.model.item(index),
                    self.selected.contains(&index),
                    focused && self.cursor == Some(index),
                    &style,
                );
            } else {
                row.unbind();
            }

            if let Some(widget) = cache.get_mut(id)
                && widget.is_hidden() != (slot >= shown)
            {
                widget.set_hidden(slot >= shown);
            }
        }

        self.update_scroll_bar(cache, parts.scroll);

        let unreported = std::mem::take(&mut self.selection_unreported);

        if self.selected != before || unreported {
            ctx.emit(Event::SelectionChanged { id: self.get_id() });
        }

        if self.reported_top != self.top {
            self.reported_top = self.top;
            ctx.emit(Event::ViewportChanged {
                id: self.get_id(),
                offset: point(0, self.top as i32),
            });
        }
    }

    /// Places the rows from the top, one `row_height` apart, and the scroll bar along the right
    /// edge when the model has more rows than fit.
    ///
    /// # Returns
    ///
    /// Each part and its rectangle.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let frame = self.get_size();
        let bar = self.bar_shown();
        let width = frame
            .w
            .saturating_sub(if bar { DEFAULT_THICKNESS } else { 0 });
        let mut placed = Vec::with_capacity(self.pool + 1);

        for (i, id) in self.parts.list("row").into_iter().enumerate() {
            placed.push((
                id,
                Rect::new(
                    0,
                    (i as u32 * self.row_height) as i32,
                    width,
                    self.row_height,
                ),
            ));
        }

        if let Some(id) = self.parts.get("scroll") {
            placed.push((id, Rect::new(width as i32, 0, DEFAULT_THICKNESS, frame.h)));
        }

        placed
    }

    /// Notes that the user moved the scroll bar.
    ///
    /// # Arguments
    ///
    /// * `part` - The part that emitted the event.
    /// * `event` - The event.
    /// * `_ctx` - Not used.
    fn handle_part_event(&mut self, part: WidgetId, event: &Event, _ctx: &mut EventCtx) {
        if matches!(event, Event::ValueChanged { .. }) && self.parts.name_of(part) == Some("scroll")
        {
            self.scroll_moved = true;
            self.request_arrange_parts();
        }
    }
}

impl TypedParts for ListView {
    type Parts = ListViewParts;

    /// Returns the list's parts by role.
    ///
    /// # Returns
    ///
    /// The row IDs and the scroll bar's ID; before the list is added to a `WidgetCache` the
    /// rows are empty and the scroll bar is the list's own ID.
    fn typed_parts(&self) -> ListViewParts {
        ListViewParts {
            rows: self.parts.list("row"),
            scroll: self.parts.get("scroll").unwrap_or(self.get_id()),
        }
    }
}

impl ListView {
    /// Creates a list of the rows in a model.
    ///
    /// It is 200 by 200 pixels at the upper left-hand corner of its parent, with 20-pixel rows
    /// and single selection, and can take the keyboard focus.  The pool holds enough rows to
    /// fill the height it has when it is added; reserve more with [`ListView::max_visible_rows`]
    /// before making the list taller.
    ///
    /// # Arguments
    ///
    /// * `model` - The rows.
    ///
    /// # Returns
    ///
    /// The new `ListView`.
    pub fn new(model: impl ListModel) -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(200, 200));

        core.set_event_mask(
            EventMask::POINTER_BUTTONS | EventMask::SCROLL | EventMask::KEYBOARD | EventMask::FOCUS,
        );

        let len = model.len();
        let mut list = Self {
            core,
            parts: PartList::new(),
            model: Box::new(model),
            row_style: None,
            mode: SelectionMode::Single,
            row_height: DEFAULT_ROW_HEIGHT,
            pool: 0,
            len,
            top: 0,
            cursor: None,
            anchor: None,
            selected: BTreeSet::new(),
            selection_unreported: false,
            scroll_moved: false,
            reported_top: 0,
            model_touched: false,
            search: String::new(),
            searched_at: None,
            search_timeout: TYPE_AHEAD_TIMEOUT,
        };

        list.set_behavior(Behavior::FOCUSABLE);
        list.pool = list.fit_rows(200);
        list
    }

    /// Returns how many rows of the current row height fit in a height.
    ///
    /// # Arguments
    ///
    /// * `height` - The height in pixels.
    ///
    /// # Returns
    ///
    /// The number of whole rows, at least `1`.
    fn fit_rows(&self, height: u32) -> usize {
        ((height / self.row_height.max(1)) as usize).max(1)
    }

    /// Sets the upper left-hand corner, relative to the parent `Widget`.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height, and grows the row pool to fill the height.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        Widget::set_size(&mut self, size(width, height));
        self.pool = self.fit_rows(height);
        self
    }

    /// Sets how rows are selected.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode.
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the height of each row.  Call it before [`ListView::size`] so the pool is sized
    /// for it.
    ///
    /// # Arguments
    ///
    /// * `pixels` - The height, at least `1`.
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn row_height(mut self, pixels: u32) -> Self {
        self.row_height = pixels.max(1);
        self.pool = self.fit_rows(self.get_size().h);
        self
    }

    /// Reserves a row pool large enough for `rows` visible rows, so the list can be resized
    /// taller later without losing rows.  The pool never shrinks below what the height needs.
    ///
    /// # Arguments
    ///
    /// * `rows` - The most rows that will be visible.
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn max_visible_rows(mut self, rows: usize) -> Self {
        self.pool = self.pool.max(rows);
        self
    }

    /// Sets how long a pause starts a new type-ahead search.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The pause; [`TYPE_AHEAD_TIMEOUT`] by default.
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn type_ahead_timeout(mut self, timeout: Duration) -> Self {
        self.search_timeout = timeout;
        self
    }

    /// Sets a callback that styles each row from the model: its font, text color, and
    /// background.  It runs when a row is bound to the model, never for rows out of view.
    ///
    /// # Arguments
    ///
    /// * `style` - Called with the model row; returns that row's [`RowStyle`].
    ///
    /// # Returns
    ///
    /// The list, for chaining.
    #[must_use]
    pub fn row_style(mut self, style: impl Fn(usize) -> RowStyle + 'static) -> Self {
        self.row_style = Some(Box::new(style));
        self
    }

    /// Returns the number of rows in the model, as of the last time the list synchronized.
    ///
    /// # Returns
    ///
    /// The row count.
    pub fn row_count(&self) -> usize {
        self.len
    }

    /// Returns how many rows fit in the list (and are bound while there are enough rows).
    ///
    /// # Returns
    ///
    /// The count: the height over the row height, at most the pool size.
    pub fn visible_rows(&self) -> usize {
        self.fit_rows(self.get_size().h).min(self.pool.max(1))
    }

    /// Returns the first visible row.
    ///
    /// # Returns
    ///
    /// The row index at the top of the list.
    pub fn top_row(&self) -> usize {
        self.top
    }

    /// Returns the row that has the keyboard cursor.
    ///
    /// # Returns
    ///
    /// The row, or `None` before any selection or key.
    pub fn cursor_row(&self) -> Option<usize> {
        self.cursor
    }

    /// Returns the selected rows.
    ///
    /// # Returns
    ///
    /// The selected model rows, in ascending order.
    pub fn selected_rows(&self) -> Vec<usize> {
        self.selected.iter().copied().collect()
    }

    /// Returns whether a row is selected.
    ///
    /// # Arguments
    ///
    /// * `row` - The model row.
    ///
    /// # Returns
    ///
    /// `true` if it is selected.
    pub fn is_selected(&self, row: usize) -> bool {
        self.selected.contains(&row)
    }

    /// Returns the selection mode.
    ///
    /// # Returns
    ///
    /// The mode.
    pub fn get_selection_mode(&self) -> SelectionMode {
        self.mode
    }

    /// Changes the selection mode; a selection larger than the new mode allows keeps its
    /// first row.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode.
    pub fn set_selection_mode(&mut self, mode: SelectionMode) {
        self.mode = mode;

        if mode == SelectionMode::Single && self.selected.len() > 1 {
            let first = self.selected.iter().next().copied();

            self.selected = first.into_iter().collect();
            self.request_arrange_parts();
        }
    }

    /// Returns the row height.
    ///
    /// # Returns
    ///
    /// The height in pixels.
    pub fn get_row_height(&self) -> u32 {
        self.row_height
    }

    /// Sets the row height and lays the rows out again.
    ///
    /// # Arguments
    ///
    /// * `pixels` - The height, at least `1`.
    pub fn set_row_height(&mut self, pixels: u32) {
        self.row_height = pixels.max(1);
        self.request_arrange_parts();
    }

    /// Selects one row and scrolls it into view.  Nothing is emitted.
    ///
    /// # Arguments
    ///
    /// * `row` - The model row; ignored if it is past the end.
    pub fn select(&mut self, row: usize) {
        if row >= self.model.len() {
            return;
        }

        self.selected = BTreeSet::from([row]);
        self.cursor = Some(row);
        self.anchor = Some(row);
        self.reveal(row);
        self.request_arrange_parts();
    }

    /// Selects a range of rows (the first row only in single mode).  Nothing is emitted.
    ///
    /// # Arguments
    ///
    /// * `rows` - The rows, clamped to the model.
    pub fn select_range(&mut self, rows: Range<usize>) {
        let end = rows.end.min(self.model.len());

        if rows.start >= end {
            return;
        }

        let end = if self.mode == SelectionMode::Single {
            rows.start + 1
        } else {
            end
        };

        self.selected = (rows.start..end).collect();
        self.cursor = Some(end - 1);
        self.anchor = Some(rows.start);
        self.reveal(end - 1);
        self.request_arrange_parts();
    }

    /// Clears the selection.  Nothing is emitted.
    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.request_arrange_parts();
    }

    /// Scrolls so a row is the first visible one (clamped).  Nothing is emitted.
    ///
    /// # Arguments
    ///
    /// * `row` - The model row to show at the top.
    pub fn scroll_to_row(&mut self, row: usize) {
        self.top = row.min(self.max_top());
        self.request_arrange_parts();
    }

    /// Returns the model.
    ///
    /// # Returns
    ///
    /// The model as its concrete type, or `None` if it is another type.
    pub fn model<M: ListModel>(&self) -> Option<&M> {
        let any: &dyn Any = &*self.model;

        any.downcast_ref::<M>()
    }

    /// Returns the model for modification, and schedules the list to read the changes it
    /// reports before the next frame.
    ///
    /// # Returns
    ///
    /// The model as its concrete type, or `None` if it is another type.
    pub fn model_mut<M: ListModel>(&mut self) -> Option<&mut M> {
        self.model_touched = true;
        self.request_arrange_parts();

        let any: &mut dyn Any = &mut *self.model;

        any.downcast_mut::<M>()
    }

    /// Adjusts the selection and scroll position for a change to the model.  The list does
    /// this itself for the changes a model reports; call it only for a model that cannot.
    ///
    /// # Arguments
    ///
    /// * `change` - The change.
    pub fn apply_change(&mut self, change: ModelChange) {
        let before = self.selected.clone();

        match change {
            ModelChange::Inserted(range) => {
                let n = range.len();

                self.selected = self
                    .selected
                    .iter()
                    .map(|&i| if i >= range.start { i + n } else { i })
                    .collect();
                self.cursor = self
                    .cursor
                    .map(|i| if i >= range.start { i + n } else { i });
                self.anchor = self
                    .anchor
                    .map(|i| if i >= range.start { i + n } else { i });

                if self.top >= range.start && self.top > 0 {
                    self.top += n;
                }
            }
            ModelChange::Removed(range) => {
                let n = range.len();
                let shift = |i: usize| {
                    if i >= range.end {
                        Some(i - n)
                    } else if i >= range.start {
                        None
                    } else {
                        Some(i)
                    }
                };

                self.selected = self.selected.iter().filter_map(|&i| shift(i)).collect();
                self.cursor = self.cursor.and_then(shift);
                self.anchor = self.anchor.and_then(shift);
                self.top = self
                    .top
                    .saturating_sub(range.len().min(self.top.saturating_sub(range.start)));
            }
            ModelChange::Changed(_) => {}
            ModelChange::Reset => {
                self.selected.clear();
                self.cursor = None;
                self.anchor = None;
                self.top = 0;
            }
        }

        if self.selected != before {
            self.selection_unreported = true;
        }

        self.request_arrange_parts();
    }

    /// Returns the largest first-visible row.
    ///
    /// # Returns
    ///
    /// Row count less the visible rows; `0` when everything fits.
    fn max_top(&self) -> usize {
        self.len.saturating_sub(self.visible_rows())
    }

    /// Returns whether the scroll bar is shown.
    ///
    /// # Returns
    ///
    /// `true` if the model has more rows than fit.
    fn bar_shown(&self) -> bool {
        self.len > self.visible_rows()
    }

    /// Shows or hides the scroll bar and sets its thumb.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the bar.
    /// * `id` - The bar's ID.
    fn update_scroll_bar(&self, cache: &mut WidgetCache, id: WidgetId) {
        let shown = self.bar_shown();

        if let Some(widget) = cache.get_mut(id)
            && widget.is_hidden() == shown
        {
            widget.set_hidden(!shown);
        }

        let Some(bar) = cache.get_as_mut::<ScrollBar>(id) else {
            return;
        };
        let max = self.max_top();

        bar.set_proportion(self.visible_rows() as f64, self.len.max(1) as f64);

        if max == 0 {
            bar.set_step(0.0);
            bar.set_value(0.0);
            return;
        }

        bar.set_step(1.0 / max as f64);

        if (bar.get_value() * max as f64).round() as usize != self.top {
            bar.set_value(self.top as f64 / max as f64);
        }
    }

    /// Finds the model row a part shows.
    ///
    /// # Arguments
    ///
    /// * `part` - A part's ID (the widget under the pointer).
    ///
    /// # Returns
    ///
    /// The model row, or `None` if the part is not a row showing one.
    fn row_for_part(&self, part: WidgetId) -> Option<usize> {
        let slot = self.parts.list("row").iter().position(|&id| id == part)?;
        let row = self.top + slot;

        (row < self.len).then_some(row)
    }

    /// Scrolls so a row is within the visible rows.
    ///
    /// # Arguments
    ///
    /// * `row` - The model row.
    fn reveal(&mut self, row: usize) {
        let visible = self.visible_rows();

        if row < self.top {
            self.top = row;
        } else if row >= self.top + visible {
            self.top = row + 1 - visible;
        }

        self.top = self.top.min(self.max_top());
    }

    /// Moves the first visible row from inside event handling, emitting `ViewportChanged` at
    /// once if it moved.
    ///
    /// # Arguments
    ///
    /// * `target` - The wanted first row, clamped to the list.
    /// * `ctx` - The context used to emit the event.
    ///
    /// # Returns
    ///
    /// `true` if the list scrolled.
    fn scroll_top(&mut self, target: i64, ctx: &mut EventCtx) -> bool {
        let top = target.clamp(0, self.max_top() as i64) as usize;

        if top == self.top {
            return false;
        }

        self.top = top;
        self.reported_top = top;
        self.request_arrange_parts();
        ctx.emit(Event::ViewportChanged {
            id: self.get_id(),
            offset: point(0, top as i32),
        });

        true
    }

    /// Applies a click on a row to the selection.
    ///
    /// # Arguments
    ///
    /// * `row` - The model row clicked.
    /// * `modifiers` - The modifier keys held.
    fn click_row(&mut self, row: usize, modifiers: Modifiers) {
        let extend = modifiers.contains(Modifiers::SHIFT) && self.mode != SelectionMode::Single;
        let toggle = modifiers.intersects(Modifiers::CTRL | Modifiers::GUI)
            && self.mode == SelectionMode::Multiple;

        if extend {
            let from = self.anchor.unwrap_or(row);

            self.selected = (from.min(row)..=from.max(row)).collect();
        } else if toggle {
            if !self.selected.remove(&row) {
                self.selected.insert(row);
            }

            self.anchor = Some(row);
        } else {
            self.selected = BTreeSet::from([row]);
            self.anchor = Some(row);
        }

        self.cursor = Some(row);
        self.reveal(row);
        self.request_arrange_parts();
    }

    /// Moves the keyboard cursor, selecting as it goes.
    ///
    /// # Arguments
    ///
    /// * `row` - The row to move to.
    /// * `extend` - Whether Shift is held (extend from the anchor).
    fn move_cursor(&mut self, row: usize, extend: bool) {
        if extend && self.mode != SelectionMode::Single {
            let from = self.anchor.unwrap_or(row);

            self.selected = (from.min(row)..=from.max(row)).collect();
        } else {
            self.selected = BTreeSet::from([row]);
            self.anchor = Some(row);
        }

        self.cursor = Some(row);
        self.reveal(row);
        self.request_arrange_parts();
    }

    /// Emits `SelectionChanged` if the selection differs from an earlier one.
    ///
    /// # Arguments
    ///
    /// * `before` - The selection before the change.
    /// * `ctx` - The context used to emit the event.
    fn report(&mut self, before: &BTreeSet<usize>, ctx: &mut EventCtx) {
        if &self.selected != before {
            ctx.emit(Event::SelectionChanged { id: self.get_id() });
        }
    }

    /// Handles a key while the list has the keyboard.
    ///
    /// # Arguments
    ///
    /// * `key` - The key.
    /// * `modifiers` - The modifier keys held.
    /// * `ctx` - The context used to emit events.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for a key the list used.
    fn handle_key(&mut self, key: Key, modifiers: Modifiers, ctx: &mut EventCtx) -> Propagation {
        let len = self.len;
        let extend = modifiers.contains(Modifiers::SHIFT);
        let page = self.visible_rows().saturating_sub(1).max(1);
        let last = len.saturating_sub(1);
        let current = self.cursor;
        let before = self.selected.clone();
        let target = match key {
            Key::Up => Some(current.map_or(last, |c| c.saturating_sub(1))),
            Key::Down => Some(current.map_or(0, |c| (c + 1).min(last))),
            Key::PageUp => Some(current.map_or(0, |c| c.saturating_sub(page))),
            Key::PageDown => Some(current.map_or(page.min(last), |c| (c + page).min(last))),
            Key::Home => Some(0),
            Key::End => Some(last),
            Key::Return | Key::KpEnter => {
                if current.is_some() {
                    ctx.emit(Event::Activated { id: self.get_id() });
                    return Propagation::Stop;
                }

                return Propagation::Continue;
            }
            Key::Char('a')
                if modifiers.intersects(Modifiers::CTRL | Modifiers::GUI)
                    && self.mode != SelectionMode::Single =>
            {
                self.selected = (0..len).collect();
                self.request_arrange_parts();
                self.report(&before, ctx);

                return Propagation::Stop;
            }
            Key::Char(c)
                if !modifiers.intersects(Modifiers::CTRL | Modifiers::ALT | Modifiers::GUI) =>
            {
                return self.type_ahead(c, ctx);
            }
            _ => return Propagation::Continue,
        };

        if len == 0 {
            return Propagation::Stop;
        }

        if let Some(row) = target {
            self.move_cursor(row, extend);
            self.report(&before, ctx);
        }

        Propagation::Stop
    }

    /// Jumps to the next row whose text starts with the characters typed in quick succession.
    ///
    /// # Arguments
    ///
    /// * `c` - The character typed.
    /// * `ctx` - The context used to emit events.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] if a row matched, [`Propagation::Continue`] otherwise.
    fn type_ahead(&mut self, c: char, ctx: &mut EventCtx) -> Propagation {
        let now = Instant::now();

        if self
            .searched_at
            .is_none_or(|at| now.duration_since(at) > self.search_timeout)
        {
            self.search.clear();
        }

        self.searched_at = Some(now);
        self.search.extend(c.to_lowercase());

        let len = self.len;

        if len == 0 {
            return Propagation::Continue;
        }

        // A second character refines the search from the current row; the first (or a repeat
        // of it) looks from the next row.
        let first = self.search.chars().count() == 1;
        let start = self
            .cursor
            .map_or(0, |c| if first { (c + 1) % len } else { c });
        let found = (0..len)
            .map(|k| (start + k) % len)
            .find(|&i| self.model.item(i).to_lowercase().starts_with(&self.search));

        let Some(row) = found else {
            return Propagation::Continue;
        };
        let before = self.selected.clone();

        self.move_cursor(row, false);
        self.report(&before, ctx);

        Propagation::Stop
    }

    /// Returns the point a row's rectangle starts at, in the list's own coordinates.
    ///
    /// # Arguments
    ///
    /// * `row` - The model row.
    ///
    /// # Returns
    ///
    /// The upper left-hand corner, or `None` if the row is out of view.
    pub fn row_origin(&self, row: usize) -> Option<Point> {
        let slot = row.checked_sub(self.top)?;

        (slot < self.visible_rows() && row < self.len)
            .then(|| point(0, (slot as u32 * self.row_height) as i32))
    }
}
