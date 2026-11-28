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

//! The Resources page: strings and an image found by typed ID.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::resources;
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::resource::StringId;
use pushrod::widget::WidgetId;

/// Resources by ID: `StringId` and `ImageId` constants, and the same lookup by name.
pub struct ResourcesByIdPage;

impl Page for ResourcesByIdPage {
    fn name(&self) -> &'static str {
        "Strings and Images by ID"
    }

    fn group(&self) -> Group {
        Group::Forms
    }

    fn description(&self) -> &'static str {
        "Strings and images from a resource file, looked up by typed ID or by name."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 250));

        label(ui, root, 10, 10, 480, "Strings, by StringId");

        for (i, (name, id)) in [
            ("FORM_TITLE", resources::FORM_TITLE),
            ("GREETING", resources::GREETING),
            ("GOODBYE", resources::GOODBYE),
        ]
        .into_iter()
        .enumerate()
        {
            let text = ui.resources().string(id).unwrap_or("(missing)").to_string();

            label(
                ui,
                root,
                10,
                34 + i as i32 * 24,
                500,
                &format!("StringId({}) {name}: {text}", id.get()),
            );
        }

        label(ui, root, 10, 118, 480, "The same string, found by name");

        let by_name = ui
            .resources()
            .find::<StringId>("GREETING")
            .and_then(|id| ui.resources().string(id))
            .unwrap_or("(missing)")
            .to_string();

        label(
            ui,
            root,
            10,
            142,
            500,
            &format!("find(\"GREETING\"): {by_name}"),
        );

        label(ui, root, 10, 178, 480, "Images, by ImageId");

        let image = match ui.resources().image(resources::LOGO) {
            Some(handle) => format!(
                "ImageId({}) LOGO: {} bytes, key {}",
                resources::LOGO.get(),
                handle.data().len(),
                handle.key()
            ),
            None => "(missing image)".to_string(),
        };

        label(ui, root, 10, 202, 500, &image);

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("resources_by_id.rs")
    }
}
