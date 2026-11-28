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

//! The Radio Button page: a group where one choice is selected, and a disabled group.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::radio_button::RadioGroup;

/// Radio Button: a size picker and a disabled pair.
pub struct RadioButtonPage;

impl Page for RadioButtonPage {
    fn name(&self) -> &'static str {
        "Radio Button"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.radio_button"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "Choose exactly one of a few options."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 130));

        label(ui, root, 10, 10, 480, "Enabled and disabled groups");

        RadioGroup::new(["Small", "Medium", "Large"])
            .at(10, 40)
            .item_size(130, 20)
            .selected(1)
            .add_to(ui.widgets(), root);

        let disabled = RadioGroup::new(["Disabled", "Disabled"])
            .at(150, 40)
            .item_size(130, 20)
            .selected(0)
            .add_to(ui.widgets(), root);

        for id in disabled {
            if let Some(button) = ui.widgets().get_mut(id) {
                button.set_enabled(false);
            }
        }

        root
    }

    fn source(&self) -> &'static str {
        include_str!("radio_button.rs")
    }
}
