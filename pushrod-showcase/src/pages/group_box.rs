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

//! The Group Box page: a titled frame that holds other widgets.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::button::Button;
use pushrod_widgets::group_box::GroupBox;

/// Group Box: a titled frame with children.
pub struct GroupBoxPage;

impl Page for GroupBoxPage {
    fn name(&self) -> &'static str {
        "Group Box"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.group_box"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A titled frame that groups related widgets."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 170));
        let group = ui.widgets().add_child(
            root,
            Box::new(GroupBox::new("Size").at(10, 10).size(300, 130)),
        );

        label(ui, group, 12, 24, 260, "Small, medium, large");
        ui.widgets().add_child(
            group,
            Box::new(Button::new("Apply").at(12, 56).size(120, 24)),
        );

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("group_box.rs")
    }
}
