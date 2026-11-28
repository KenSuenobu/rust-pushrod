// TabView Widget
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

//! # TabView
//!
//! A strip of tabs above a stack of pages, one page shown at a time.  Settings dialogs use it to
//! group many controls into pages.
//!
//! `TabView` is a [`Composite`].  Each page is an ordinary [`Panel`] part with its own ID, and
//! each tab is an ordinary toggle [`Button`] part, so the application reaches them with the usual
//! API: add controls to a page with `Engine::add_child_widget(page, ..)`, or restyle a tab with
//! `ui.get_mut::<Button>(tab)`.
//!
//! ```text
//!   TabView
//!    ├── "tab:0"   Button  "General"    <- inverted while its page is shown
//!    ├── "tab:1"   Button  "Advanced"
//!    ├── "page:0"  Panel   (shown)
//!    └── "page:1"  Panel   (hidden: its children keep their IDs and state)
//! ```
//!
//! Switching pages **hides** the other pages; it never removes them, so the controls inside a
//! hidden page keep their IDs, values, and callbacks.  A hidden page is not drawn, receives no
//! mouse events, and is skipped by the Tab key.  If the keyboard focus was inside the page that
//! was hidden, it moves to the new page's tab.
//!
//! The user switches pages by clicking a tab, or with **Ctrl+Tab** (next page) and
//! **Ctrl+Shift+Tab** (previous page), wrapping around at the ends; the shortcut works while the
//! focus is on a tab or on a control inside any page.
//!
//! It emits `SelectionChanged { id }` whenever the shown page changes, whether the user, the
//! application ([`TabView::select_page`]), or a page being removed changed it; read the page
//! with [`TabView::selected_page`].  A new `TabView` shows page 0 and does not report that as a
//! change.
//!
//! ```mermaid
//! flowchart LR
//!   C[Click a tab] --> S[selected page changes]
//!   K[Ctrl+Tab] --> S
//!   A["select_page()"] --> S
//!   S --> H[other pages hidden, tab inverted]
//!   S --> E[SelectionChanged]
//! ```
//!
//! # Examples
//!
//! ```rust
//! use pushrod_widgets::tab_view::TabView;
//!
//! let tabs = TabView::new()
//!     .at(10, 10)
//!     .size(300, 200)
//!     .with_page("General")
//!     .with_page("Advanced");
//!
//! assert_eq!(tabs.selected_page(), None); // no pages exist until it is added to a window
//! ```

use pushrod::accessibility::{AccessInfo, Role};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::error::{Error, Result};
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{Key, Modifiers};
use pushrod::paint::{self, Painter};
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};

use crate::button::Button;
use crate::panel::Panel;

/// The name, before the colon, of the list of tabs: `"tab:0"`, `"tab:1"`, ...
const TAB_LIST: &str = "tab";

/// The name, before the colon, of the list of pages: `"page:0"`, `"page:1"`, ...
const PAGE_LIST: &str = "page";

/// The height of the tab strip, in pixels.
const TAB_HEIGHT: u32 = 24;

/// The room kept for the margins of a tab's label, in pixels.
const TAB_PADDING: u32 = 20;

/// The width one character of a tab's label is given, in pixels.
const TAB_CHAR_WIDTH: u32 = 8;

/// The narrowest a tab is made, in pixels.
const TAB_MIN_WIDTH: u32 = 56;

/// The width of the border drawn around the pages, in pixels.
const BORDER: u32 = 1;

/// Returns the width a tab is given for its label.
///
/// # Arguments
///
/// * `label` - The tab's label.
///
/// # Returns
///
/// The width in pixels: a width per character plus margins, but at least the minimum.
fn tab_width(label: &str) -> u32 {
    let characters = label.chars().count() as u32;

    (TAB_PADDING + characters * TAB_CHAR_WIDTH).max(TAB_MIN_WIDTH)
}

/// A `TabView`'s parts by role.
///
/// The tabs and pages are in the same order: `tabs[i]` switches to `pages[i]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabViewParts {
    /// The tab buttons, left to right.
    pub tabs: Vec<WidgetId>,
    /// The page panels, in the same order as the tabs.
    pub pages: Vec<WidgetId>,
}

/// A strip of tabs above a stack of pages, one page shown at a time.  See the
/// [module documentation](self).
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::tab_view::TabView;
///
/// let tabs = TabView::new().size(320, 240).with_page("General");
/// ```
#[derive(Debug)]
pub struct TabView {
    core: WidgetCore,
    parts: PartList,
    /// Titles of pages to create when the view is built.
    pending: Vec<String>,
    /// The shown page, `None` until the first arrangement and while there are no pages.
    selected: Option<WidgetId>,
    /// Where the shown page was, to pick its neighbor when it is removed.
    index_hint: usize,
    /// The page last reported, `None` until the first arrangement.
    reported: Option<WidgetId>,
    /// The width of each tab, from its label, as last arranged.
    widths: Vec<u32>,
}

impl Widget for TabView {
    /// Returns the tab view's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the tab view's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Describes the widget to a screen reader.
    ///
    /// # Returns
    ///
    /// The widget's role, name, value, state, and actions.
    fn accessibility(&self) -> AccessInfo {
        AccessInfo::of(self, Role::Group)
    }

    /// Fills the background and draws the one-pixel border around the pages, leaving the strip
    /// of tabs above it.  The tabs and pages draw themselves on top.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        let frame = self.get_size();
        let style = self.style();

        paint::fill(p, style.background.unwrap_or(self.get_color()));

        if frame.h <= TAB_HEIGHT || frame.w == 0 {
            return;
        }

        let color = style.border_color.unwrap_or(Color::BLACK);
        let top = TAB_HEIGHT - BORDER;
        let height = frame.h - top;
        let edge = BORDER.min(frame.w);

        p.fill_rect(Rect::new(0, top as i32, frame.w, BORDER), color);
        p.fill_rect(Rect::new(0, top as i32, edge, height), color);
        p.fill_rect(
            Rect::new((frame.w - edge) as i32, top as i32, edge, height),
            color,
        );
        p.fill_rect(
            Rect::new(0, (frame.h - BORDER.min(height)) as i32, frame.w, BORDER),
            color,
        );
    }

    /// Switches pages on Ctrl+Tab (next) and Ctrl+Shift+Tab (previous), wrapping around, when the
    /// key reaches the view from a tab or from a control in a page.
    ///
    /// # Arguments
    ///
    /// * `event` - The event.
    /// * `_ctx` - Not used.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for the shortcut, otherwise [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, _ctx: &mut EventCtx) -> Propagation {
        let Event::KeyPressed {
            key: Key::Tab,
            modifiers,
            ..
        } = event
        else {
            return Propagation::Continue;
        };

        let count = self.parts.list(PAGE_LIST).len();

        if !modifiers.contains(Modifiers::CTRL) || count < 2 {
            return Propagation::Continue;
        }

        let current = self.selected_page().unwrap_or(0);
        let next = if modifiers.contains(Modifiers::SHIFT) {
            (current + count - 1) % count
        } else {
            (current + 1) % count
        };

        self.select_page(next);
        Propagation::Stop
    }

    /// Returns the tab view as a composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    /// Returns the tab view as a mutable composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for TabView {
    /// Creates a tab and a page for each page given with [`TabView::with_page`].  Tabs are
    /// created first, so they draw over the pages' border.
    ///
    /// # Arguments
    ///
    /// * `parts` - Adds the parts.
    fn build(&mut self, parts: &mut PartsBuilder) {
        let titles = std::mem::take(&mut self.pending);

        for (i, title) in titles.iter().enumerate() {
            parts.add(format!("{PAGE_LIST}:{i}"), new_page());
            parts.add(format!("{TAB_LIST}:{i}"), new_tab(title));
        }
    }

    /// Returns the tab view's parts.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list(&self) -> &PartList {
        &self.parts
    }

    /// Returns the tab view's parts for modification.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    /// Brings the parts up to date: keeps the selection in range, shows only the selected page,
    /// inverts only the selected tab, moves the focus out of a page that was hidden, widens the
    /// tabs to their labels, and emits `SelectionChanged` if the page changed since last time.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the parts.
    /// * `ctx` - Queues `SelectionChanged`.
    fn sync_parts(&mut self, cache: &mut WidgetCache, ctx: &mut EventCtx) {
        let parts = self.typed_parts();

        if parts.pages.is_empty() {
            self.selected = None;
            self.reported = None;
            self.index_hint = 0;
            return;
        }

        // The shown page, or its neighbor if it was removed, or the first page at the start.
        let shown = self
            .selected
            .filter(|id| parts.pages.contains(id))
            .unwrap_or(parts.pages[self.index_hint.min(parts.pages.len() - 1)]);
        let shown_index = parts.pages.iter().position(|&p| p == shown).unwrap_or(0);

        self.selected = Some(shown);
        self.index_hint = shown_index;

        let widths: Vec<u32> = parts
            .tabs
            .iter()
            .map(|&tab| {
                cache
                    .get_as::<Button>(tab)
                    .map_or(TAB_MIN_WIDTH, |b| tab_width(&b.get_text()))
            })
            .collect();

        if widths != self.widths {
            self.widths = widths;
            self.request_arrange_parts();
        }

        for &page in &parts.pages {
            if let Some(widget) = cache.get_mut(page)
                && widget.is_hidden() != (page != shown)
            {
                widget.set_hidden(page != shown);
            }
        }

        for (i, &tab) in parts.tabs.iter().enumerate() {
            if let Some(button) = cache.get_as_mut::<Button>(tab) {
                button.set_checked(i == shown_index);
            }
        }

        if let Some(focused) = cache.focused()
            && hides_focus(cache, focused, &parts.pages, shown)
            && let Some(&tab) = parts.tabs.get(shown_index)
        {
            cache.set_focus(tab);
        }

        if self.reported.is_some_and(|before| before != shown) {
            ctx.emit(Event::SelectionChanged { id: self.get_id() });
        }

        self.reported = Some(shown);
    }

    /// Places the tabs in a row along the top, each as wide as its label, and every page in the
    /// area under them, inside the border.
    ///
    /// # Returns
    ///
    /// Each part and its rectangle.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let frame = self.get_size();
        let body = Rect::new(
            BORDER as i32,
            TAB_HEIGHT as i32,
            frame.w.saturating_sub(2 * BORDER),
            frame.h.saturating_sub(TAB_HEIGHT + BORDER),
        );
        let mut placed = Vec::new();
        let mut x = 0;

        for (i, id) in self.parts.list(TAB_LIST).into_iter().enumerate() {
            let width = self.widths.get(i).copied().unwrap_or(TAB_MIN_WIDTH);

            placed.push((id, Rect::new(x, 0, width, TAB_HEIGHT)));
            x += width as i32;
        }

        placed.extend(self.parts.list(PAGE_LIST).into_iter().map(|id| (id, body)));
        placed
    }

    /// Switches to a tab's page when the tab is clicked.
    ///
    /// # Arguments
    ///
    /// * `part` - The part that emitted the event.
    /// * `event` - The event.
    /// * `_ctx` - Not used.
    fn handle_part_event(&mut self, part: WidgetId, event: &Event, _ctx: &mut EventCtx) {
        if matches!(event, Event::Toggled { .. })
            && let Some(index) = self.parts.list(TAB_LIST).iter().position(|&t| t == part)
        {
            self.select_page(index);
            // Clicking the shown tab flips its toggle; arrange again to check it back.
            self.request_arrange_parts();
        }
    }
}

impl TypedParts for TabView {
    type Parts = TabViewParts;

    /// Returns the tab view's parts by role.
    ///
    /// # Returns
    ///
    /// The tab and page IDs; both are empty before the view is added to a `WidgetCache`.
    fn typed_parts(&self) -> TabViewParts {
        TabViewParts {
            tabs: self.parts.list(TAB_LIST),
            pages: self.parts.list(PAGE_LIST),
        }
    }
}

/// Creates a page: a transparent panel that fills the area under the tabs.
///
/// # Returns
///
/// The panel.
fn new_page() -> Panel {
    Panel::new()
}

/// Creates a tab: a focusable toggle button, inverted while its page is shown.
///
/// # Arguments
///
/// * `title` - The label.
///
/// # Returns
///
/// The button.
fn new_tab(title: &str) -> Button {
    let mut tab = Button::new(title)
        .toggle()
        .size(tab_width(title), TAB_HEIGHT);

    // Focusable, so the keyboard reaches the tabs (Tab, then Space or Return, or Ctrl+Tab).
    tab.set_behavior(tab.behavior() | Behavior::FOCUSABLE);
    tab
}

impl TabView {
    /// Creates a tab view with no pages, 200 by 150 pixels at the upper left-hand corner of its
    /// parent.  Add pages with [`TabView::with_page`] before it is added to a window, or
    /// [`TabView::add_page`] after.
    ///
    /// # Returns
    ///
    /// The new `TabView`.
    pub fn new() -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(200, 150));

        core.set_event_mask(EventMask::KEYBOARD);

        Self {
            core,
            parts: PartList::new(),
            pending: Vec::new(),
            selected: None,
            index_hint: 0,
            reported: None,
            widths: Vec::new(),
        }
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
    /// The tab view, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height, including the strip of tabs.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The tab view, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        Widget::set_size(&mut self, size(width, height));
        self
    }

    /// Queues a page, with a tab labeled `title`, to be created when the view is added to a
    /// window.  Pages appear in the order given.
    ///
    /// # Arguments
    ///
    /// * `title` - The tab's label.
    ///
    /// # Returns
    ///
    /// The tab view, for chaining.
    #[must_use]
    pub fn with_page(mut self, title: impl Into<String>) -> Self {
        self.pending.push(title.into());
        self
    }

    /// Returns the index of the shown page.
    ///
    /// # Returns
    ///
    /// The page's index, or `None` if the view has no pages (or is not yet in a window).
    pub fn selected_page(&self) -> Option<usize> {
        let pages = self.parts.list(PAGE_LIST);

        match self
            .selected
            .and_then(|id| pages.iter().position(|&p| p == id))
        {
            Some(index) => Some(index),
            None => (!pages.is_empty()).then(|| self.index_hint.min(pages.len() - 1)),
        }
    }

    /// Returns how many pages there are.
    ///
    /// # Returns
    ///
    /// The number of pages.
    pub fn page_count(&self) -> usize {
        self.parts.list(PAGE_LIST).len()
    }

    /// Returns every tab button, left to right.  Each can be restyled like any `Button`
    /// (`ui.get_mut::<Button>(tab)`): its font, color, and text.
    ///
    /// # Returns
    ///
    /// The tab IDs.
    pub fn tabs(&self) -> Vec<WidgetId> {
        self.parts.list(TAB_LIST)
    }

    /// Returns every page, in the same order as the tabs.  Add controls to a page with
    /// `Engine::add_child_widget(page, ..)`.
    ///
    /// # Returns
    ///
    /// The page IDs.
    pub fn pages(&self) -> Vec<WidgetId> {
        self.parts.list(PAGE_LIST)
    }

    /// Returns the ID of the shown page.
    ///
    /// # Returns
    ///
    /// The page's ID, or `None` if there are no pages.
    pub fn selected_page_id(&self) -> Option<WidgetId> {
        self.selected_page()
            .and_then(|i| self.pages().get(i).copied())
    }

    /// Shows a page.  The other pages are hidden at the next frame and, if the page changed,
    /// `SelectionChanged` is emitted then.
    ///
    /// # Arguments
    ///
    /// * `index` - The page's index.
    ///
    /// # Returns
    ///
    /// `true` if there is such a page; `false` (and nothing changes) if `index` is past the
    /// last page.
    pub fn select_page(&mut self, index: usize) -> bool {
        let Some(&page) = self.parts.list(PAGE_LIST).get(index) else {
            return false;
        };

        if self.selected != Some(page) {
            self.selected = Some(page);
            self.index_hint = index;
            self.request_arrange_parts();
        }

        true
    }

    /// Adds a page, with a tab at the right end of the strip.  The shown page does not change.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the view.
    /// * `view` - The tab view's ID.
    /// * `title` - The tab's label.
    ///
    /// # Returns
    ///
    /// The new tab's ID and the new page's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `view` does not exist, [`Error::Composite`] if it is not a
    /// `TabView`.
    pub fn add_page(
        cache: &mut WidgetCache,
        view: WidgetId,
        title: impl Into<String>,
    ) -> Result<(WidgetId, WidgetId)> {
        Self::checked(cache, view)?;

        let title = title.into();
        let page = cache.insert_list_part(view, PAGE_LIST, usize::MAX, Box::new(new_page()))?;
        let tab = cache.insert_list_part(view, TAB_LIST, usize::MAX, Box::new(new_tab(&title)))?;

        cache.arrange_parts(view)?;
        Ok((tab, page))
    }

    /// Removes a page, its tab, and everything inside the page.  If the shown page is removed the
    /// next page (or, for the last, the previous one) is shown; if an earlier page is removed the
    /// same page stays shown.  `SelectionChanged` is emitted if the shown page is a different one.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the view.
    /// * `view` - The tab view's ID.
    /// * `index` - The page's index.
    ///
    /// # Returns
    ///
    /// The removed tab's ID and page's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `view` does not exist; [`Error::Composite`] if it is not a
    /// `TabView` or has no page `index`.
    pub fn remove_page(
        cache: &mut WidgetCache,
        view: WidgetId,
        index: usize,
    ) -> Result<(WidgetId, WidgetId)> {
        let count = Self::checked(cache, view)?.page_count();

        if index >= count {
            return Err(Error::composite(view, format!("has no page {index}")));
        }

        let page = cache.remove_list_part(view, PAGE_LIST, index)?;
        let tab = cache.remove_list_part(view, TAB_LIST, index)?;

        // The shown page is tracked by ID, so it stays shown unless it was the one removed; then
        // its neighbor takes its place.
        if let Some(owner) = cache.get_as_mut::<TabView>(view) {
            owner.index_hint = index;
            owner.request_arrange_parts();
        }

        cache.arrange_parts(view)?;
        Ok((tab, page))
    }

    /// Retitles a page's tab and widens or narrows the tab to fit.  (Setting the text of the
    /// tab's `Button` directly also works, but the tab keeps its width until the next time the
    /// view is arranged.)
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the view.
    /// * `view` - The tab view's ID.
    /// * `index` - The page's index.
    /// * `title` - The new label.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the tab is retitled.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `view` does not exist; [`Error::Composite`] if it is not a
    /// `TabView` or has no page `index`.
    pub fn set_page_title(
        cache: &mut WidgetCache,
        view: WidgetId,
        index: usize,
        title: impl Into<String>,
    ) -> Result<()> {
        let tab = Self::checked(cache, view)?
            .tabs()
            .get(index)
            .copied()
            .ok_or_else(|| Error::composite(view, format!("has no page {index}")))?;

        if let Some(button) = cache.get_as_mut::<Button>(tab) {
            button.set_text(title.into());
        }

        if let Some(owner) = cache.get_as_mut::<TabView>(view) {
            owner.request_arrange_parts();
        }

        cache.arrange_parts(view)?;
        Ok(())
    }

    /// Returns the `TabView` with an ID.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding it.
    /// * `view` - The ID.
    ///
    /// # Returns
    ///
    /// The tab view.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if there is no such `Widget`, [`Error::Composite`] if it is not a
    /// `TabView`.
    fn checked(cache: &WidgetCache, view: WidgetId) -> Result<&TabView> {
        if cache.get(view).is_none() {
            return Err(Error::UnknownWidget { id: view });
        }

        cache
            .get_as::<TabView>(view)
            .ok_or_else(|| Error::composite(view, "is not a TabView"))
    }
}

/// Returns whether the keyboard focus is inside a page that is now hidden.
///
/// # Arguments
///
/// * `cache` - The cache holding the focus.
/// * `focused` - The focused `Widget`.
/// * `pages` - The pages.
/// * `shown` - The page that is shown.
///
/// # Returns
///
/// `true` if `focused` is a page other than the shown one, or is below one.
fn hides_focus(
    cache: &WidgetCache,
    focused: WidgetId,
    pages: &[WidgetId],
    shown: WidgetId,
) -> bool {
    let mut at = focused;

    for _ in 0..=cache.len() {
        if pages.contains(&at) {
            return at != shown;
        }

        let Some(widget) = cache.get(at) else {
            return false;
        };
        let parent = widget.get_parent();

        if parent == at {
            return false;
        }

        at = parent;
    }

    false
}

impl Default for TabView {
    /// Creates a tab view with no pages; see [`TabView::new`].
    ///
    /// # Returns
    ///
    /// The new `TabView`.
    fn default() -> Self {
        Self::new()
    }
}
