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

//! The Boxed Label page: text with a fill and a border.

use super::page_root;
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::color::Color;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::boxed_label::BoxedLabel;

/// Boxed Label: enabled and disabled.
pub struct BoxedLabelPage;

impl Page for BoxedLabelPage {
    fn name(&self) -> &'static str {
        "Boxed Label"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.boxed_label"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "Text drawn inside a filled, bordered box."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 120));
        let mut ready = BoxedLabel::new("Ready")
            .at(10, 10)
            .size(300, 28)
            .border_color(Color::BLUE);

        ready.set_color(Color::CYAN);
        ui.widgets().add_child(root, Box::new(ready));

        let mut offline = BoxedLabel::new("Offline").at(10, 50).size(300, 28);

        offline.set_color(Color::WHITE);
        offline.set_enabled(false);
        ui.widgets().add_child(root, Box::new(offline));

        root
    }

    fn source(&self) -> &'static str {
        include_str!("boxed_label.rs")
    }
}
