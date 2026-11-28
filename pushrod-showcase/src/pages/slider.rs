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

//! The Slider page: horizontal and vertical sliders with steps, ticks, and a disabled state.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::slider::Slider;

/// Slider: the options a slider can have.
pub struct SliderPage;

impl Page for SliderPage {
    fn name(&self) -> &'static str {
        "Slider"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.slider"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "Pick a number by dragging a knob: steps, ticks, value text, vertical, and disabled."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 250));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Plain, stepped with ticks, and disabled",
        );
        ui.widgets().add_child(
            root,
            Box::new(Slider::new(0.0, 100.0).value(40.0).at(10, 34).size(300, 24)),
        );
        ui.widgets().add_child(
            root,
            Box::new(
                Slider::new(0.0, 10.0)
                    .step(1.0)
                    .ticks(1.0)
                    .show_value(true)
                    .value(3.0)
                    .at(10, 70)
                    .size(300, 40),
            ),
        );

        let mut disabled = Slider::new(0.0, 100.0)
            .value(70.0)
            .at(10, 120)
            .size(300, 24);

        disabled.set_enabled(false);
        ui.widgets().add_child(root, Box::new(disabled));

        label(ui, root, 10, 160, 480, "Vertical");
        ui.widgets().add_child(
            root,
            Box::new(
                Slider::new(0.0, 100.0)
                    .vertical()
                    .value(60.0)
                    .at(10, 182)
                    .size(24, 60),
            ),
        );

        root
    }

    fn source(&self) -> &'static str {
        include_str!("slider.rs")
    }
}
