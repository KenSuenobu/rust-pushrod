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

//! The Progress Bar page: empty, partly full, and busy.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::progress_bar::ProgressBar;

/// Progress Bar: determinate and indeterminate bars.
pub struct ProgressBarPage;

impl Page for ProgressBarPage {
    fn name(&self) -> &'static str {
        "Progress Bar"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.progress_bar"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "How far along a task is, or that it is busy."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 160));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Empty, 40%, busy, and busy in outline mode",
        );

        for (row, value) in [0.0, 40.0].into_iter().enumerate() {
            let bar = ProgressBar::new(100.0)
                .at(10, 40 + row as i32 * 24)
                .size(300, 16)
                .value(value);

            ui.widgets().add_child(root, Box::new(bar));
        }

        let busy = ui.widgets().add_child(
            root,
            Box::new(ProgressBar::indeterminate().at(10, 88).size(300, 16)),
        );

        ui.widgets().request_animation(busy);

        let mut outline = ProgressBar::indeterminate().at(10, 112).size(300, 16);

        outline.set_outline_mode(true);
        ui.widgets().add_child(root, Box::new(outline));

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("progress_bar.rs")
    }
}
