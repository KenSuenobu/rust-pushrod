// Drag and drop
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

//! # Drag and drop
//!
//! Drag data from one `Widget` to another inside the window, or drop files from the operating
//! system onto the window.
//!
//! * A **drag source** ([`DragSource`]) makes a [`DragPayload`] when the user presses on the
//!   `Widget` (or a descendant) and moves the pointer past a small threshold.  It may name an
//!   *image* `Widget`: a hidden `Widget` that is shown just below and to the right of the pointer
//!   while the drag lasts, so it follows the pointer without any per-frame work beyond moving a
//!   cached texture.
//! * A **drop target** ([`DropTarget`]) lists the payload types it accepts.  While a payload it
//!   accepts is over it, it has [`VisualState::DRAG_OVER`](crate::widget::VisualState::DRAG_OVER)
//!   so it can highlight itself.
//! * Releasing the button over an accepting target sends the application
//!   [`Event::Dropped { id, payload }`](crate::event::Event::Dropped); register for it with
//!   [`Engine::on_drop`](crate::engine::Engine::on_drop).  Escape, or releasing anywhere else,
//!   cancels the drag.
//! * Files dropped on the window from the OS arrive the same way, with a `Vec<PathBuf>` payload.
//!   A target accepts them by listing `Vec<PathBuf>`.
//!
//! While a drag is in progress the other `Widget`s receive no pointer events, so a `Button` under
//! the pointer is not clicked.  Every step emits an [`EngineEvent`](crate::event::EngineEvent):
//! `DragStarted`, `DragEnded { dropped }` and `FilesDropped`.
//!
//! # Examples
//!
//! ```rust
//! use pushrod::dnd::{DragPayload, DragSource, DropTarget};
//!
//! #[derive(Debug, PartialEq)]
//! struct Row(u32);
//!
//! let source = DragSource::new(|| DragPayload::new(Row(7)));
//! let target = DropTarget::new().accepts::<Row>();
//!
//! let payload = source.make_payload();
//!
//! assert!(target.accepts_payload(&payload));
//! assert_eq!(payload.get::<Row>(), Some(&Row(7)));
//! assert!(payload.get::<String>().is_none());
//! ```

use crate::widget::WidgetId;
use std::any::{Any, TypeId};
use std::fmt;
use std::rc::Rc;

/// The data a drag carries: any value, shared and read-only.
///
/// Cloning is cheap; every clone refers to the same value.
#[derive(Clone)]
pub struct DragPayload {
    value: Rc<dyn Any>,
    type_name: &'static str,
}

impl DragPayload {
    /// Wraps a value as a payload.
    ///
    /// # Arguments
    ///
    /// * `value` - The data to carry.
    ///
    /// # Returns
    ///
    /// The payload.
    pub fn new<T: Any>(value: T) -> Self {
        Self {
            value: Rc::new(value),
            type_name: std::any::type_name::<T>(),
        }
    }

    /// Reads the payload as a `T`.
    ///
    /// # Returns
    ///
    /// The value, or `None` if the payload holds another type.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    /// Returns the [`TypeId`] of the carried value.
    ///
    /// # Returns
    ///
    /// The type ID.
    pub fn type_id(&self) -> TypeId {
        (*self.value).type_id()
    }

    /// Returns the name of the carried value's type, for diagnostics.
    ///
    /// # Returns
    ///
    /// The type name.
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
}

impl fmt::Debug for DragPayload {
    /// Formats the payload as its type name.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DragPayload<{}>", self.type_name)
    }
}

/// Makes a payload each time a drag starts.
pub type PayloadFn = Rc<dyn Fn() -> DragPayload>;

/// Describes a `Widget` that can be dragged.  Register it with
/// [`Engine::set_drag_source`](crate::engine::Engine::set_drag_source).
#[derive(Clone)]
pub struct DragSource {
    pub(crate) make: PayloadFn,
    pub(crate) image: Option<WidgetId>,
}

impl DragSource {
    /// Creates a drag source.
    ///
    /// # Arguments
    ///
    /// * `make` - Called when a drag starts; returns the payload.  It may read the application's
    ///   state through whatever it captured, so it should be quick.
    ///
    /// # Returns
    ///
    /// A source with no drag image.
    pub fn new<F>(make: F) -> Self
    where
        F: Fn() -> DragPayload + 'static,
    {
        Self {
            make: Rc::new(make),
            image: None,
        }
    }

    /// Names the `Widget` shown under the pointer during the drag.
    ///
    /// # Arguments
    ///
    /// * `image` - A `Widget` that is a child of the root `Widget`, normally added hidden.  It is
    ///   brought to the top and shown when the drag starts and hidden again when it ends.
    ///
    /// # Returns
    ///
    /// The source with the image set.
    pub fn image(mut self, image: WidgetId) -> Self {
        self.image = Some(image);
        self
    }

    /// Makes a payload, as when a drag starts.
    ///
    /// # Returns
    ///
    /// The payload.
    pub fn make_payload(&self) -> DragPayload {
        (self.make)()
    }
}

impl fmt::Debug for DragSource {
    /// Formats the source without its closure.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DragSource")
            .field("image", &self.image)
            .finish_non_exhaustive()
    }
}

/// Describes a `Widget` that accepts drops.  Register it with
/// [`Engine::set_drop_target`](crate::engine::Engine::set_drop_target).
#[derive(Debug, Clone, Default)]
pub struct DropTarget {
    pub(crate) accepted: Vec<TypeId>,
}

impl DropTarget {
    /// Creates a target that accepts nothing yet.
    ///
    /// # Returns
    ///
    /// The target.
    pub fn new() -> Self {
        Self::default()
    }

    /// Accepts payloads holding a `T`.
    ///
    /// # Returns
    ///
    /// The target with `T` added.
    pub fn accepts<T: Any>(mut self) -> Self {
        let id = TypeId::of::<T>();

        if !self.accepted.contains(&id) {
            self.accepted.push(id);
        }

        self
    }

    /// Returns whether the target accepts a payload.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload.
    ///
    /// # Returns
    ///
    /// `true` if the payload's type was listed with [`accepts`](DropTarget::accepts).
    pub fn accepts_payload(&self, payload: &DragPayload) -> bool {
        self.accepted.contains(&payload.type_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn payload_reads_back_only_its_own_type() {
        let p = DragPayload::new(5u32);

        assert_eq!(p.get::<u32>(), Some(&5));
        assert!(p.get::<i64>().is_none());
        assert_eq!(p.type_id(), TypeId::of::<u32>());
        assert!(p.type_name().contains("u32"));
        assert!(format!("{p:?}").contains("u32"));
    }

    #[test]
    fn clones_share_the_value() {
        let p = DragPayload::new(String::from("row"));
        let q = p.clone();

        assert_eq!(q.get::<String>().unwrap(), "row");
    }

    #[test]
    fn target_accepts_only_listed_types() {
        let target = DropTarget::new().accepts::<u32>().accepts::<u32>();

        assert_eq!(target.accepted.len(), 1);
        assert!(target.accepts_payload(&DragPayload::new(1u32)));
        assert!(!target.accepts_payload(&DragPayload::new("x")));
        assert!(!DropTarget::new().accepts_payload(&DragPayload::new(1u32)));
        assert!(
            DropTarget::new()
                .accepts::<Vec<PathBuf>>()
                .accepts_payload(&DragPayload::new(Vec::<PathBuf>::new()))
        );
    }

    #[test]
    fn source_makes_a_fresh_payload_each_time() {
        let source = DragSource::new(|| DragPayload::new(1u8)).image(4);

        assert_eq!(source.image, Some(4));
        assert_eq!(source.make_payload().get::<u8>(), Some(&1));
    }
}
