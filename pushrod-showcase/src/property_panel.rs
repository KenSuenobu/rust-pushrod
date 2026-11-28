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

//! The property panel: click a widget on a page to select it (it gets an outline), then
//! change any of its properties live.
//!
//! The panel lists the selected widget's [`properties`](pushrod::widget::Widget::properties)
//! through introspection, so it works for any registered widget with no per-widget code.  Each
//! property gets an editor chosen from its kind: a check box for true or false, a text field for
//! numbers, text, and colors (`#rrggbb` or `#rrggbbaa`), and a button that cycles through the
//! choices for the alignments.  Edits apply at once through
//! [`Ui::set_widget_property`](pushrod::engine::Ui::set_widget_property), which emits a
//! `PropertyChanged` engine event.  **Reset** rebuilds the page.

use crate::shell::{ShowcaseState, ShowcaseUi, add_panel, fill_width, select_page};
use pushrod::color::Color;
use pushrod::event::{Event, EventCtx, EventMask, EventPhase, EventTarget, Propagation};
use pushrod::geometry::{Size, point};
use pushrod::input::MouseButton;
use pushrod::layout::{Align, Alignment, Column, LayoutParams, Margin, Row};
use pushrod::paint::{self, Painter};
use pushrod::properties::{Property, PropertyKind};
use pushrod::registry::PropertyValue;
use pushrod::widget::{ROOT_WIDGET_ID, Widget, WidgetCore, WidgetId};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::container::{Container, ContentSize};
use pushrod_widgets::frame::Frame;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The width of the panel's column, in pixels.
pub const PANEL_W: u32 = 276;
/// The starting height of the scrolling property list; it stretches to fill the column.
const LIST_H: u32 = 200;
/// The height of one property row, in pixels.
const ROW_H: u32 = 24;
/// The width of a property's name, in pixels.
const NAME_W: u32 = 110;
/// The width of a property's editor, in pixels.
const EDITOR_W: u32 = 160;
/// The thickness of the selection outline, in pixels.
const OUTLINE: u32 = 1;
/// The gap between the selected widget and its outline, in pixels.
const GAP: u32 = 4;
/// The properties edited by cycling through fixed choices, with the choices.
const CHOICES: [(&str, &[&str]); 2] = [
    ("align_horizontal", &["start", "center", "end", "fill"]),
    ("align_vertical", &["start", "center", "end", "fill"]),
];

/// How one property is edited.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditorKind {
    /// A check box (true or false).
    Check,
    /// A text field holding a value of the given kind.
    Field(PropertyKind),
    /// A button that cycles through fixed choices.
    Cycle,
}

/// One editor in the panel.
#[derive(Debug, Clone)]
pub struct Editor {
    /// The editor widget.
    pub id: WidgetId,
    /// The name of the property it edits.
    pub name: String,
    /// How it edits.
    pub kind: EditorKind,
}

/// The panel's widgets and the selection.
#[derive(Debug, Default)]
pub struct PropertyPanel {
    /// The **Reset** button.
    pub reset: WidgetId,
    /// The text that reports the selection or the last error.
    pub status: WidgetId,
    /// The content of the scrolling list of property rows.
    pub list: WidgetId,
    /// The selected page widget.
    pub selected: Option<WidgetId>,
    /// The four strips of the selection outline.
    pub outline: Vec<WidgetId>,
    /// The editors now in the list.
    pub editors: Vec<Editor>,
}

/// Posted by a [`PageRoot`] when the pointer is pressed on one of the page's widgets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PickRequest {
    /// The widget under the pointer.
    pub id: WidgetId,
}

/// The root panel of a showcase page.  It sees every left-button press on the page in the
/// capture phase, before the widget under the pointer, so a widget that consumes the press
/// (a slider, say) can still be selected.  It posts a [`PickRequest`] and lets the press go on.
#[derive(Debug)]
pub struct PageRoot {
    core: WidgetCore,
}

impl PageRoot {
    /// Creates a transparent page root.
    ///
    /// # Arguments
    ///
    /// * `extent` - The page's width and height in pixels.
    ///
    /// # Returns
    ///
    /// The page root.
    pub fn new(extent: Size) -> Self {
        let mut core = WidgetCore::new(point(0, 0), extent);

        core.set_event_mask(EventMask::POINTER_BUTTONS);

        Self { core }
    }
}

impl Widget for PageRoot {
    /// Returns the widget's shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the widget's shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Wants presses before the widgets below it.
    fn handles_capture(&self) -> bool {
        true
    }

    /// Fills the page with its color.
    fn paint(&self, p: &mut dyn Painter) {
        paint::fill(p, self.style().background.unwrap_or(self.get_color()));
    }

    /// Posts a [`PickRequest`] for a left press on a widget below the page root.
    ///
    /// # Arguments
    ///
    /// * `event` - The event.
    /// * `ctx` - The context the request is posted with.
    ///
    /// # Returns
    ///
    /// [`Propagation::Continue`] always: selecting never keeps the widget from its press.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::PointerPressed {
            id,
            button: MouseButton::Left,
            ..
        } = event
            && ctx.phase() == EventPhase::Capture
        {
            ctx.post(EventTarget::ApplicationOnly, PickRequest { id: *id });
        }

        Propagation::Continue
    }
}

/// Returns the fixed choices of a property, if it is edited by cycling.
///
/// # Arguments
///
/// * `name` - The property name.
///
/// # Returns
///
/// The choices, or `None` for a property edited another way.
pub fn choices_for(name: &str) -> Option<&'static [&'static str]> {
    CHOICES.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

/// Returns the choice after `current`, wrapping to the first.
///
/// # Arguments
///
/// * `choices` - The choices.
/// * `current` - The current choice; one not in the list starts at the first.
///
/// # Returns
///
/// The next choice.
pub fn next_choice<'a>(choices: &[&'a str], current: &str) -> &'a str {
    let next = choices
        .iter()
        .position(|c| *c == current)
        .map_or(0, |i| (i + 1) % choices.len());

    choices[next]
}

/// Chooses how a property is edited.
///
/// # Arguments
///
/// * `property` - The property.
///
/// # Returns
///
/// The editor kind.
pub fn editor_for(property: &Property) -> EditorKind {
    match property.kind {
        PropertyKind::Bool => EditorKind::Check,
        PropertyKind::Text if choices_for(&property.name).is_some() => EditorKind::Cycle,
        kind => EditorKind::Field(kind),
    }
}

/// Formats a value for its text field.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// The text: colors as `#rrggbbaa`.
pub fn format_value(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Bool(b) => b.to_string(),
        PropertyValue::Int(i) => i.to_string(),
        PropertyValue::Float(f) => f.to_string(),
        PropertyValue::Text(t) => t.clone(),
        PropertyValue::Color(c) => format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a),
    }
}

/// Reads a value from text typed in a field.
///
/// # Arguments
///
/// * `kind` - The kind of value to read.
/// * `text` - The text.
///
/// # Returns
///
/// The value, or `None` when the text is not (yet) a valid value of that kind.
pub fn parse_value(kind: PropertyKind, text: &str) -> Option<PropertyValue> {
    let text = text.trim();

    match kind {
        PropertyKind::Bool => text.parse().ok().map(PropertyValue::Bool),
        PropertyKind::Int => text.parse().ok().map(PropertyValue::Int),
        PropertyKind::Float => text
            .parse::<f64>()
            .ok()
            .filter(|f| f.is_finite())
            .map(PropertyValue::Float),
        PropertyKind::Text => Some(PropertyValue::Text(text.to_string())),
        PropertyKind::Color => parse_color(text).map(PropertyValue::Color),
    }
}

/// Reads `#rrggbb` or `#rrggbbaa`.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The color, or `None` if the text is not in either form.
fn parse_color(text: &str) -> Option<Color> {
    let hex = text.strip_prefix('#')?;

    if (hex.len() != 6 && hex.len() != 8) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();

    Some(Color::RGBA(
        byte(0)?,
        byte(2)?,
        byte(4)?,
        if hex.len() == 8 { byte(6)? } else { 255 },
    ))
}

/// Returns whether `id` is `root` or below it.
///
/// # Arguments
///
/// * `ui` - The callback handle.
/// * `root` - The top of the subtree.
/// * `id` - The widget to look for.
///
/// # Returns
///
/// `true` if `id` is in the subtree.
fn in_subtree(ui: &mut ShowcaseUi<'_>, root: WidgetId, id: WidgetId) -> bool {
    let mut current = id;

    // The depth is bounded by the number of widgets, so a broken parent link cannot loop.
    for _ in 0..1024 {
        if current == root {
            return true;
        }

        let Some(parent) = ui.widgets().get(current).map(|w| w.get_parent()) else {
            return false;
        };

        if parent == current || current == ROOT_WIDGET_ID {
            return false;
        }

        current = parent;
    }

    false
}

/// Builds the panel's widgets, top to bottom, as in the roadmap mockup: the **Properties**
/// heading, the scrolling property list (a name on the left and its editor on the right), the
/// status text, and the **Reset** button at the bottom.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `parent` - The column on the right of the window the panel goes in.
///
/// # Returns
///
/// The panel, with no selection.
pub fn build(cache: &mut pushrod::cache::WidgetCache, parent: WidgetId) -> PropertyPanel {
    let heading = pushrod::font::Font::new("", 16, pushrod::color::FontStyle::NORMAL);
    let title = cache.add_child(
        parent,
        Box::new(Text::new("Properties").size(PANEL_W, 24).font(heading)),
    );

    if let Some(widget) = cache.get_mut(title) {
        widget.set_layout_params(fill_width());
    }

    let container = cache.add_child(
        parent,
        Box::new(
            Container::new(
                point(0, 0),
                pushrod::geometry::size(PANEL_W, LIST_H),
                pushrod::geometry::size(0, 0),
            )
            .content_mode(ContentSize::FollowWidth { fit_height: true }),
        ),
    );

    // The list takes all the height the status and Reset do not.
    if let Some(widget) = cache.get_mut(container) {
        widget.set_layout_params(LayoutParams {
            stretch: 1,
            alignment: Alignment::new(Align::Fill, Align::Fill),
            ..LayoutParams::default()
        });
    }

    let list = cache.get_as::<Container>(container).map_or(container, |c| {
        pushrod::composite::TypedParts::typed_parts(c).content
    });

    if let Some(content) = cache.get_mut(list) {
        content.set_layout(Box::new(Column::new().spacing(4).margins(Margin::all(0))));
    }

    let mut note = Text::new("").size(PANEL_W, 20);

    note.set_font_color(Color::GRAY);

    let status = cache.add_child(parent, Box::new(note));

    if let Some(widget) = cache.get_mut(status) {
        widget.set_layout_params(fill_width());
    }

    let reset = cache.add_child(parent, Box::new(Button::new("Reset").size(100, 28)));

    PropertyPanel {
        reset,
        status,
        list,
        ..PropertyPanel::default()
    }
}

/// Sets the status text.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `message` - The text.
fn set_status(state: &ShowcaseState, ui: &mut ShowcaseUi<'_>, message: String) {
    if let Some(text) = ui.get_mut::<Text>(state.panel.status) {
        text.set_text(message);
    }
}

/// Forgets the selection: removes the rows and the outline.  Call it when the page is replaced
/// (the outline goes with the old page's widgets, so only the bookkeeping is cleared).
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
pub fn clear(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    clear_rows(state, ui);
    state.panel.selected = None;
    state.panel.outline.clear();
    set_status(
        state,
        ui,
        "Click a widget on the page to select it.".to_string(),
    );
}

/// Removes every row from the property list.
fn clear_rows(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    let cache = ui.widgets();

    for child in cache.children_of(state.panel.list).to_vec() {
        let _ = cache.remove(child);
    }

    state.panel.editors.clear();
}

/// Selects a page widget: outlines it and lists its properties.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `id` - The widget to select.
pub fn select_widget(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, id: WidgetId) {
    let Some(properties) = ui.widgets().get(id).map(|w| w.properties()) else {
        return;
    };

    state.panel.selected = Some(id);
    clear_rows(state, ui);

    for property in &properties {
        add_row(state, ui, property);
    }

    draw_outline(state, ui);
    set_status(
        state,
        ui,
        format!("Selected widget {id}: {} properties.", properties.len()),
    );
}

/// Adds one property row: its name and an editor.
fn add_row(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, property: &Property) {
    let cache = ui.widgets();
    let row = add_panel(
        cache,
        state.panel.list,
        Box::new(Row::new().spacing(8)),
        fill_width(),
        PANEL_W,
    );

    if let Some(panel) = cache.get_mut(row) {
        panel.set_size(pushrod::geometry::size(PANEL_W, ROW_H));
    }

    // The name takes the space the editor leaves, so the editors line up on the right.
    let name = cache.add_child(row, Box::new(Text::new(&property.name).size(NAME_W, 20)));

    if let Some(widget) = cache.get_mut(name) {
        widget.set_layout_params(LayoutParams {
            stretch: 1,
            alignment: Alignment::new(Align::Fill, Align::Center),
            ..LayoutParams::default()
        });
    }

    let kind = editor_for(property);
    let editor = match kind {
        EditorKind::Check => {
            let checked = property.value == PropertyValue::Bool(true);

            cache.add_child(
                row,
                Box::new(CheckBox::new("").size(EDITOR_W, ROW_H).checked(checked)),
            )
        }
        EditorKind::Cycle => cache.add_child(
            row,
            Box::new(Button::new(format_value(&property.value)).size(EDITOR_W, ROW_H)),
        ),
        EditorKind::Field(_) => {
            let mut field = TextField::new().size(EDITOR_W, ROW_H);

            field.set_text(&format_value(&property.value));
            cache.add_child(row, Box::new(field))
        }
    };

    state.panel.editors.push(Editor {
        id: editor,
        name: property.name.clone(),
        kind,
    });
}

/// Draws the selection outline: four strips around the selected widget, added to the page so
/// the outline goes when the page does.  The strips leave the widget itself uncovered, so it
/// still gets its clicks.
fn draw_outline(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    let (Some(selected), Some(root)) = (state.panel.selected, state.page_root) else {
        return;
    };

    let cache = ui.widgets();

    for strip in state.panel.outline.drain(..) {
        let _ = cache.remove(strip);
    }

    let Some(extent) = cache.get(selected).map(|w| w.get_size()) else {
        return;
    };
    let origin = cache.absolute_origin(selected);
    let base = cache.absolute_origin(root);
    let (x, y) = (origin.x - base.x, origin.y - base.y);
    // The outline sits GAP pixels outside the widget, like the dotted box in the mockup.
    let (t, g) = (OUTLINE, GAP);
    let (w, h) = (extent.w + 2 * g, extent.h + 2 * g);
    let (x, y) = (x - g as i32, y - g as i32);
    let strips = [
        (x - t as i32, y - t as i32, w + 2 * t, t),
        (x - t as i32, y + h as i32, w + 2 * t, t),
        (x - t as i32, y, t, h),
        (x + w as i32, y, t, h),
    ];

    for (sx, sy, sw, sh) in strips {
        let mut strip = Frame::new().at(sx, sy).size(sw, sh).border_width(0);

        strip.set_color(Color::BLACK);
        state
            .panel
            .outline
            .push(cache.add_child(root, Box::new(strip)));
    }
}

/// Applies an edit and reports the result in the status text.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `name` - The property name.
/// * `value` - The new value.
pub fn apply(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, name: &str, value: PropertyValue) {
    let Some(selected) = state.panel.selected else {
        return;
    };

    match ui.set_widget_property(selected, name, &value) {
        Ok(_) => {
            draw_outline(state, ui);
            set_status(state, ui, format!("Set {name}."));
        }
        Err(error) => set_status(state, ui, error.to_string()),
    }
}

/// Handles the events the panel cares about: an editor edits, and **Reset** rebuilds the page.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `event` - The event.
pub fn handle(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, event: &Event) {
    match event {
        Event::Toggled { id, checked } => {
            if let Some(editor) = state.panel.editors.iter().find(|e| e.id == *id).cloned() {
                apply(state, ui, &editor.name, PropertyValue::Bool(*checked));
            }
        }
        Event::TextChanged { id } => {
            let Some(editor) = state.panel.editors.iter().find(|e| e.id == *id).cloned() else {
                return;
            };
            let EditorKind::Field(kind) = editor.kind else {
                return;
            };
            let text = ui
                .get::<TextField>(*id)
                .map(|f| f.text())
                .unwrap_or_default();

            // Half-typed text (such as a lone "-") is not an error: wait for a valid value.
            if let Some(value) = parse_value(kind, &text) {
                apply(state, ui, &editor.name, value);
            }
        }
        Event::Clicked { id, .. } => {
            if *id == state.panel.reset {
                if let Some(index) = state.selected {
                    select_page(state, ui, index);
                }

                return;
            }

            let Some(editor) = state.panel.editors.iter().find(|e| e.id == *id).cloned() else {
                return;
            };

            if let (EditorKind::Cycle, Some(choices)) = (editor.kind, choices_for(&editor.name)) {
                let current = ui
                    .get::<Button>(*id)
                    .map(|b| b.get_text())
                    .unwrap_or_default();
                let next = next_choice(choices, &current);

                if let Some(button) = ui.get_mut::<Button>(*id) {
                    button.set_text(next.to_string());
                }

                apply(
                    state,
                    ui,
                    &editor.name,
                    PropertyValue::Text(next.to_string()),
                );
            }
        }
        _ => {}
    }
}

/// Handles a [`PickRequest`]: selects the widget unless it is the page root, the outline, or
/// already selected.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `id` - The widget that was pressed.
pub fn pick(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, id: WidgetId) {
    let Some(root) = state.page_root else {
        return;
    };

    if id != root
        && state.panel.selected != Some(id)
        && !state.panel.outline.contains(&id)
        && in_subtree(ui, root, id)
    {
        select_widget(state, ui, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_round_trip_through_text() {
        let value = PropertyValue::Color(Color::RGBA(1, 2, 255, 128));
        let text = format_value(&value);

        assert_eq!(text, "#0102ff80");
        assert_eq!(parse_value(PropertyKind::Color, &text), Some(value));
        assert_eq!(
            parse_value(PropertyKind::Color, "#ff0000"),
            Some(PropertyValue::Color(Color::RGB(255, 0, 0)))
        );
    }

    #[test]
    fn bad_text_is_not_a_value() {
        assert_eq!(parse_value(PropertyKind::Int, "-"), None);
        assert_eq!(parse_value(PropertyKind::Int, "1.5"), None);
        assert_eq!(parse_value(PropertyKind::Float, "NaN"), None);
        assert_eq!(parse_value(PropertyKind::Color, "red"), None);
        assert_eq!(parse_value(PropertyKind::Color, "#12345"), None);
        assert_eq!(parse_value(PropertyKind::Color, "#+12345"), None);
        assert_eq!(
            parse_value(PropertyKind::Int, " 42 "),
            Some(PropertyValue::Int(42))
        );
        assert_eq!(
            parse_value(PropertyKind::Float, "2.5"),
            Some(PropertyValue::Float(2.5))
        );
    }

    #[test]
    fn choices_cycle_and_wrap() {
        let choices = choices_for("align_horizontal").unwrap();

        assert_eq!(next_choice(choices, "start"), "center");
        assert_eq!(next_choice(choices, "fill"), "start");
        assert_eq!(next_choice(choices, "unknown"), "start");
        assert!(choices_for("width").is_none());
    }

    #[test]
    fn editors_follow_the_kind() {
        let bool_p = Property::new("hidden", PropertyValue::Bool(false));
        let text_p = Property::new("tooltip", PropertyValue::Text(String::new()));
        let align = Property::new("align_vertical", PropertyValue::Text("fill".into()));

        assert_eq!(editor_for(&bool_p), EditorKind::Check);
        assert_eq!(editor_for(&text_p), EditorKind::Field(PropertyKind::Text));
        assert_eq!(editor_for(&align), EditorKind::Cycle);
    }
}
