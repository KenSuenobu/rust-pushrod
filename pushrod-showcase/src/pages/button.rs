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

//! The Button page: normal, disabled, default, cancel, and toggle buttons.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::button::Button;

/// Button: the behaviors a button can have.
pub struct ButtonPage;

impl Page for ButtonPage {
    fn name(&self) -> &'static str {
        "Button"
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A push button: normal, disabled, default, cancel, and toggle."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 170));

        label(ui, root, 10, 10, 480, "Normal and disabled");
        ui.widgets().add_child(
            root,
            Box::new(Button::new("Normal").at(10, 34).size(120, 24)),
        );

        let mut disabled = Button::new("Disabled").at(150, 34).size(120, 24);

        disabled.set_enabled(false);
        ui.widgets().add_child(root, Box::new(disabled));

        label(ui, root, 10, 80, 480, "Default, cancel, and toggle");
        ui.widgets().add_child(
            root,
            Box::new(Button::new("OK").at(10, 104).size(80, 24).default_action()),
        );
        ui.widgets().add_child(
            root,
            Box::new(
                Button::new("Cancel")
                    .at(100, 104)
                    .size(80, 24)
                    .cancel_action(),
            ),
        );

        let mut bold = Button::new("Bold").at(190, 104).size(80, 24).toggle();

        bold.set_checked(true);
        ui.widgets().add_child(root, Box::new(bold));

        root
    }

    fn source(&self) -> &'static str {
        include_str!("button.rs")
    }
}
