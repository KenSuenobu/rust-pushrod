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

//! The Panel page: a plain container that groups and clips its children.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;

/// Panel: a panel holding other widgets.
pub struct PanelPage;

impl Page for PanelPage {
    fn name(&self) -> &'static str {
        "Panel"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.panel"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A plain surface that holds other widgets, here two panels side by side."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 150));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Two panels, each with its own children",
        );

        for (index, name) in ["First", "Second"].into_iter().enumerate() {
            let panel = ui.widgets().add_child(
                root,
                Box::new(Panel::new().at(10 + index as i32 * 160, 34).size(150, 100)),
            );

            ui.widgets()
                .add_child(panel, Box::new(Button::new(name).at(15, 38).size(120, 24)));
        }

        root
    }

    fn source(&self) -> &'static str {
        include_str!("panel.rs")
    }
}
