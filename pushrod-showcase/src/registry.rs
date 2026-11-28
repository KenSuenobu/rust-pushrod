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

//! The list of pages, and the search that filters it.

use crate::page::{Group, Page};

/// Every registered page, in registration order.
#[derive(Default)]
pub struct Registry {
    pages: Vec<Box<dyn Page>>,
}

impl Registry {
    /// Creates an empty registry.
    ///
    /// # Returns
    ///
    /// The registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a page.
    ///
    /// # Arguments
    ///
    /// * `page` - The page to add.
    ///
    /// # Panics
    ///
    /// Panics when a page with the same name is already registered: names are the key the last
    /// page is remembered by.
    pub fn register<P: Page + 'static>(&mut self, page: P) {
        assert!(
            self.find(page.name()).is_none(),
            "duplicate showcase page name: {}",
            page.name()
        );
        self.pages.push(Box::new(page));
    }

    /// Returns how many pages are registered.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Returns `true` when no page is registered.
    ///
    /// # Returns
    ///
    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Returns a page by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The page's index, in registration order.
    ///
    /// # Returns
    ///
    /// The page, or `None` when the index is out of range.
    pub fn get(&self, index: usize) -> Option<&dyn Page> {
        self.pages.get(index).map(|page| page.as_ref())
    }

    /// Finds a page by its exact name.
    ///
    /// # Arguments
    ///
    /// * `name` - The page's name.
    ///
    /// # Returns
    ///
    /// The page's index, or `None`.
    pub fn find(&self, name: &str) -> Option<usize> {
        self.pages.iter().position(|page| page.name() == name)
    }

    /// Returns the pages in a group that match a search, in registration order.
    ///
    /// A page matches when the query, ignoring case and surrounding spaces, appears in its name
    /// or its description.  An empty query matches every page.
    ///
    /// # Arguments
    ///
    /// * `group` - The group to list.
    /// * `query` - The text typed in the search field.
    ///
    /// # Returns
    ///
    /// The matching pages' indexes.
    pub fn matching(&self, group: Group, query: &str) -> Vec<usize> {
        let query = query.trim().to_lowercase();

        self.pages
            .iter()
            .enumerate()
            .filter(|(_, page)| page.group() == group)
            .filter(|(_, page)| {
                query.is_empty()
                    || page.name().to_lowercase().contains(&query)
                    || page.description().to_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect()
    }
}
