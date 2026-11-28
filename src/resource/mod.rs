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

//! # Resources
//!
//! The typed IDs that name resources: one newtype per resource kind, so an ID of one kind cannot
//! be used as another.  See [`id`].

pub mod id;

pub use id::{AlertId, FileId, FormId, FormIndex, IconId, ImageId, MenuId, StringId};
