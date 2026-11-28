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

//! The showcase's pages: one file per widget or feature.

use crate::property_panel::PageRoot;
use crate::registry::Registry;
use crate::shell::ShowcaseUi;
use pushrod::geometry::Size;
use pushrod::widget::WidgetId;
use pushrod_widgets::text::Text;

mod appearance;
mod boxed_label;
mod button;
mod check_box;
mod dialogs;
mod focus;
mod form_layout;
mod frame;
mod glyph_button;
mod group_box;
mod image_widget;
mod panel;
mod progress_bar;
mod radio_button;
mod resource_form;
mod resources_by_id;
mod row_and_column;
mod separator;
mod slider;
mod text;
mod text_field;

/// Registers every page.  Adding a page is one line here.
///
/// # Arguments
///
/// * `registry` - The registry to add the pages to.
pub fn register_all(registry: &mut Registry) {
    registry.register(text::TextPage);
    registry.register(appearance::AppearancePage);
    registry.register(focus::FocusPage);
    registry.register(dialogs::DialogsPage);
    registry.register(button::ButtonPage);
    registry.register(check_box::CheckBoxPage);
    registry.register(radio_button::RadioButtonPage);
    registry.register(progress_bar::ProgressBarPage);
    registry.register(separator::SeparatorPage);
    registry.register(group_box::GroupBoxPage);
    registry.register(boxed_label::BoxedLabelPage);
    registry.register(frame::FramePage);
    registry.register(glyph_button::GlyphButtonPage);
    registry.register(slider::SliderPage);
    registry.register(image_widget::ImageWidgetPage);
    registry.register(panel::PanelPage);
    registry.register(row_and_column::RowAndColumnPage);
    registry.register(form_layout::FormLayoutPage);
    registry.register(resource_form::ResourceFormPage);
    registry.register(resources_by_id::ResourcesByIdPage);
    registry.register(text_field::TextFieldPage);
}

/// Adds the root panel of a page.
///
/// # Arguments
///
/// * `ui` - The callback handle.
/// * `page_size` - The page's width and height in pixels.
///
/// # Returns
///
/// The panel's ID.
pub(crate) fn page_root(ui: &mut ShowcaseUi<'_>, page_size: Size) -> WidgetId {
    ui.widgets().add(Box::new(PageRoot::new(page_size)))
}

/// Adds a plain text label.
///
/// # Arguments
///
/// * `ui` - The callback handle.
/// * `parent` - The widget to add the label to.
/// * `x`, `y` - The label's upper left-hand corner, relative to the parent.
/// * `width` - The label's width in pixels.
/// * `text` - The label's text.
///
/// # Returns
///
/// The label's ID.
pub(crate) fn label(
    ui: &mut ShowcaseUi<'_>,
    parent: WidgetId,
    x: i32,
    y: i32,
    width: u32,
    text: &str,
) -> WidgetId {
    ui.widgets()
        .add_child(parent, Box::new(Text::new(text).at(x, y).size(width, 18)))
}
