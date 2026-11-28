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

//! The Resource Form page: a form loaded from the showcase's resource file.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::resources;
use crate::shell::ShowcaseUi;
use pushrod::geometry::{point, size};
use pushrod::widget::WidgetId;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;

/// Resource form: the contact form comes from `resources/showcase.prsc`, not from code.
pub struct ResourceFormPage;

impl Page for ResourceFormPage {
    fn name(&self) -> &'static str {
        "Form from a Resource File"
    }

    fn group(&self) -> Group {
        Group::Forms
    }

    fn description(&self) -> &'static str {
        "A form loaded from a resource file by its typed ID, with its title found by StringId."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 250));
        let title = ui
            .resources()
            .string(resources::FORM_TITLE)
            .unwrap_or("(missing string)")
            .to_string();

        label(
            ui,
            root,
            10,
            10,
            480,
            &format!("{title}: FormId(1), loaded from showcase.prsc"),
        );

        // The form was loaded once when the showcase started; borrow it from the shelf.
        let form = resources::shelf(ui.widgets())
            .and_then(|shelf| ui.widgets().children_of(shelf).first().copied());

        match form {
            Some(form) => {
                let _ = ui.widgets().reparent(form, root);

                if let Some(widget) = ui.widgets().get_mut(form) {
                    widget.set_origin(point(10, 40));
                }
            }
            None => {
                ui.widgets().add_child(
                    root,
                    Box::new(
                        Text::new("The resource form is not loaded.")
                            .at(10, 40)
                            .size(480, 18),
                    ),
                );
            }
        }

        label(
            ui,
            root,
            10,
            204,
            480,
            "Name, Email, OK, and Cancel are objects in the file.",
        );

        root
    }

    fn leave(&self, ui: &mut ShowcaseUi<'_>, root: WidgetId) {
        // Give the form back, so it survives this page's widgets being removed.
        let Some(shelf) = resources::shelf(ui.widgets()) else {
            return;
        };
        let children = ui.widgets().children_of(root).to_vec();
        let forms: Vec<WidgetId> = children
            .into_iter()
            .filter(|id| ui.get::<Panel>(*id).is_some())
            .collect();

        for form in forms {
            let _ = ui.widgets().reparent(form, shelf);
        }
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("resource_form.rs")
    }
}
