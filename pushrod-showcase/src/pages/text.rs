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

//! The Text page: justification, truncation, and wrapping.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::paint::{Truncate, Wrap};
use pushrod::widget::WidgetId;
use pushrod_widgets::text::{Text, TextJustify};

/// Text: left, centered, and right justified; truncated; wrapped.
pub struct TextPage;

impl Page for TextPage {
    fn name(&self) -> &'static str {
        "Text"
    }

    fn group(&self) -> Group {
        Group::Core
    }

    fn description(&self) -> &'static str {
        "Static text: justification, truncation, and wrapping."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 330));

        label(ui, root, 10, 10, 480, "Justify: left, center, right");

        for (i, (justify, text)) in [
            (TextJustify::Left, "Jane Doe"),
            (TextJustify::Center, "Anytown"),
            (TextJustify::Right, "jane.doe@example.com"),
        ]
        .into_iter()
        .enumerate()
        {
            let sample = Text::new(text)
                .at(10, 34 + i as i32 * 26)
                .size(300, 18)
                .justify(justify);

            ui.widgets().add_child(root, Box::new(sample));
        }

        label(ui, root, 10, 124, 480, "Truncate: end, start, middle");

        for (i, mode) in [Truncate::End, Truncate::Start, Truncate::Middle]
            .into_iter()
            .enumerate()
        {
            let mut sample = Text::new("Quarterly sales report for the western region.txt")
                .at(10, 148 + i as i32 * 26)
                .size(300, 18);

            sample.set_truncate(mode);
            ui.widgets().add_child(root, Box::new(sample));
        }

        label(ui, root, 10, 238, 480, "Wrap: word, at most four lines");

        let mut paragraph = Text::new(
            "Your changes to the quarterly report have not been saved.\nClose the document anyway and discard them?",
        )
        .at(10, 262)
        .size(300, 60);

        paragraph.set_wrap(Wrap::Word);
        paragraph.set_max_lines(Some(4));
        ui.widgets().add_child(root, Box::new(paragraph));

        root
    }

    fn source(&self) -> &'static str {
        include_str!("text.rs")
    }
}
