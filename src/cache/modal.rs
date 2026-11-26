// Modal
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

//! Modal dialogs for the [`WidgetCache`]: a stack of open dialogs, the rule that only the top
//! one receives input, and closing a dialog when a [`Behavior::DISMISSES_DIALOG`] `Widget` is
//! activated.
//!
//! While a modal dialog is open, every `Widget` outside its subtree is still drawn but is
//! **inert**: it receives no pointer, key, or text input, cannot take focus, and its default
//! and cancel `Widget`s do not answer Return and Escape.  Timers, animation ticks, custom
//! events, and broadcasts are not input and still reach it.
//!
//! **Layers.**  Each open dialog is drawn as its own layer, after everything beneath it.  Before
//! a layer is drawn, everything already in the frame is darkened and softened (see
//! [`ModalStyle`]), and the dialog gets a GEM-style outlined border, so every nested dialog adds
//! one more visible step of modality.

use super::WidgetCache;
use crate::backend::Backend;
use crate::color::Color;
use crate::error::{Error, Result};
use crate::event::{EngineEvent, Event};
use crate::geometry::Rect;
use crate::widget::{Behavior, ROOT_WIDGET_ID, WidgetId};

/// How open modal dialogs are drawn: the backdrop laid over everything beneath each dialog, and
/// the border drawn around it.
///
/// The defaults follow GEM's `OUTLINED` dialogs: a one-pixel black border, a two-pixel white
/// gap, and a one-pixel black outline, over a slightly darkened and softened window.
///
/// # Examples
///
/// ```rust
/// use pushrod::cache::{ModalStyle, WidgetCache};
/// use pushrod::color::Color;
///
/// let mut cache = WidgetCache::new();
///
/// // A plain one-pixel border and a darker backdrop, with no blur.
/// cache.set_modal_style(ModalStyle {
///     shade: Color::RGBA(0, 0, 0, 96),
///     blur: 0,
///     outlined: false,
///     ..ModalStyle::default()
/// });
///
/// assert_eq!(cache.modal_style().blur, 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalStyle {
    /// The color laid over everything beneath a dialog.  Its alpha sets how much it darkens;
    /// fully transparent draws no shade.
    pub shade: Color,
    /// How much everything beneath a dialog is softened, in halvings of resolution: `0` keeps
    /// it sharp, `1` halves it, `2` quarters it.  Backends that cannot blur ignore it.
    pub blur: u8,
    /// The color of the one-pixel border drawn just outside the dialog.
    pub border: Color,
    /// `true` to add GEM's outline outside the border: a two-pixel `gap` and a one-pixel line
    /// in the `border` color.
    pub outlined: bool,
    /// The color of the gap between the border and the outline.
    pub gap: Color,
}

impl Default for ModalStyle {
    /// The GEM look: a 25% black shade, softened to a quarter of its resolution, a black border,
    /// and a white-gapped outline.
    ///
    /// # Returns
    ///
    /// The default style.
    fn default() -> Self {
        Self {
            shade: Color::RGBA(0, 0, 0, 64),
            blur: 2,
            border: Color::BLACK,
            outlined: true,
            gap: Color::WHITE,
        }
    }
}

/// The width of GEM's outline gap, in logical pixels.
const OUTLINE_GAP: i32 = 2;

/// One layer of a frame: the `Widget`s drawn together, and the dialog the layer belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DrawLayer {
    /// The root of the modal dialog this layer draws, or `None` for the window beneath every
    /// dialog.
    pub(super) modal: Option<WidgetId>,
    /// The `Widget`s of the layer in drawing order.
    pub(super) ids: Vec<WidgetId>,
}

/// Returns the four one-pixel-thick edges of a ring `distance` pixels outside `rect`.
///
/// # Arguments
///
/// * `rect` - The rectangle the ring surrounds.
/// * `distance` - How far outside `rect` the ring lies; `1` touches it.
/// * `thickness` - How many pixels thick the ring is, growing outward.
///
/// # Returns
///
/// The top, bottom, left, and right edges.
fn ring(rect: Rect, distance: i32, thickness: i32) -> [Rect; 4] {
    let outer_x = rect.x - distance - thickness + 1;
    let outer_y = rect.y - distance - thickness + 1;
    let outer_w = (rect.w as i32 + 2 * (distance + thickness - 1)).max(0) as u32;
    let outer_h = (rect.h as i32 + 2 * (distance + thickness - 1)).max(0) as u32;
    let t = thickness.max(0) as u32;

    [
        Rect::new(outer_x, outer_y, outer_w, t),
        Rect::new(outer_x, outer_y + outer_h as i32 - t as i32, outer_w, t),
        Rect::new(
            outer_x,
            outer_y + t as i32,
            t,
            outer_h.saturating_sub(2 * t),
        ),
        Rect::new(
            outer_x + outer_w as i32 - t as i32,
            outer_y + t as i32,
            t,
            outer_h.saturating_sub(2 * t),
        ),
    ]
}

/// One open modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ModalFrame {
    /// The root `Widget` of the dialog's subtree.
    root: WidgetId,
    /// The `Widget` that had focus when the dialog opened, given focus back when it closes.
    restore_focus: Option<WidgetId>,
}

impl WidgetCache {
    /// Opens a modal dialog: from now on, input goes only to `root` and its descendants.
    ///
    /// The root is shown (its [`Behavior::HIDDEN`] flag is cleared) and brought to the top of
    /// its parent's stacking order.  Until it closes it is drawn as a layer of its own, above
    /// everything else, after a backdrop that darkens and softens what lies beneath and with a
    /// border around it (see [`ModalStyle`]).  Focus moves to
    /// the dialog's first focusable `Widget` in Tab order, or is cleared if it has none.  Return
    /// and Escape activate the dialog's default and cancel `Widget`s.  Activating a
    /// [`Behavior::DISMISSES_DIALOG`] `Widget` inside it closes it (see
    /// [`WidgetCache::top_modal`]).
    ///
    /// Dialogs nest: opening another dialog while one is open makes the new one the only one
    /// that receives input, until it closes.  Emits [`EngineEvent::ModalOpened`].
    ///
    /// Applications normally open dialogs with `Ui::open_modal`, which also registers the
    /// callback that runs when the dialog is dismissed.
    ///
    /// # Arguments
    ///
    /// * `root` - The root `Widget` of the dialog, for example a hidden `Panel` holding the
    ///   dialog's controls.
    ///
    /// # Returns
    ///
    /// `Ok(())` once the dialog is open.
    ///
    /// # Errors
    ///
    /// [`Error::UnknownWidget`] if `root` does not exist or was removed.  [`Error::Tree`] if
    /// `root` is the window's root `Widget` (the whole window cannot be a dialog), or is already
    /// open as a modal dialog.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pushrod::cache::WidgetCache;
    /// use pushrod::geometry::size;
    /// use pushrod::root_widget::RootWidget;
    ///
    /// let mut cache = WidgetCache::new();
    /// cache.add(Box::new(RootWidget::new(size(320, 200))));
    /// let main = cache.add(Box::new(RootWidget::new(size(100, 100))));
    /// let dialog = cache.add(Box::new(RootWidget::new(size(100, 100))));
    ///
    /// cache.open_modal(dialog).unwrap();
    ///
    /// assert_eq!(cache.top_modal(), Some(dialog));
    /// assert!(cache.is_inert(main));
    /// assert!(!cache.is_inert(dialog));
    /// ```
    pub fn open_modal(&mut self, root: WidgetId) -> Result<()> {
        if self.get(root).is_none() {
            return Err(Error::UnknownWidget { id: root });
        }

        if root == ROOT_WIDGET_ID {
            return Err(Error::tree(root, "the root cannot be a modal dialog"));
        }

        if self.modals.iter().any(|frame| frame.root == root) {
            return Err(Error::tree(root, "already open as a modal dialog"));
        }

        if self.get(root).is_some_and(|w| w.is_hidden()) {
            self.show_subtree(root);
        }

        // The root is not the window root, so it has siblings to rise above.
        let _ = self.set_stacking_index(root, usize::MAX);

        self.modals.push(ModalFrame {
            root,
            restore_focus: self.focused_widget_id,
        });
        self.needs_full_redraw = true;
        self.queue_event(Event::Engine(EngineEvent::ModalOpened { root }));
        self.focus_first_in_modal();

        Ok(())
    }

    /// Shows a hidden dialog root and repaints it and every `Widget` under it.  Hidden
    /// `Widget`s are not painted, so a subtree that was hidden since it was built has nothing
    /// in its textures yet.
    ///
    /// # Arguments
    ///
    /// * `root` - The dialog's root `Widget`.
    fn show_subtree(&mut self, root: WidgetId) {
        for id in self.subtree(root) {
            if let Some(widget) = self.get_mut(id) {
                if id == root {
                    widget.set_hidden(false);
                }

                widget.set_invalidated(true);
            }
        }
    }

    /// Returns how open modal dialogs are drawn.
    ///
    /// # Returns
    ///
    /// The style set with [`WidgetCache::set_modal_style`]; [`ModalStyle::default`] until then.
    pub fn modal_style(&self) -> ModalStyle {
        self.modal_style
    }

    /// Changes how open modal dialogs are drawn, and redraws the window.
    ///
    /// # Arguments
    ///
    /// * `style` - The backdrop and border to use from the next frame.
    pub fn set_modal_style(&mut self, style: ModalStyle) {
        self.modal_style = style;
        self.needs_full_redraw = true;
    }

    /// Splits the visible `Widget`s into drawing layers: the window, then one layer per open
    /// modal dialog in the order they opened.  A dialog's subtree is taken out of the layer it
    /// sits in, so every dialog is drawn above everything beneath it, wherever it is in the tree.
    /// A dialog that is hidden itself or through a parent has no layer.
    ///
    /// # Returns
    ///
    /// The layers, bottom first; the first is always the window's.
    pub(super) fn draw_layers(&self) -> Vec<DrawLayer> {
        let roots = self.open_modals();
        let mut layers = vec![DrawLayer {
            modal: None,
            ids: self.visible_subtree(ROOT_WIDGET_ID, &roots),
        }];

        for &root in &roots {
            let shown = std::iter::once(root)
                .chain(self.ancestors(root))
                .all(|id| self.get(id).is_some_and(|w| !w.is_hidden()));

            if shown {
                layers.push(DrawLayer {
                    modal: Some(root),
                    ids: self.visible_subtree(root, &roots),
                });
            }
        }

        layers
    }

    /// Lists the visible `Widget`s under `top` in drawing order (parents before children,
    /// siblings in stacking order), leaving out hidden subtrees and the subtrees of other open
    /// dialogs.
    ///
    /// # Arguments
    ///
    /// * `top` - Where to start; it is included.
    /// * `dialogs` - The roots of the open dialogs, each drawn in a layer of its own.
    ///
    /// # Returns
    ///
    /// The IDs in drawing order.
    fn visible_subtree(&self, top: WidgetId, dialogs: &[WidgetId]) -> Vec<WidgetId> {
        let mut order = Vec::new();
        let mut stack = vec![top];

        while let Some(id) = stack.pop() {
            let Some(widget) = self.get(id) else {
                continue;
            };

            if widget.is_hidden() || (id != top && dialogs.contains(&id)) {
                continue;
            }

            order.push(id);
            stack.extend(self.children_of(id).iter().rev());
        }

        order
    }

    /// Draws what goes beneath a dialog's layer: the backdrop over everything already in the
    /// frame, then the dialog's border and outline.
    ///
    /// # Arguments
    ///
    /// * `root` - The dialog's root `Widget`.
    /// * `backend` - The backend composing the frame.
    pub(super) fn draw_modal_backdrop(&self, root: WidgetId, backend: &mut dyn Backend) {
        let style = self.modal_style;

        backend.draw_backdrop(style.shade, style.blur);

        let Some(bounds) = self.visible_rect(root) else {
            return;
        };
        let mut rings = vec![(1, 1, style.border)];

        if style.outlined {
            rings.push((2, OUTLINE_GAP, style.gap));
            rings.push((2 + OUTLINE_GAP, 1, style.border));
        }

        for (distance, thickness, color) in rings {
            for edge in ring(bounds, distance, thickness) {
                if edge.w > 0 && edge.h > 0 {
                    backend.fill_frame_rect(edge, color);
                }
            }
        }
    }

    /// Returns the modal dialog that receives input.
    ///
    /// # Returns
    ///
    /// The root `Widget` of the most recently opened dialog that is still open, or `None` when
    /// no dialog is open.
    pub fn top_modal(&self) -> Option<WidgetId> {
        self.modals.last().map(|frame| frame.root)
    }

    /// Returns the open modal dialogs, the first opened first.
    ///
    /// # Returns
    ///
    /// The root `Widget` of each open dialog; the last one receives input.
    pub fn open_modals(&self) -> Vec<WidgetId> {
        self.modals.iter().map(|frame| frame.root).collect()
    }

    /// Returns whether a `Widget` is blocked from input by a modal dialog.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the `Widget`.
    ///
    /// # Returns
    ///
    /// `true` if a dialog is open and `id` is neither its root nor one of its descendants;
    /// `false` when no dialog is open.
    pub fn is_inert(&self, id: WidgetId) -> bool {
        let Some(top) = self.top_modal() else {
            return false;
        };

        id != top && !self.ancestors(id).contains(&top)
    }

    /// Returns the `Widget` whose subtree receives input.
    ///
    /// # Returns
    ///
    /// The top modal dialog's root, or the window's root when no dialog is open.
    pub(super) fn input_root(&self) -> WidgetId {
        self.top_modal().unwrap_or(ROOT_WIDGET_ID)
    }

    /// Gives focus to the first focusable `Widget` of the top dialog in Tab order, or clears
    /// the focus if it has none.
    fn focus_first_in_modal(&mut self) {
        // Inert `Widget`s cannot take focus, so every Tab stop is inside the dialog.
        match self.tab_stops().first() {
            Some(&id) => {
                self.set_focus(id);
            }
            None => self.clear_focus(),
        }
    }

    /// Closes the top modal dialog if one of `events` activated a
    /// [`Behavior::DISMISSES_DIALOG`] `Widget` inside it: its `Clicked`, its `Activated`, or the
    /// [`EngineEvent::DialogDismissRequested`] it emitted itself (as a `Button` does).  Called
    /// for every batch of events the `Widget`s generate.
    ///
    /// # Arguments
    ///
    /// * `events` - The events the `Widget`s generated, in order.
    pub(super) fn dismiss_on(&mut self, events: &[Event]) {
        for event in events {
            let id = match event {
                Event::Clicked { id, .. }
                | Event::Activated { id }
                | Event::Engine(EngineEvent::DialogDismissRequested { id }) => *id,
                _ => continue,
            };

            // A button activated by Return reports `Activated`, `Clicked` and its own dismiss
            // request; once the dialog has closed the others are no longer inside the top one.
            if self.top_modal().is_some()
                && !self.is_inert(id)
                && self
                    .get(id)
                    .is_some_and(|w| w.behavior().contains(Behavior::DISMISSES_DIALOG))
            {
                let reported = events.iter().any(|e| {
                    matches!(e, Event::Engine(EngineEvent::DialogDismissRequested { id: d }) if *d == id)
                });

                self.close_top_modal(id, !reported);
            }
        }
    }

    /// Closes the top modal dialog: hides its root, gives focus back to the `Widget` that had
    /// it when the dialog opened (or to the next dialog down), and queues
    /// [`EngineEvent::ModalClosed`], after [`EngineEvent::DialogDismissRequested`] when the
    /// dismissing `Widget` did not report one itself.
    ///
    /// # Arguments
    ///
    /// * `dismissed_by` - The `Widget` that dismissed the dialog.
    /// * `announce` - `true` to queue `DialogDismissRequested` for it.
    fn close_top_modal(&mut self, dismissed_by: WidgetId, announce: bool) {
        let Some(frame) = self.modals.pop() else {
            return;
        };

        if let Some(widget) = self.get_mut(frame.root) {
            widget.set_hidden(true);
        }

        self.needs_full_redraw = true;

        if announce {
            self.queue_event(Event::Engine(EngineEvent::DialogDismissRequested {
                id: dismissed_by,
            }));
        }

        self.queue_event(Event::Engine(EngineEvent::ModalClosed {
            root: frame.root,
            dismissed_by,
        }));

        match frame.restore_focus {
            Some(id) if self.set_focus(id) => {}
            _ if self.top_modal().is_some() => self.focus_first_in_modal(),
            _ => self.clear_focus(),
        }
    }

    /// Forgets an open dialog whose root was removed from the tree.  No event is queued and
    /// no callback runs: the removal already emitted [`EngineEvent::WidgetRemoved`].
    ///
    /// # Arguments
    ///
    /// * `id` - A removed `Widget`.
    pub(super) fn forget_modal(&mut self, id: WidgetId) {
        self.modals.retain(|frame| frame.root != id);
    }
}
