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

//! The Row and Column page: panels that lay out their children.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::color::Color;
use pushrod::geometry::size;
use pushrod::layout::{Column, Margin, Row};
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::button::Button;
use pushrod_widgets::panel::Panel;

/// Row and Column: the same three buttons placed both ways.
pub struct RowAndColumnPage;

impl Page for RowAndColumnPage {
    fn name(&self) -> &'static str {
        "Row and Column"
    }

    fn group(&self) -> Group {
        Group::Layouts
    }

    fn description(&self) -> &'static str {
        "Layouts place children left to right, or top to bottom."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 240));

        label(ui, root, 10, 10, 480, "Row");

        let mut row = Panel::new().at(10, 34).size(340, 44);

        row.set_color(Color::RGBA(225, 225, 225, 255));
        row.set_layout(Box::new(Row::new().spacing(8).margins(Margin::all(8))));

        let row = ui.widgets().add_child(root, Box::new(row));

        label(ui, root, 10, 94, 480, "Column");

        let mut column = Panel::new().at(10, 118).size(160, 112);

        column.set_color(Color::RGBA(225, 225, 225, 255));
        column.set_layout(Box::new(Column::new().spacing(8).margins(Margin::all(8))));

        let column = ui.widgets().add_child(root, Box::new(column));

        for parent in [row, column] {
            for text in ["One", "Two", "Three"] {
                ui.widgets()
                    .add_child(parent, Box::new(Button::new(text).size(90, 24)));
            }
        }

        root
    }

    fn source(&self) -> &'static str {
        include_str!("row_and_column.rs")
    }
}
