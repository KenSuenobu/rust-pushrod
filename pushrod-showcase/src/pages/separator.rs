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

//! The Separator page: plain and etched lines in both directions.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::color::Color;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::separator::Separator;

/// Separator: horizontal and vertical lines.
pub struct SeparatorPage;

impl Page for SeparatorPage {
    fn name(&self) -> &'static str {
        "Separator"
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A thin line that divides a window into areas."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 160));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Horizontal and vertical, plain and etched",
        );

        let cache = ui.widgets();

        cache.add_child(root, Box::new(Separator::horizontal(300).at(10, 44)));
        cache.add_child(
            root,
            Box::new(Separator::horizontal(300).etched().at(10, 64)),
        );
        cache.add_child(
            root,
            Box::new(Separator::vertical(60).line_color(Color::BLUE).at(70, 84)),
        );
        cache.add_child(root, Box::new(Separator::vertical(60).etched().at(140, 84)));

        root
    }

    fn source(&self) -> &'static str {
        include_str!("separator.rs")
    }
}
