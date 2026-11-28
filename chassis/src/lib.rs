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

//! # Chassis
//!
//! Chassis is the visual form designer for Pushrod.  This crate holds its core, which has no
//! window and is fully unit-tested: the [project model](project) and the [code generation
//! rules](codegen).

pub mod codegen;
pub mod project;

pub use project::{Handler, Ownership, Project, ProjectError};
