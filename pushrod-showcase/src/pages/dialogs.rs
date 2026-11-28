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

//! The Dialogs page: modal alerts that take the input until a button is chosen.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::{FLOATING, ShowcaseUi};
use pushrod::event::Event;
use pushrod::geometry::size;
use pushrod::layout::LayoutParams;
use pushrod::widget::WidgetId;
use pushrod_widgets::alert::{AlertExt, AlertKind};
use pushrod_widgets::button::Button;
use pushrod_widgets::text::Text;

/// What each button opens: its label, the alert's kind, title, message, and buttons.
const ALERTS: [(&str, AlertKind, &str, &str, &[&str]); 3] = [
    (
        "Information",
        AlertKind::Info,
        "Report saved",
        "Your changes to the quarterly report were saved.",
        &["OK"],
    ),
    (
        "Warning",
        AlertKind::Warning,
        "Delete the report?",
        "This cannot be undone.",
        &["Delete", "Cancel"],
    ),
    (
        "Question",
        AlertKind::Question,
        "Save changes?",
        "Your changes to the quarterly report have not been saved.",
        &["Save", "Don't Save", "Cancel"],
    ),
];

/// Dialogs: three alerts, and the page reports the button chosen.
pub struct DialogsPage;

impl Page for DialogsPage {
    fn name(&self) -> &'static str {
        "Dialogs and Alerts"
    }

    fn group(&self) -> Group {
        Group::Core
    }

    fn description(&self) -> &'static str {
        "Modal alerts: the window behind goes inert until a button, Return, or Escape closes it."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 150));

        label(ui, root, 10, 10, 480, "Open an alert");

        for (i, (name, ..)) in ALERTS.iter().enumerate() {
            ui.widgets().add_child(
                root,
                Box::new(Button::new(*name).at(10 + i as i32 * 130, 34).size(120, 24)),
            );
        }

        label(ui, root, 10, 80, 480, "You chose:");
        ui.widgets().add_child(
            root,
            Box::new(Text::new("nothing yet").at(10, 104).size(480, 18)),
        );

        root
    }

    fn handle_event(&self, ui: &mut ShowcaseUi<'_>, root: WidgetId, event: &Event) {
        let Event::Clicked { id, .. } = event else {
            return;
        };
        let children = ui.widgets().children_of(root).to_vec();
        let Some(position) = children
            .iter()
            .filter(|c| ui.get::<Button>(**c).is_some())
            .position(|c| c == id)
        else {
            return;
        };
        let result = children
            .iter()
            .copied()
            .rev()
            .find(|c| ui.get::<Text>(*c).is_some());
        let (_, kind, title, message, buttons) = ALERTS[position];

        let alert = ui.alert(kind, title, message, buttons, 0, move |_, ui, choice| {
            if let Some(text) = result.and_then(|r| ui.get_mut::<Text>(r)) {
                text.set_text(format!("{} (button {choice})", buttons[choice]));
            }
        });

        // The alert centers itself; keep the window's row from placing it.
        if let Some(widget) = ui.widgets().get_mut(alert.id()) {
            widget.set_layout_params(LayoutParams {
                stretch: FLOATING,
                ..LayoutParams::default()
            });
        }
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("dialogs.rs")
    }
}
