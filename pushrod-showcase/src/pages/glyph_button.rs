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

//! The Glyph Button page: one-character buttons, normal, disabled, and activate-on-press.

use super::{label, page_root};
use crate::page::{Group, Page};
use crate::shell::ShowcaseUi;
use pushrod::color::Color;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::glyph_button::GlyphButton;

/// Glyph Button: a button that shows one character.
pub struct GlyphButtonPage;

impl Page for GlyphButtonPage {
    fn name(&self) -> &'static str {
        "Glyph Button"
    }

    fn widget_types(&self) -> &'static [&'static str] {
        &["pushrod.glyph_button"]
    }

    fn group(&self) -> Group {
        Group::Standard
    }

    fn description(&self) -> &'static str {
        "A small button that shows one character: normal, colored, disabled, and on press."
    }

    fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {
        let root = page_root(ui, size(520, 140));

        label(
            ui,
            root,
            10,
            10,
            480,
            "Normal, colored, disabled, and activate on press",
        );
        ui.widgets().add_child(
            root,
            Box::new(GlyphButton::new('+').at(10, 40).size(32, 32)),
        );
        ui.widgets().add_child(
            root,
            Box::new(
                GlyphButton::new('x')
                    .at(52, 40)
                    .size(32, 32)
                    .font_color(Color::RED),
            ),
        );

        let mut disabled = GlyphButton::new('-').at(94, 40).size(32, 32);

        disabled.set_enabled(false);
        ui.widgets().add_child(root, Box::new(disabled));
        ui.widgets().add_child(
            root,
            Box::new(
                GlyphButton::new('>')
                    .at(136, 40)
                    .size(32, 32)
                    .activate_on_press(),
            ),
        );

        root
    }

    fn source(&self) -> &'static str {
        crate::source_view::build_source("glyph_button.rs")
    }
}
