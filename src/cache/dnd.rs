// Drag and drop in the widget cache
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

//! The drag-and-drop session: registration of sources and targets, starting a drag when the
//! pointer moves past a threshold, highlighting the target under the pointer, and ending the drag.
//! See [`crate::dnd`] for the model.

use super::WidgetCache;
use crate::dnd::{DragPayload, DragSource, DropTarget};
use crate::event::{EngineEvent, Event, EventCtx};
use crate::geometry::{Point, point};
use crate::widget::{ROOT_WIDGET_ID, VisualState, WidgetId};
use std::collections::HashMap;
use std::path::PathBuf;

/// How far, in pixels, the pointer must move from the press before a drag starts.
const DRAG_THRESHOLD: i32 = 4;

/// How far, in pixels, the drag image sits right of and below the pointer, so the pointer is never
/// over the image.
const IMAGE_OFFSET: i32 = 12;

/// A drag in progress.
#[derive(Debug)]
struct Session {
    source: WidgetId,
    payload: DragPayload,
    image: Option<WidgetId>,
    over: Option<WidgetId>,
}

/// The registered sources and targets, and the drag in progress.
#[derive(Debug, Default)]
pub(super) struct DndState {
    sources: HashMap<WidgetId, DragSource>,
    targets: HashMap<WidgetId, DropTarget>,
    pending: Option<(WidgetId, Point)>,
    active: Option<Session>,
}

impl WidgetCache {
    /// Makes a `Widget` a drag source, replacing any earlier registration.  A press on the
    /// `Widget` or one of its descendants followed by movement starts the drag.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`.
    /// * `source` - The source description.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` exists.
    pub fn set_drag_source(&mut self, id: WidgetId, source: DragSource) -> bool {
        if self.get(id).is_none() {
            return false;
        }

        self.dnd.sources.insert(id, source);
        true
    }

    /// Makes a `Widget` a drop target, replacing any earlier registration.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Widget`.
    /// * `target` - The target description.
    ///
    /// # Returns
    ///
    /// `true` if the `Widget` exists.
    pub fn set_drop_target(&mut self, id: WidgetId, target: DropTarget) -> bool {
        if self.get(id).is_none() {
            return false;
        }

        self.dnd.targets.insert(id, target);
        true
    }

    /// Returns whether a drag is in progress.
    ///
    /// # Returns
    ///
    /// `true` between `DragStarted` and `DragEnded`.
    pub fn is_dragging(&self) -> bool {
        self.dnd.active.is_some()
    }

    /// Remembers a press that may become a drag.
    ///
    /// # Arguments
    ///
    /// * `target` - The `Widget` pressed.
    /// * `position` - Where the press was.
    pub(super) fn dnd_pointer_down(&mut self, target: WidgetId, position: Point) {
        let mut current = target;

        loop {
            if self.dnd.sources.contains_key(&current) && self.get(current).is_some() {
                self.dnd.pending = Some((current, position));
                return;
            }

            let parent = self.get(current).map(|w| w.get_parent());

            match parent {
                Some(p) if p != current => current = p,
                _ => break,
            }
        }

        self.dnd.pending = None;
    }

    /// Handles pointer motion for the drag: starts it past the threshold, and moves it.
    ///
    /// # Arguments
    ///
    /// * `position` - The pointer position.
    ///
    /// # Returns
    ///
    /// `true` if a drag is in progress, so the motion is not delivered to the `Widget`s.
    pub(super) fn dnd_pointer_move(&mut self, position: Point) -> bool {
        if self.dnd.active.is_none() {
            let Some((source, start)) = self.dnd.pending else {
                return false;
            };
            let (dx, dy) = (position.x - start.x, position.y - start.y);

            if dx * dx + dy * dy < DRAG_THRESHOLD * DRAG_THRESHOLD {
                return false;
            }

            self.dnd_start(source);
        }

        self.dnd_update(position);
        true
    }

    /// Starts the drag from `source`.
    fn dnd_start(&mut self, source: WidgetId) {
        self.dnd.pending = None;

        let Some(description) = self.dnd.sources.get(&source).cloned() else {
            return;
        };

        // The press must not turn into a click.
        if let Some(captured) = self.captured_widget_id.take() {
            self.set_state_flag(captured, VisualState::PRESSED, false);
        }

        if let Some(image) = description.image {
            self.show_drag_image(image);
        }

        self.dnd.active = Some(Session {
            source,
            payload: description.make_payload(),
            image: description.image,
            over: None,
        });
        self.queue_event(Event::Engine(EngineEvent::DragStarted { source }));
    }

    /// Shows the drag image and brings it in front of its siblings.
    fn show_drag_image(&mut self, image: WidgetId) {
        let Some(parent) = self.get(image).map(|w| w.get_parent()) else {
            return;
        };
        let top = self.children_of(parent).len().saturating_sub(1);

        if let Some(widget) = self.get_mut(image) {
            widget.set_hidden(false);
        }

        let _ = self.set_stacking_index(image, top);
    }

    /// Moves the drag image and the `DRAG_OVER` highlight to the pointer.
    fn dnd_update(&mut self, position: Point) {
        let Some(session) = self.dnd.active.as_ref() else {
            return;
        };
        let (image, old) = (session.image, session.over);
        let found = self.find_drop_target(position, image, &session.payload);

        if let Some(image) = image
            && let Some(widget) = self.get_mut(image)
        {
            widget.set_origin(point(position.x + IMAGE_OFFSET, position.y + IMAGE_OFFSET));
        }

        if found != old {
            if let Some(previous) = old {
                self.set_state_flag(previous, VisualState::DRAG_OVER, false);
            }

            if let Some(next) = found {
                self.set_state_flag(next, VisualState::DRAG_OVER, true);
            }

            if let Some(session) = self.dnd.active.as_mut() {
                session.over = found;
            }
        }
    }

    /// Finds the accepting drop target at a point: the nearest `Widget`, from the one under the
    /// point up to the root, that is registered, accepts the payload, and is not inert.
    ///
    /// # Arguments
    ///
    /// * `position` - The point.
    /// * `skip` - A `Widget` (the drag image) that is never under the point.
    /// * `payload` - The payload.
    ///
    /// # Returns
    ///
    /// The target's ID, or `None`.
    fn find_drop_target(
        &self,
        position: Point,
        skip: Option<WidgetId>,
        payload: &DragPayload,
    ) -> Option<WidgetId> {
        let mut current = self.widget_at_skipping(position, skip);

        loop {
            let accepts = self
                .dnd
                .targets
                .get(&current)
                .is_some_and(|t| t.accepts_payload(payload));

            if accepts && !self.is_inert(current) && self.get(current)?.is_enabled() {
                return Some(current);
            }

            let parent = self.get(current)?.get_parent();

            if parent == current {
                return None;
            }

            current = parent;
        }
    }

    /// Returns the top-most visible `Widget` at a point, ignoring one `Widget` and its descendants.
    fn widget_at_skipping(&self, position: Point, skip: Option<WidgetId>) -> WidgetId {
        for id in self.draw_order().into_iter().rev() {
            let hidden_by_skip = skip.is_some_and(|s| id == s || self.ancestors(id).contains(&s));

            if !hidden_by_skip
                && let Some(visible) = self.visible_rect(id)
                && visible.contains(position.x, position.y)
            {
                return id;
            }
        }

        ROOT_WIDGET_ID
    }

    /// Handles the pointer button going up.
    ///
    /// # Arguments
    ///
    /// * `position` - The pointer position.
    /// * `ctx` - Collects the `Dropped` event.
    ///
    /// # Returns
    ///
    /// `true` if it ended a drag, so the release is not delivered to the `Widget`s.
    pub(super) fn dnd_pointer_up(&mut self, position: Point, ctx: &mut EventCtx) -> bool {
        self.dnd.pending = None;

        if self.dnd.active.is_none() {
            return false;
        }

        self.dnd_update(position);
        self.dnd_end(true, ctx);
        true
    }

    /// Cancels the drag in progress.
    ///
    /// # Returns
    ///
    /// `true` if there was one.
    pub(super) fn dnd_cancel(&mut self) -> bool {
        if self.dnd.active.is_none() {
            return false;
        }

        self.dnd_end(false, &mut EventCtx::default());
        true
    }

    /// Ends the drag: removes the highlight and image, sends `Dropped` when `drop` and a target
    /// is under the pointer, and queues `DragEnded`.
    fn dnd_end(&mut self, drop: bool, ctx: &mut EventCtx) {
        let Some(session) = self.dnd.active.take() else {
            return;
        };

        if let Some(over) = session.over {
            self.set_state_flag(over, VisualState::DRAG_OVER, false);
        }

        if let Some(widget) = session.image.and_then(|image| self.get_mut(image)) {
            widget.set_hidden(true);
        }

        let target = session.over.filter(|_| drop);

        if let Some(id) = target {
            ctx.emit(Event::Dropped {
                id,
                payload: session.payload,
            });
        }

        self.queue_event(Event::Engine(EngineEvent::DragEnded {
            source: session.source,
            target,
            dropped: target.is_some(),
        }));
    }

    /// Delivers files dropped from the operating system to the accepting target at a point.
    ///
    /// # Arguments
    ///
    /// * `position` - Where the pointer was.
    /// * `paths` - The files.
    /// * `ctx` - Collects the `Dropped` event.
    pub(super) fn dnd_files(&mut self, position: Point, paths: Vec<PathBuf>, ctx: &mut EventCtx) {
        let count = paths.len();
        let payload = DragPayload::new(paths);
        let target = self.find_drop_target(position, None, &payload);

        if let Some(id) = target {
            ctx.emit(Event::Dropped { id, payload });
        }

        self.queue_event(Event::Engine(EngineEvent::FilesDropped { target, count }));
    }
}
