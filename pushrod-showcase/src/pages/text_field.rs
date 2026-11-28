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

//! The Text Field page: a labeled, editable field.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::text_field::TextField;

/// Text Field: fictional contact fields.
pub struct TextFieldPage;

impl Page for TextFieldPage {
    fn name(&self) -> &'static str {
        "Text Field"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.text_field"]
    }

    fn group(&self) -> Group {
        Group::Forms
    }

    fn description(&self) -> &'static str {
        "A single-line field for typing text."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 120));

        label(ui, root, 10, 10, 480, "Click a field and type");

        let cache = ui.widgets();

        cache.add_child(
            root,
            Box::new(
                TextField::new()
                    .at(10, 40)
                    .size(300, 24)
                    .with_label("Name")
                    .with_text("Jane Doe"),
            ),
        );
        cache.add_child(
            root,
            Box::new(
                TextField::new()
                    .at(10, 74)
                    .size(300, 24)
                    .with_label("City")
                    .with_text("Anytown"),
            ),
        );

        root
    }

    fn source(&self) -> &'static str {
        include_str!("text_field.rs")
    }
}
