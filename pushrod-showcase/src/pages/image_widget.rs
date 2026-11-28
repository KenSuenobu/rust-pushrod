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

//! The Image page: how an image fits its box, and what a missing file looks like.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::image_widget::{FitMode, ImageWidget};

/// The path the page loads.  The repository ships no sample bitmap, so the page shows the
/// crossed box that stands in for a file that cannot be loaded.
const SAMPLE_PATH: &str = "assets/sample-missing.png";

/// Image: the fit modes, and a clickable image.
pub struct ImageWidgetPage;

impl Page for ImageWidgetPage {
    fn name(&self) -> &'static str {
        "Image"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.image"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A bitmap from a file: fit modes, clickable, and the box drawn for a missing file."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 240));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Center, Stretch, Fit, and Tile (file not found)",
        );

        for (index, fit) in [
            FitMode::Center,
            FitMode::Stretch,
            FitMode::Fit,
            FitMode::Tile,
        ]
        .into_iter()
        .enumerate()
        {
            ui.widgets().add_child(
                root,
                Box::new(
                    ImageWidget::new(SAMPLE_PATH)
                        .fit(fit)
                        .at(10 + index as i32 * 120, 34)
                        .size(100, 80),
                ),
            );
        }

        label(ui, root, 10, 130, 480, "Clickable");
        ui.widgets().add_child(
            root,
            Box::new(
                ImageWidget::new(SAMPLE_PATH)
                    .clickable()
                    .at(10, 154)
                    .size(100, 60),
            ),
        );

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("image_widget.rs")
    }
}
