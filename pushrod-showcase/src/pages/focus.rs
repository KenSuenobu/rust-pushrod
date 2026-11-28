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

//! The Focus page: Tab order, Return for the default button, and Escape for Cancel.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::WidgetId;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::text_field::TextField;

/// Focus: a small form to move through with the keyboard.
pub struct FocusPage;

impl Page for FocusPage {
    fn name(&self) -> &'static str {
        "Focus and Tab"
    }

    fn group(&self) -> Group {
        Group::Core
    }

    fn description(&self) -> &'static str {
        "Keyboard focus: Tab and Shift-Tab move, Return presses OK, Escape presses Cancel."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 250));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Click a field, then press Tab or Shift-Tab",
        );
        ui.widgets().add_child(
            root,
            Box::new(
                TextField::new()
                    .at(10, 34)
                    .size(240, 24)
                    .with_label("First name"),
            ),
        );
        ui.widgets().add_child(
            root,
            Box::new(
                TextField::new()
                    .at(10, 68)
                    .size(240, 24)
                    .with_label("Last name"),
            ),
        );
        ui.widgets().add_child(
            root,
            Box::new(CheckBox::new("Send me updates").at(10, 104).size(240, 20)),
        );
        ui.widgets().add_child(
            root,
            Box::new(Button::new("OK").at(10, 140).size(80, 24).default_action()),
        );
        ui.widgets().add_child(
            root,
            Box::new(
                Button::new("Cancel")
                    .at(100, 140)
                    .size(80, 24)
                    .cancel_action(),
            ),
        );

        label(ui, root, 10, 184, 480, "Tab: next    Shift-Tab: previous");
        label(
            ui,
            root,
            10,
            206,
            480,
            "Return: press OK    Escape: press Cancel",
        );
        label(
            ui,
            root,
            10,
            228,
            480,
            "The focused widget is drawn with a dotted outline.",
        );

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("focus.rs")
    }
}
