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

//! # Alert
//!
//! A one-call modal alert: an icon, a title, a message, and a row of buttons.
//!
//! [`AlertExt::alert`] builds an [`Alert`] from existing `Widget`s, centers it in the window,
//! and opens it as a modal dialog ([`Ui::open_modal`]).  While it is open nothing else in the
//! window receives input.  Choosing a button closes it, removes it from the window, and calls
//! the completion closure with the button's index:
//!
//! * **Return** chooses the default button (drawn with a heavier border).
//! * **Escape** (or Cmd-. on macOS) chooses the last of the buttons given to `alert`.
//! * **Clicking** a button chooses it.
//!
//! Every piece is an ordinary part ([`AlertParts`]) that can be retrieved and restyled: the icon
//! is a `BoxedLabel`, the title and message are `Text`s, and each button is a `Button`.
//! Buttons can be added, removed, and made the default after the alert is open.
//!
//! ```text
//!  +-----------------------------------------------+
//!  |  [!]  Title                                   |
//!  |       Message, wrapped onto two lines.        |
//!  |                                               |
//!  |  [Help]              [[Delete]]   [Cancel]    |
//!  +-----------------------------------------------+
//!    added buttons         buttons given to alert
//! ```

use crate::boxed_label::BoxedLabel;
use crate::button::Button;
use crate::text::Text;
use pushrod::accessibility::{AccessInfo, Role};
use pushrod::cache::WidgetCache;
use pushrod::color::{Color, FontStyle};
use pushrod::composite::{Composite, PartList, PartsBuilder, TypedParts};
use pushrod::engine::Ui;
use pushrod::error::{Error, Result};
use pushrod::event::EventCtx;
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::paint::{self, Painter, Wrap};
use pushrod::widget::{Behavior, ROOT_WIDGET_ID, Widget, WidgetCore, WidgetId};

/// The alert's background color.
pub const ALERT_COLOR: Color = Color::RGB(236, 236, 236);

/// The message's text color.
pub const MESSAGE_COLOR: Color = Color::RGB(96, 96, 96);

/// The narrowest an alert gets, in pixels.
pub const MIN_WIDTH: u32 = 360;

/// The alert's height, in pixels.
pub const HEIGHT: u32 = 140;

/// The width of each button, in pixels.
pub const BUTTON_WIDTH: u32 = 84;

/// The height of each button, in pixels.
pub const BUTTON_HEIGHT: u32 = 24;

/// The space around the alert's contents, in pixels.
const PADDING: u32 = 16;

/// The space between two buttons in the same group, in pixels.
const BUTTON_GAP: u32 = 12;

/// The extra space between the added buttons (left) and the alert's own buttons (right).
const GROUP_GAP: u32 = 24;

/// The icon's width and height, in pixels.
const ICON_SIZE: u32 = 32;

/// The title's height, in pixels.
const TITLE_HEIGHT: u32 = 22;

/// The space between the title and the message, in pixels.
const TITLE_GAP: u32 = 6;

/// The message's height (two lines of the default font), in pixels.
const MESSAGE_HEIGHT: u32 = 40;

/// The title's font size, in points.
const TITLE_FONT_SIZE: u16 = 16;

/// The icon's font size, in points.
const ICON_FONT_SIZE: u16 = 20;

/// The list of parts that holds the buttons (`"button:0"`, `"button:1"`, ...).
const BUTTON_LIST: &str = "button";

/// What an alert is about; it picks the icon's symbol and colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertKind {
    /// Information: a white `i` on blue.
    #[default]
    Info,
    /// A warning: a black `!` on yellow.
    Warning,
    /// An error: a white `×` on red.
    Error,
    /// A question: a white `?` on dark blue.
    Question,
}

impl AlertKind {
    /// Returns the icon's symbol for this kind.
    ///
    /// # Returns
    ///
    /// The one-character symbol drawn in the icon.
    pub fn symbol(self) -> &'static str {
        match self {
            AlertKind::Info => "i",
            AlertKind::Warning => "!",
            AlertKind::Error => "\u{00d7}",
            AlertKind::Question => "?",
        }
    }

    /// Returns the icon's fill and symbol colors for this kind.
    ///
    /// # Returns
    ///
    /// `(fill, symbol)`.
    pub fn colors(self) -> (Color, Color) {
        match self {
            AlertKind::Info => (Color::RGB(64, 128, 224), Color::WHITE),
            AlertKind::Warning => (Color::RGB(255, 204, 0), Color::BLACK),
            AlertKind::Error => (Color::RGB(200, 32, 32), Color::WHITE),
            AlertKind::Question => (Color::RGB(48, 80, 144), Color::WHITE),
        }
    }
}

/// An alert's parts, by role.  Get them with [`TypedParts::typed_parts`] or
/// [`AlertHandle::parts`].
///
/// # Examples
///
/// ```rust
/// use pushrod::cache::WidgetCache;
/// use pushrod::composite::TypedParts;
/// use pushrod::geometry::size;
/// use pushrod::root_widget::RootWidget;
/// use pushrod_widgets::alert::{Alert, AlertKind};
///
/// let mut cache = WidgetCache::new();
/// cache.add(Box::new(RootWidget::new(size(480, 320))));
///
/// let alert = cache.add(Box::new(Alert::new(AlertKind::Info, "Saved", "", &["OK"], 0)));
/// let parts = cache.get_as::<Alert>(alert).unwrap().typed_parts();
///
/// assert_eq!(parts.buttons.len(), 1);
/// assert_eq!(cache.part(alert, "message"), Some(parts.message));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertParts {
    /// The icon: a `BoxedLabel` holding the kind's symbol.
    pub icon: WidgetId,
    /// The title: a `Text`.
    pub title: WidgetId,
    /// The message: a `Text` that wraps at spaces.
    pub message: WidgetId,
    /// The buttons, by index: each a `Button`.  The index is the one the completion closure
    /// receives.
    pub buttons: Vec<WidgetId>,
}

/// A modal alert built from parts: an icon, a title, a message, and buttons.
///
/// Usually opened with [`AlertExt::alert`], which also centers it, opens it as a modal dialog,
/// and removes it once a button is chosen.  See the [module documentation](self).
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::alert::{Alert, AlertKind, HEIGHT, MIN_WIDTH};
/// use pushrod::widget::Widget;
///
/// let alert = Alert::new(AlertKind::Warning, "Delete?", "This cannot be undone.", &["Delete", "Cancel"], 0);
///
/// assert_eq!(alert.get_size().w, MIN_WIDTH);
/// assert_eq!(alert.get_size().h, HEIGHT);
/// ```
#[derive(Debug)]
pub struct Alert {
    core: WidgetCore,
    parts: PartList,
    kind: AlertKind,
    title: String,
    message: String,
    labels: Vec<String>,
    default: usize,
    added: Vec<WidgetId>,
}

impl Widget for Alert {
    /// Returns the alert's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the alert's shared state for modification.
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
        let mut info = AccessInfo::of(self, Role::Dialog).with_name(&self.title);

        info.description = self.message.clone();
        info
    }

    /// Fills the alert with its color; the parts draw themselves on top.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the alert.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.style().background.unwrap_or(self.get_color()));
    }

    /// Returns the alert as a composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite(&self) -> Option<&dyn Composite> {
        Some(self)
    }

    /// Returns the alert as a mutable composite.
    ///
    /// # Returns
    ///
    /// `Some(self)`.
    fn as_composite_mut(&mut self) -> Option<&mut dyn Composite> {
        Some(self)
    }
}

impl Composite for Alert {
    /// Creates the icon, the title, the message, and one button per label.  Every button
    /// dismisses the alert; the default one also answers Return, and the last one Escape.
    ///
    /// # Arguments
    ///
    /// * `parts` - Adds the parts.
    fn build(&mut self, parts: &mut PartsBuilder) {
        let (fill, symbol) = self.kind.colors();
        let mut icon = BoxedLabel::new(self.kind.symbol())
            .font(Font::new("", ICON_FONT_SIZE, FontStyle::BOLD))
            .font_color(symbol)
            .border_color(Color::BLACK);

        icon.set_color(fill);
        parts.add("icon", icon);
        // The title and message share the alert's background.
        let mut title =
            Text::new(self.title.clone()).font(Font::new("", TITLE_FONT_SIZE, FontStyle::NORMAL));
        let mut message = Text::new(self.message.clone()).font_color(MESSAGE_COLOR);

        title.set_color(self.get_color());
        message.set_color(self.get_color());
        message.set_wrap(Wrap::Word);
        parts.add("title", title);
        parts.add("message", message);

        let last = self.labels.len().saturating_sub(1);

        for (index, label) in self.labels.iter().enumerate() {
            let mut button = Button::new(label.clone()).dismisses_dialog();

            if index == self.default {
                button = button.default_action();
            }

            if index == last {
                button = button.cancel_action();
            }

            parts.add(format!("{BUTTON_LIST}:{index}"), button);
        }
    }

    /// Returns the alert's parts.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list(&self) -> &PartList {
        &self.parts
    }

    /// Returns the alert's parts for modification.
    ///
    /// # Returns
    ///
    /// The [`PartList`].
    fn part_list_mut(&mut self) -> &mut PartList {
        &mut self.parts
    }

    /// Widens the alert to fit its buttons and centers it in the window.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the window's root, whose size the alert centers in.
    /// * `_ctx` - Unused; the alert emits no events of its own here.
    fn sync_parts(&mut self, cache: &mut WidgetCache, _ctx: &mut EventCtx) {
        let buttons = self.parts.list(BUTTON_LIST);

        self.added.retain(|id| buttons.contains(id));

        let added = self.added.len() as u32;
        let own = buttons.len() as u32 - added;
        let wanted = size(preferred_width(own, added), HEIGHT);

        if self.get_size() != wanted {
            self.set_size(wanted);
        }

        if self.get_parent() == ROOT_WIDGET_ID
            && let Some(root) = cache.get(ROOT_WIDGET_ID)
        {
            let window = root.get_size();
            let origin = point(
                (window.w.saturating_sub(wanted.w) / 2) as i32,
                (window.h.saturating_sub(wanted.h) / 2) as i32,
            );

            if self.get_origin() != origin {
                self.set_origin(origin);
            }
        }
    }

    /// Places the icon at the upper left, the title and message beside it, the buttons given to
    /// the alert in a row at the lower right, and added buttons at the lower left.
    ///
    /// # Returns
    ///
    /// Each part and its rectangle.
    fn layout_parts(&self) -> Vec<(WidgetId, Rect)> {
        let width = self.get_size().w;
        let pad = PADDING as i32;
        let text_x = pad + (ICON_SIZE + PADDING) as i32;
        let text_w = width.saturating_sub(text_x as u32 + PADDING);
        let button_y = (HEIGHT - PADDING - BUTTON_HEIGHT) as i32;
        let step = (BUTTON_WIDTH + BUTTON_GAP) as i32;
        let mut placements = Vec::new();
        let mut place = |name: &str, rect: Rect| {
            if let Some(id) = self.parts.get(name) {
                placements.push((id, rect));
            }
        };

        place("icon", Rect::new(pad, pad, ICON_SIZE, ICON_SIZE));
        place("title", Rect::new(text_x, pad, text_w, TITLE_HEIGHT));
        place(
            "message",
            Rect::new(
                text_x,
                pad + (TITLE_HEIGHT + TITLE_GAP) as i32,
                text_w,
                MESSAGE_HEIGHT,
            ),
        );

        let buttons = self.parts.list(BUTTON_LIST);
        let (added, own): (Vec<WidgetId>, Vec<WidgetId>) =
            buttons.into_iter().partition(|id| self.added.contains(id));
        let right = width as i32 - pad - BUTTON_WIDTH as i32;

        for (slot, id) in own.iter().rev().enumerate() {
            let rect = Rect::new(
                right - slot as i32 * step,
                button_y,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
            );

            placements.push((*id, rect));
        }

        for (slot, id) in added.iter().enumerate() {
            let rect = Rect::new(
                pad + slot as i32 * step,
                button_y,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
            );

            placements.push((*id, rect));
        }

        placements
    }
}

impl TypedParts for Alert {
    type Parts = AlertParts;

    /// Returns the alert's parts by role.
    ///
    /// # Returns
    ///
    /// The part IDs; before the alert is added to a `WidgetCache` they are the alert's own ID,
    /// and it has no buttons.
    fn typed_parts(&self) -> AlertParts {
        let find = |name: &str| self.parts.get(name).unwrap_or(self.get_id());

        AlertParts {
            icon: find("icon"),
            title: find("title"),
            message: find("message"),
            buttons: self.parts.list(BUTTON_LIST),
        }
    }
}

impl Alert {
    /// Creates an alert, [`MIN_WIDTH`] (or wider, to fit its buttons) by [`HEIGHT`] pixels.  Its
    /// parts are created when it is added to a `WidgetCache`.
    ///
    /// # Arguments
    ///
    /// * `kind` - Picks the icon.
    /// * `title` - The first line, in a larger font.
    /// * `message` - The text under the title; it wraps onto two lines.
    /// * `buttons` - The button labels, left to right.  An empty list gets one `OK` button.
    /// * `default` - The index of the button Return chooses; past the end means the last one.
    ///
    /// # Returns
    ///
    /// The new `Alert`.
    pub fn new(
        kind: AlertKind,
        title: impl Into<String>,
        message: impl Into<String>,
        buttons: &[&str],
        default: usize,
    ) -> Self {
        let labels: Vec<String> = if buttons.is_empty() {
            vec!["OK".to_string()]
        } else {
            buttons.iter().map(|label| (*label).to_string()).collect()
        };
        let default = default.min(labels.len() - 1);
        let core = WidgetCore::new(
            point(0, 0),
            size(preferred_width(labels.len() as u32, 0), HEIGHT),
        );
        let mut alert = Self {
            core,
            parts: PartList::new(),
            kind,
            title: title.into(),
            message: message.into(),
            labels,
            default,
            added: Vec::new(),
        };

        alert.set_color(ALERT_COLOR);
        alert
    }

    /// Returns what the alert is about.
    ///
    /// # Returns
    ///
    /// The [`AlertKind`] it was created with.
    pub fn kind(&self) -> AlertKind {
        self.kind
    }

    /// Returns the index of a button.
    ///
    /// # Arguments
    ///
    /// * `button` - A button's `Widget` ID.
    ///
    /// # Returns
    ///
    /// Its index in [`AlertParts::buttons`], or `None` if it is not one of the alert's buttons.
    pub fn button_index(&self, button: WidgetId) -> Option<usize> {
        self.parts
            .list(BUTTON_LIST)
            .iter()
            .position(|id| *id == button)
    }

    /// Adds a button after the alert's buttons; its index is the next one.  It is placed at the
    /// lower left, apart from the buttons given to the alert, and it dismisses the alert like
    /// they do.  Escape keeps choosing the same button as before.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the alert.
    /// * `alert` - The alert's ID.
    /// * `label` - The button's label.
    ///
    /// # Returns
    ///
    /// The new button's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `alert` does not exist, [`Error::Composite`] if it is not an
    /// `Alert`.
    pub fn add_button(
        cache: &mut WidgetCache,
        alert: WidgetId,
        label: impl Into<String>,
    ) -> Result<WidgetId> {
        Self::checked(cache, alert)?;

        let button = Button::new(label).dismisses_dialog();
        let id = cache.insert_list_part(alert, BUTTON_LIST, usize::MAX, Box::new(button))?;

        if let Some(owner) = cache.get_as_mut::<Alert>(alert) {
            owner.added.push(id);
        }

        cache.arrange_parts(alert)?;
        Ok(id)
    }

    /// Removes a button; the buttons after it move down one index.  If it was the Escape button,
    /// Escape moves to the last of the remaining buttons given to the alert (or the last button).
    /// If it was the default, Return does nothing until [`Alert::set_default_button`] is called.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the alert.
    /// * `alert` - The alert's ID.
    /// * `index` - The button's index.
    ///
    /// # Returns
    ///
    /// The removed button's ID.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `alert` does not exist; [`Error::Composite`] if it is not an
    /// `Alert`, has no button at `index`, or `index` is its only button (an alert always keeps
    /// one button, so it can be dismissed).
    pub fn remove_button(
        cache: &mut WidgetCache,
        alert: WidgetId,
        index: usize,
    ) -> Result<WidgetId> {
        let buttons = Self::checked(cache, alert)?.parts.list(BUTTON_LIST);
        let Some(&button) = buttons.get(index) else {
            return Err(Error::composite(alert, format!("has no button {index}")));
        };

        if buttons.len() == 1 {
            return Err(Error::composite(alert, "cannot remove its only button"));
        }

        let was_cancel = cache
            .get(button)
            .is_some_and(|b| b.behavior().contains(Behavior::CANCEL_ACTION));
        let removed = cache.remove_list_part(alert, BUTTON_LIST, index)?;

        if was_cancel {
            let alert_widget = Self::checked(cache, alert)?;
            let remaining = alert_widget.parts.list(BUTTON_LIST);
            let next = remaining
                .iter()
                .rev()
                .find(|id| !alert_widget.added.contains(id))
                .or(remaining.last())
                .copied();

            if let Some(next) = next {
                set_flag(cache, next, Behavior::CANCEL_ACTION, true);
            }
        }

        Ok(removed)
    }

    /// Makes a button the default: Return chooses it, and it is drawn with a heavier border.
    /// The previous default stops being one.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the alert.
    /// * `alert` - The alert's ID.
    /// * `index` - The button's index.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the button at `index` is the default.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `alert` does not exist; [`Error::Composite`] if it is not an
    /// `Alert` or has no button at `index`.
    pub fn set_default_button(
        cache: &mut WidgetCache,
        alert: WidgetId,
        index: usize,
    ) -> Result<()> {
        let buttons = Self::checked(cache, alert)?.parts.list(BUTTON_LIST);

        if index >= buttons.len() {
            return Err(Error::composite(alert, format!("has no button {index}")));
        }

        for (position, id) in buttons.into_iter().enumerate() {
            set_flag(cache, id, Behavior::DEFAULT_ACTION, position == index);
        }

        Ok(())
    }

    /// Returns the alert with this ID.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache holding the alert.
    /// * `alert` - The alert's ID.
    ///
    /// # Returns
    ///
    /// The alert.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `alert` does not exist, [`Error::Composite`] if it is not an
    /// `Alert`.
    fn checked(cache: &WidgetCache, alert: WidgetId) -> Result<&Alert> {
        cache.get(alert).ok_or(Error::UnknownWidget { id: alert })?;
        cache
            .get_as::<Alert>(alert)
            .ok_or_else(|| Error::composite(alert, "is not an alert"))
    }
}

/// Returns how wide an alert must be to fit its buttons.
///
/// # Arguments
///
/// * `own` - How many of the buttons were given to the alert (the right-hand group).
/// * `added` - How many were added later (the left-hand group).
///
/// # Returns
///
/// The width, at least [`MIN_WIDTH`].
fn preferred_width(own: u32, added: u32) -> u32 {
    let group = |count: u32| count * BUTTON_WIDTH + count.saturating_sub(1) * BUTTON_GAP;
    let mut width = 2 * PADDING + group(own);

    if added > 0 {
        width += group(added) + GROUP_GAP;
    }

    width.max(MIN_WIDTH)
}

/// Sets or clears one behavior flag on a `Widget`, if it exists.
///
/// # Arguments
///
/// * `cache` - The cache holding the `Widget`.
/// * `id` - The `Widget`'s ID.
/// * `flag` - The flag.
/// * `on` - Whether the flag is set.
fn set_flag(cache: &mut WidgetCache, id: WidgetId, flag: Behavior, on: bool) {
    if let Some(widget) = cache.get_mut(id) {
        let mut behavior = widget.behavior();

        if behavior.contains(flag) != on {
            behavior.set(flag, on);
            widget.set_behavior(behavior);
        }
    }
}

/// A handle to an open alert, returned by [`AlertExt::alert`].
///
/// It holds the alert's ID and a copy of its parts, which its methods keep up to date.  A handle
/// moved into a callback may fall behind changes made elsewhere; read the current parts with
/// `ui.get::<Alert>(handle.id())` and [`TypedParts::typed_parts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertHandle {
    id: WidgetId,
    parts: AlertParts,
}

impl AlertHandle {
    /// Returns the alert's `Widget` ID.
    ///
    /// # Returns
    ///
    /// The ID of the [`Alert`].
    pub fn id(&self) -> WidgetId {
        self.id
    }

    /// Returns the alert's parts, as of the last change made through this handle.
    ///
    /// # Returns
    ///
    /// The [`AlertParts`].
    pub fn parts(&self) -> &AlertParts {
        &self.parts
    }

    /// Adds a button; see [`Alert::add_button`].
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback's [`Ui`].
    /// * `label` - The button's label.
    ///
    /// # Returns
    ///
    /// The new button's ID, or `None` if the alert has been closed.
    pub fn add_button<S>(&mut self, ui: &mut Ui<S>, label: impl Into<String>) -> Option<WidgetId> {
        let id = Alert::add_button(ui.widgets(), self.id, label).ok();

        self.refresh(ui);
        id
    }

    /// Removes a button; see [`Alert::remove_button`].
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback's [`Ui`].
    /// * `index` - The button's index.
    ///
    /// # Returns
    ///
    /// `true` if the button was removed; `false` if there is no such button, it is the only one,
    /// or the alert has been closed.
    pub fn remove_button<S>(&mut self, ui: &mut Ui<S>, index: usize) -> bool {
        let removed = Alert::remove_button(ui.widgets(), self.id, index).is_ok();

        self.refresh(ui);
        removed
    }

    /// Makes a button the default; see [`Alert::set_default_button`].
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback's [`Ui`].
    /// * `index` - The button's index.
    ///
    /// # Returns
    ///
    /// `true` if it is now the default; `false` if there is no such button, or the alert has
    /// been closed.
    pub fn set_default_button<S>(&mut self, ui: &mut Ui<S>, index: usize) -> bool {
        Alert::set_default_button(ui.widgets(), self.id, index).is_ok()
    }

    /// Reads the alert's parts again.  Nothing changes if the alert has been closed.
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback's [`Ui`].
    fn refresh<S>(&mut self, ui: &mut Ui<S>) {
        if let Some(alert) = ui.get::<Alert>(self.id) {
            self.parts = alert.typed_parts();
        }
    }
}

/// Adds [`alert`](AlertExt::alert) to [`Ui`], so any callback can open an alert.
///
/// It is in the [prelude](crate::prelude).
pub trait AlertExt<S> {
    /// Builds an [`Alert`], centers it in the window, and opens it as a modal dialog.
    ///
    /// While it is open nothing else in the window receives input.  When a button is chosen
    /// (clicked, or Return for the default, or Escape for the last of `buttons`), the alert is
    /// closed and removed from the window, the button's own callbacks run, and then `completion`
    /// runs once with the button's index.  `completion` does not run if the alert is removed
    /// another way.
    ///
    /// Building and closing it emits the usual engine events: `WidgetAdded` for the alert and
    /// each part, `PartsArranged`, `ModalOpened`, `DialogDismissRequested`, `ModalClosed`, and
    /// `WidgetRemoved`.
    ///
    /// # Arguments
    ///
    /// * `kind` - Picks the icon.
    /// * `title` - The first line, in a larger font.
    /// * `message` - The text under the title; it wraps onto two lines.
    /// * `buttons` - The button labels, left to right.  An empty list gets one `OK` button.
    /// * `default` - The index of the button Return chooses; past the end means the last one.
    /// * `completion` - Runs once with the application state, a [`Ui`], and the chosen
    ///   button's index.
    ///
    /// # Returns
    ///
    /// A handle for retrieving and changing the alert's parts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::engine::Engine;
    /// use pushrod::geometry::size;
    /// use pushrod_widgets::alert::{AlertExt, AlertKind};
    ///
    /// let mut engine: Engine<Option<usize>> = Engine::new(size(480, 320), 30);
    ///
    /// engine.on_idle(|_, ui| {
    ///     if ui.widgets().top_modal().is_none() {
    ///         ui.alert(AlertKind::Warning, "Delete \"report.txt\"?", "This cannot be undone.",
    ///                  &["Delete", "Cancel"], 0, |chosen, _ui, choice| *chosen = Some(choice));
    ///     }
    /// });
    ///
    /// let mut chosen = None;
    ///
    /// engine.run_idle(&mut chosen);
    /// assert!(engine.widgets().top_modal().is_some());
    /// ```
    fn alert<F>(
        &mut self,
        kind: AlertKind,
        title: &str,
        message: &str,
        buttons: &[&str],
        default: usize,
        completion: F,
    ) -> AlertHandle
    where
        F: FnOnce(&mut S, &mut Ui<S>, usize) + 'static;
}

impl<S> AlertExt<S> for Ui<'_, S> {
    /// See [`AlertExt::alert`].
    ///
    /// # Arguments
    ///
    /// * `kind` - Picks the icon.
    /// * `title` - The first line.
    /// * `message` - The text under the title.
    /// * `buttons` - The button labels.
    /// * `default` - The index of the default button.
    /// * `completion` - Runs once with the chosen button's index.
    ///
    /// # Returns
    ///
    /// A handle to the alert.
    fn alert<F>(
        &mut self,
        kind: AlertKind,
        title: &str,
        message: &str,
        buttons: &[&str],
        default: usize,
        completion: F,
    ) -> AlertHandle
    where
        F: FnOnce(&mut S, &mut Ui<S>, usize) + 'static,
    {
        let widget = Alert::new(kind, title, message, buttons, default);
        let id = self.widgets().add_child(ROOT_WIDGET_ID, Box::new(widget));
        let parts = self
            .get::<Alert>(id)
            .map(|alert| alert.typed_parts())
            .unwrap_or(AlertParts {
                icon: id,
                title: id,
                message: id,
                buttons: Vec::new(),
            });

        let opened = self.open_modal(id, move |state, ui, dismissed_by| {
            let choice = ui
                .get::<Alert>(id)
                .and_then(|alert| alert.button_index(dismissed_by));

            // The alert was built for this one use; it is closed, so take it out of the window.
            let _ = ui.widgets().remove(id);

            if let Some(choice) = choice {
                completion(state, ui, choice);
            }
        });

        // A freshly added child of the root can always be opened; if that ever changes, do not
        // leave an alert in the window that nothing can dismiss.
        if opened.is_err() {
            let _ = self.widgets().remove(id);
        }

        AlertHandle { id, parts }
    }
}
