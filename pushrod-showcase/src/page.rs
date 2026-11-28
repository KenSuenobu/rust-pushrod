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

//! The [`Page`] trait: one showcase page per widget or feature, and the groups pages sit in.

use crate::shell::ShowcaseUi;
use pushrod::event::Event;
use pushrod::widget::WidgetId;

/// The heading a page is listed under in the page list.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Group {
    /// Text, focus, dialogs: what is not a widget.
    Core,
    /// The widgets in `pushrod-widgets`.
    Standard,
    /// Row, column, and form layouts.
    Layouts,
    /// Forms and resources.
    Forms,
    /// The widgets in `pushrod-extra-widgets`.
    Extra,
}

impl Group {
    /// Every group, in the order the page list shows them.
    pub const ALL: [Group; 5] = [
        Group::Core,
        Group::Standard,
        Group::Layouts,
        Group::Forms,
        Group::Extra,
    ];

    /// Returns the heading shown in the page list.
    ///
    /// # Returns
    ///
    /// The group's title.
    pub fn title(self) -> &'static str {
        match self {
            Group::Core => "Core",
            Group::Standard => "Standard widgets",
            Group::Layouts => "Layouts",
            Group::Forms => "Forms and resources",
            Group::Extra => "Extra widgets",
        }
    }
}

/// One page of the showcase.  Adding a page is one small file: implement this trait, then add
/// one registration line in `pages::register_all`.
pub trait Page {
    /// Returns the page's name: the button label in the page list, and the key the showcase
    /// uses to remember the last page.  Names must be unique.
    ///
    /// # Returns
    ///
    /// The name.
    fn name(&self) -> &'static str;

    /// Returns the registered widget type names (for example `"pushrod.button"`) this page
    /// demonstrates.  The completeness test fails when a widget library registers a type that
    /// no page lists here.
    ///
    /// # Returns
    ///
    /// The type names; empty (the default) for a page that is not about a registered widget.
    fn widget_types(&self) -> &'static [&'static str] {
        &[]
    }

    /// Returns the group the page is listed under.
    ///
    /// # Returns
    ///
    /// The group.
    fn group(&self) -> Group;

    /// Returns the one-line description shown under the page's title.  Search matches it too.
    ///
    /// # Returns
    ///
    /// The description.
    fn description(&self) -> &'static str;

    /// Builds the page's widgets.
    ///
    /// The returned widget is the page's root; the shell moves it into the scrolling area and
    /// removes it, with its children, when another page is chosen.  Build it with a fixed size
    /// so the scrolling area knows how much to scroll.
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback handle: add widgets with `ui.widgets()`.
    ///
    /// # Returns
    ///
    /// The ID of the page's root widget.
    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId;

    /// Reacts to an event while the page is showing.  A page that has controls of its own (a
    /// width slider, a button that opens a dialog) answers them here; the default ignores
    /// every event.
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback handle.
    /// * `root` - The page's root widget, as `build` returned it.
    /// * `event` - The event a widget generated.
    fn handle_event(&self, _ui: &mut ShowcaseUi<'_>, _root: WidgetId, _event: &Event) {}

    /// Runs just before the page's widgets are removed, so a page can give back anything it
    /// borrowed (such as a form loaded once at startup).  The default does nothing.
    ///
    /// # Arguments
    ///
    /// * `ui` - The callback handle.
    /// * `root` - The page's root widget.
    fn leave(&self, _ui: &mut ShowcaseUi<'_>, _root: WidgetId) {}

    /// Returns the source code of the page, for the source view.
    ///
    /// # Returns
    ///
    /// The code, or an empty string when the page does not provide it.
    fn source(&self) -> &'static str {
        ""
    }
}
