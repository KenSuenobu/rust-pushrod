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

//! The Appearance page: how widgets look in each state.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text_field::TextField;

/// Appearance: normal, checked, indeterminate, and disabled widgets side by side.
pub struct AppearancePage;

impl Page for AppearancePage {
    fn name(&self) -> &'static str {
        "Appearance"
    }

    fn group(&self) -> Group {
        Group::Core
    }

    fn description(&self) -> &'static str {
        "Per-state appearance: normal, hovered, pressed, focused, checked, and disabled."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 270));

        label(ui, root, 10, 10, 480, "Buttons: normal, checked, disabled");
        ui.widgets().add_child(
            root,
            Box::new(Button::new("Normal").at(10, 34).size(100, 24)),
        );

        let mut checked = Button::new("Checked").at(120, 34).size(100, 24).toggle();

        checked.set_checked(true);
        ui.widgets().add_child(root, Box::new(checked));

        let mut disabled = Button::new("Disabled").at(230, 34).size(100, 24);

        disabled.set_enabled(false);
        ui.widgets().add_child(root, Box::new(disabled));

        label(
            ui,
            root,
            10,
            80,
            480,
            "Check boxes: off, on, mixed, disabled",
        );
        ui.widgets().add_child(
            root,
            Box::new(CheckBox::new("Off").at(10, 104).size(100, 20)),
        );
        ui.widgets().add_child(
            root,
            Box::new(CheckBox::new("On").at(120, 104).size(100, 20).checked(true)),
        );
        ui.widgets().add_child(
            root,
            Box::new(
                CheckBox::new("Mixed")
                    .at(230, 104)
                    .size(100, 20)
                    .indeterminate(),
            ),
        );

        let mut off = CheckBox::new("Disabled")
            .at(340, 104)
            .size(100, 20)
            .checked(true);

        off.set_enabled(false);
        ui.widgets().add_child(root, Box::new(off));

        label(ui, root, 10, 144, 480, "Fields: enabled and disabled");
        ui.widgets().add_child(
            root,
            Box::new(
                TextField::new()
                    .at(10, 168)
                    .size(200, 24)
                    .with_label("Name"),
            ),
        );

        let mut field = TextField::new()
            .at(230, 168)
            .size(200, 24)
            .with_label("Locked");

        field.set_enabled(false);
        ui.widgets().add_child(root, Box::new(field));

        let mut slider = Slider::new(0.0, 100.0)
            .value(60.0)
            .at(10, 204)
            .size(200, 24);

        slider.set_enabled(false);
        ui.widgets().add_child(root, Box::new(slider));

        label(
            ui,
            root,
            10,
            240,
            500,
            "Hover, press, or Tab to any widget above to see those states.",
        );

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("appearance.rs")
    }
}
