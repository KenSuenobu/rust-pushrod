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

//! What an application normally imports from the standard widgets:
//! `use pushrod_widgets::prelude::*;`.
//!
//! ```rust
//! use pushrod_widgets::prelude::*;
//!
//! let _ = (Button::new("OK").at(20, 60), Text::new("Hello"), Frame::new(), Panel::new());
//! ```

pub use crate::alert::{Alert, AlertExt, AlertHandle, AlertKind, AlertParts};
pub use crate::boxed_label::BoxedLabel;
pub use crate::button::Button;
pub use crate::check_box::CheckBox;
pub use crate::container::{Container, ContainerParts, ContentSize, ScrollBars};
pub use crate::custom_paint::CustomPaint;
pub use crate::frame::Frame;
pub use crate::glyph_button::GlyphButton;
pub use crate::group_box::GroupBox;
pub use crate::image_widget::{FitMode, ImageWidget};
pub use crate::list_view::{
    ListModel, ListView, ListViewParts, ModelChange, RowStyle, SelectionMode, VecModel,
};
pub use crate::panel::Panel;
pub use crate::progress_bar::ProgressBar;
pub use crate::radio_button::{RadioButton, RadioGroup};
pub use crate::scroll_bar::{ScrollBar, ScrollBarParts};
pub use crate::separator::Separator;
pub use crate::slider::Slider;
pub use crate::text::Text;
pub use crate::text_field::validate::{Mask, Validator, Validity};
pub use crate::text_field::{Accessory, IntegerField, NumberField, TextField};
