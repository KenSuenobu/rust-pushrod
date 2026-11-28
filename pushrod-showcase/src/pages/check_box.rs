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

//! The Check Box page: unchecked, checked, indeterminate, and disabled.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::check_box::CheckBox;

/// Check Box: every state.
pub struct CheckBoxPage;

impl Page for CheckBoxPage {
    fn name(&self) -> &'static str {
        "Check Box"
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A box that is on, off, or in between."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 120));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Unchecked, checked, indeterminate, disabled",
        );

        let cache = ui.widgets();

        cache.add_child(
            root,
            Box::new(CheckBox::new("Send email").at(10, 40).size(130, 20)),
        );
        cache.add_child(
            root,
            Box::new(
                CheckBox::new("Checked")
                    .at(150, 40)
                    .size(130, 20)
                    .checked(true),
            ),
        );
        cache.add_child(
            root,
            Box::new(
                CheckBox::new("All items")
                    .at(10, 70)
                    .size(130, 20)
                    .indeterminate(),
            ),
        );

        let mut disabled = CheckBox::new("Disabled")
            .at(150, 70)
            .size(130, 20)
            .checked(true);

        disabled.set_enabled(false);
        cache.add_child(root, Box::new(disabled));

        root
    }

    fn source(&self) -> &'static str {
        include_str!("check_box.rs")
    }
}
