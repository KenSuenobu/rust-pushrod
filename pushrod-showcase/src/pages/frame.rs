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

//! The Frame page: a fill with a border, at three border widths.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::color::Color;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::frame::Frame;

/// Frame: border widths.
pub struct FramePage;

impl Page for FramePage {
    fn name(&self) -> &'static str {
        "Frame"
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A filled rectangle with a border."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 130));

        label(ui, root, 10, 10, 480, "Border widths 1, 3, and 6");

        for (i, width) in [1u8, 3, 6].iter().enumerate() {
            let mut sample = Frame::new()
                .at(10 + i as i32 * 90, 40)
                .size(80, 60)
                .border_color(Color::BLUE)
                .border_width(*width);

            sample.set_color(Color::CYAN);
            ui.widgets().add_child(root, Box::new(sample));
        }

        root
    }

    fn source(&self) -> &'static str {
        include_str!("frame.rs")
    }
}
