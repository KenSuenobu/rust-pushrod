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

//! The Pushrod Showcase: an interactive demo of every widget.
//!
//! Run it with `cargo run -p pushrod-showcase`.  The window has a searchable page list on the
//! left and the selected page, with its title and description, on the right.  To add a page,
//! implement [`page::Page`] in a new file under `pages/` and add one line to
//! [`pages::register_all`].

pub mod page;
pub mod pages;
pub mod registry;
pub mod settings;
pub mod shell;
