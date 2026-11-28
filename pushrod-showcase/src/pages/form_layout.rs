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

//! The Form Layout page: aligned "Label: field" rows that stack when the form is narrow.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::color::Color;
use pushrod::event::Event;
use pushrod::geometry::size;
use pushrod::layout::{FormLayout, Margin};
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The form's width when the page opens, in pixels.
const WIDE: f64 = 440.0;

/// The narrowest width the slider offers, in pixels.
const NARROW: f64 = 200.0;

/// Form layout: drag the slider to resize the form and watch the labels move above their fields.
pub struct FormLayoutPage;

impl Page for FormLayoutPage {
    fn name(&self) -> &'static str {
        "Form Layout"
    }

    fn group(&self) -> Group {
        Group::Layouts
    }

    fn description(&self) -> &'static str {
        "Aligned label and field rows; drag the width slider to see them stack when narrow."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 290));

        label(ui, root, 10, 10, 480, "Form width (drag the slider)");
        ui.widgets().add_child(
            root,
            Box::new(
                Slider::new(NARROW, WIDE)
                    .value(WIDE)
                    .step(10.0)
                    .at(10, 34)
                    .size(300, 24),
            ),
        );

        let mut form = Panel::new().at(10, 74).size(WIDE as u32, 200);

        form.set_color(Color::RGBA(225, 225, 225, 255));

        let form = ui.widgets().add_child(root, Box::new(form));
        let mut layout = FormLayout::new()
            .spacing(8)
            .column_spacing(8)
            .margins(Margin::all(8));

        for name in ["Name", "Email", "City"] {
            let caption = ui
                .widgets()
                .add_child(form, Box::new(Text::new(format!("{name}:")).size(80, 18)));
            let field = ui.widgets().add_child(
                form,
                Box::new(TextField::new().size(200, 24).with_label(name)),
            );

            layout.add_row(caption, field);
        }

        let check = ui.widgets().add_child(
            form,
            Box::new(CheckBox::new("Send me updates").size(200, 20)),
        );

        layout.add_full_row(check);

        if let Some(panel) = ui.widgets().get_mut(form) {
            panel.set_layout(Box::new(layout));
        }

        root
    }

    fn handle_event(&self, ui: &mut ShowcaseUi<'_>, root: WidgetId, event: &Event) {
        let Event::ValueChanged { id } = event else {
            return;
        };
        let children = ui.widgets().children_of(root).to_vec();
        let Some(slider) = children.iter().copied().find(|c| c == id) else {
            return;
        };
        let Some(width) = ui.get::<Slider>(slider).map(Slider::get_value) else {
            return;
        };

        for child in children {
            if let Some(panel) = ui.get_mut::<Panel>(child) {
                let height = panel.get_size().h;

                panel.set_size(size(width as u32, height));
            }
        }
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("form_layout.rs")
    }
}
