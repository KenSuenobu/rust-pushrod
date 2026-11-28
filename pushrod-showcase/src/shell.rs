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

//! The showcase window: a search field and page list on the left, the selected page on the right.

use crate::event_log::{self, EventLog};
use crate::page::Group;
use crate::pages;
use crate::property_panel::{self, PickRequest, PropertyPanel};
use crate::registry::Registry;
use crate::settings;
use crate::source_view::{self, CopyState};
use pushrod::cache::WidgetCache;
use pushrod::color::{Color, FontStyle};
use pushrod::composite::TypedParts;
use pushrod::engine::{Engine, Ui};
use pushrod::event::Event;
use pushrod::font::Font;
use pushrod::geometry::{Rect, Size};
use pushrod::geometry::{point, size};
use pushrod::layout::{
    Align, Alignment, ChildInfo, Column, Layout, LayoutParams, Margin, Row, SizeHint,
};
use pushrod::widget::{ROOT_WIDGET_ID, Widget, WidgetId};
use pushrod_widgets::button::Button;
use pushrod_widgets::container::{Container, ContentSize};
use pushrod_widgets::custom_paint::CustomPaint;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;
use std::path::PathBuf;

/// The window's starting size, in pixels.
pub const WINDOW_W: u32 = 1180;
pub const WINDOW_H: u32 = 640;

/// The width of the page list, in pixels.
const LIST_W: u32 = 240;

/// The callback handle type pages are built with.
pub type ShowcaseUi<'a> = Ui<'a, ShowcaseState>;

/// The `stretch` value that marks a child of the window as floating: a dialog that places itself
/// and must stay out of the window's row.
pub const FLOATING: u32 = u32::MAX;

/// The window's layout: a row for the shell's columns, and no opinion about floating children.
///
/// A modal alert is added to the window and centers itself.  Left to a plain [`Row`] it would
/// join the row, so the alert is marked with [`FLOATING`] and this layout leaves it alone.
#[derive(Debug)]
struct WindowLayout {
    /// The row that places everything else.
    row: Row,
}

impl WindowLayout {
    /// Splits out the children the row places.
    ///
    /// # Arguments
    ///
    /// * `children` - Every visible child.
    ///
    /// # Returns
    ///
    /// The children that are not floating.
    fn placed(children: &[ChildInfo]) -> Vec<ChildInfo> {
        children
            .iter()
            .copied()
            .filter(|child| child.params.stretch != FLOATING)
            .collect()
    }
}

impl Layout for WindowLayout {
    /// Measures the children the row places.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children.
    /// * `available` - The window's size.
    ///
    /// # Returns
    ///
    /// The row's size hint.
    fn measure(&self, children: &[ChildInfo], available: Size) -> SizeHint {
        self.row.measure(&Self::placed(children), available)
    }

    /// Places the children the row places; a floating child keeps its position.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children, floating ones last (they are added last).
    /// * `bounds` - The window's area.
    ///
    /// # Returns
    ///
    /// One rectangle per placed child; floating children get none.
    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
        self.row.arrange(&Self::placed(children), bounds)
    }
}

/// The IDs of the shell's own widgets.
#[derive(Debug, Copy, Clone)]
pub struct Shell {
    /// The search field.
    pub search: WidgetId,
    /// The content of the page list's scrolling container.
    pub list: WidgetId,
    /// The selected page's title.
    pub title: WidgetId,
    /// The selected page's one-line description.
    pub description: WidgetId,
    /// The **Source** toggle that swaps the page for its code.
    pub source: WidgetId,
    /// The **Copy** button, shown while the code is.
    pub copy: WidgetId,
    /// The content of the page area's scrolling container.
    pub page_area: WidgetId,
}

/// The showcase's application state.
pub struct ShowcaseState {
    /// Every page.
    pub registry: Registry,
    /// The shell's widgets.
    pub shell: Shell,
    /// The page buttons now in the list, with the page index each one opens.
    pub buttons: Vec<(WidgetId, usize)>,
    /// The index of the selected page.
    pub selected: Option<usize>,
    /// The root of the selected page's widgets.
    pub page_root: Option<WidgetId>,
    /// Where the last page is remembered, if anywhere.
    pub settings_path: Option<PathBuf>,
    /// The name of the page to open when the window first runs.
    pub start_page: Option<String>,
    /// Whether the first page has been opened.
    pub started: bool,
    /// The property panel and its selection.
    pub panel: PropertyPanel,
    /// The event log and its entries.
    pub log: EventLog,
    /// Whether the code of the page is shown instead of the page.
    pub showing_source: bool,
}

/// Layout parameters that fill the slot in both directions.
///
/// # Arguments
///
/// * `stretch` - The share of the extra space.
///
/// # Returns
///
/// The parameters.
fn fill(stretch: u32) -> LayoutParams {
    LayoutParams {
        stretch,
        alignment: Alignment::new(Align::Fill, Align::Fill),
        ..LayoutParams::default()
    }
}

/// Layout parameters that fill the width and keep the height.
///
/// # Returns
///
/// The parameters.
pub(crate) fn fill_width() -> LayoutParams {
    LayoutParams {
        alignment: Alignment::new(Align::Fill, Align::Start),
        ..LayoutParams::default()
    }
}

/// Adds a panel with a layout and layout parameters.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `parent` - The panel's parent.
/// * `layout` - How the panel lays out its children.
/// * `params` - How the panel sits in its parent.
/// * `width` - The preferred width.
///
/// # Returns
///
/// The panel's ID.
pub(crate) fn add_panel(
    cache: &mut WidgetCache,
    parent: WidgetId,
    layout: Box<dyn pushrod::layout::Layout>,
    params: LayoutParams,
    width: u32,
) -> WidgetId {
    let id = cache.add_child(parent, Box::new(Panel::new().size(width, 100)));

    if let Some(panel) = cache.get_mut(id) {
        panel.set_layout(layout);
        panel.set_layout_params(params);
    }

    id
}

/// Adds a scrolling container and returns its content widget.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `parent` - The container's parent.
/// * `mode` - How the content is sized.
///
/// # Returns
///
/// The ID of the container's content.
pub(crate) fn add_scroller(
    cache: &mut WidgetCache,
    parent: WidgetId,
    mode: ContentSize,
) -> WidgetId {
    let container = cache.add_child(
        parent,
        Box::new(Container::new(point(0, 0), size(100, 100), size(0, 0)).content_mode(mode)),
    );

    if let Some(widget) = cache.get_mut(container) {
        widget.set_layout_params(fill(1));
    }

    cache
        .get_as::<Container>(container)
        .map_or(container, |c| c.typed_parts().content)
}

/// Builds the shell's widgets and the state that goes with them.
///
/// The first page opens on the first idle pass of the event loop, because pages are built
/// through the callback handle.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `settings_path` - Where the last page is remembered; `None` to not remember it.
///
/// # Returns
///
/// The application state to run the engine with.
pub fn build(engine: &mut Engine<ShowcaseState>, settings_path: Option<PathBuf>) -> ShowcaseState {
    let cache = engine.widgets();

    if let Some(root) = cache.get_mut(ROOT_WIDGET_ID) {
        root.set_layout(Box::new(WindowLayout {
            row: Row::new().spacing(0),
        }));
    }

    // Left: the search field above the page list.
    let left = add_panel(
        cache,
        ROOT_WIDGET_ID,
        Box::new(Column::new().spacing(8).margins(Margin::all(8))),
        LayoutParams {
            alignment: Alignment::new(Align::Start, Align::Fill),
            ..LayoutParams::default()
        },
        LIST_W,
    );
    let search = cache.add_child(
        left,
        Box::new(TextField::new().size(LIST_W - 16, 24).with_label("Search")),
    );

    if let Some(field) = cache.get_mut(search) {
        field.set_layout_params(fill_width());
    }

    let list = add_scroller(cache, left, ContentSize::FollowWidth { fit_height: true });

    if let Some(content) = cache.get_mut(list) {
        content.set_layout(Box::new(Column::new().spacing(4).margins(Margin::all(4))));
    }

    // Right: the title and description above the scrolling page.
    let right = add_panel(
        cache,
        ROOT_WIDGET_ID,
        Box::new(Column::new().spacing(6).margins(Margin::all(12))),
        fill(1),
        400,
    );
    let heading = Font::new("", 20, FontStyle::BOLD);
    let head = add_panel(
        cache,
        right,
        Box::new(Row::new().spacing(8)),
        fill_width(),
        400,
    );

    if let Some(panel) = cache.get_mut(head) {
        panel.set_size(size(400, 28));
    }

    let title = cache.add_child(head, Box::new(Text::new("").size(400, 28).font(heading)));
    let copy = cache.add_child(head, Box::new(source_view::copy_button()));
    let source = cache.add_child(head, Box::new(Button::new("Source").size(72, 24).toggle()));

    if let Some(widget) = cache.get_mut(copy) {
        widget.set_hidden(true);
    }

    for (id, stretch) in [(title, 1), (copy, 0), (source, 0)] {
        if let Some(widget) = cache.get_mut(id) {
            widget.set_layout_params(LayoutParams {
                stretch,
                alignment: Alignment::new(Align::Fill, Align::Center),
                ..LayoutParams::default()
            });
        }
    }

    let description = cache.add_child(right, Box::new(Text::new("").size(400, 20)));

    for id in [title, description] {
        if let Some(widget) = cache.get_mut(id) {
            widget.set_layout_params(fill_width());
        }
    }

    let page_area = add_scroller(cache, right, ContentSize::FitChildren);

    // Bottom of the right column: the event log, under the page.
    let log = event_log::build(cache, right, 400);

    // Far right: the property panel, as in the roadmap mockup.
    let props = add_panel(
        cache,
        ROOT_WIDGET_ID,
        Box::new(Column::new().spacing(8).margins(Margin::all(12))),
        LayoutParams {
            alignment: Alignment::new(Align::Start, Align::Fill),
            ..LayoutParams::default()
        },
        property_panel::PANEL_W + 24,
    );
    let panel = property_panel::build(cache, props);
    let shell = Shell {
        search,
        list,
        title,
        description,
        source,
        copy,
        page_area,
    };
    let mut registry = Registry::new();

    pages::register_all(&mut registry);

    let mut state = ShowcaseState {
        registry,
        shell,
        buttons: Vec::new(),
        selected: None,
        page_root: None,
        start_page: settings_path.as_deref().and_then(settings::load_last_page),
        settings_path,
        started: false,
        panel,
        log,
        showing_source: false,
    };

    // The resource file and its form: loaded once, borrowed by the Forms page.
    let _ = crate::resources::preload(engine);

    // The resource file and its form: loaded once, borrowed by the Forms page.
    let _ = crate::resources::preload(engine);

    rebuild_list(engine.widgets(), &mut state, "");

    engine.on_idle(|state, ui| {
        if !state.started {
            state.started = true;

            let start = state
                .start_page
                .as_deref()
                .and_then(|name| state.registry.find(name))
                .or_else(|| (!state.registry.is_empty()).then_some(0));

            if let Some(index) = start {
                select_page(state, ui, index);
            }
        }

        event_log::follow(state, ui);
    });

    engine.on_engine_event_filtered(&event_log::ENGINE_KINDS, |state, ui, event| {
        event_log::handle_engine(state, ui, event);
    });

    engine.on_custom::<PickRequest, _>(|state, ui, _source, request| {
        property_panel::pick(state, ui, request.id);
    });

    engine.on_event(|state, ui, event| {
        property_panel::handle(state, ui, event);
        event_log::handle(state, ui, event);

        if let (Some(root), Some(index)) = (state.page_root, state.selected)
            && let Some(page) = state.registry.get(index)
        {
            page.handle_event(ui, root, event);
        }

        if let Event::Toggled { id, checked } = event
            && *id == state.shell.source
        {
            state.showing_source = *checked;

            if let Some(index) = state.selected {
                select_page(state, ui, index);
            }

            return;
        }

        match event {
            // Page buttons are toggles (the selected one stays pressed), so a click arrives as `Toggled`.
            Event::Clicked { id, .. } | Event::Toggled { id, .. } => {
                if let Some(&(_, index)) = state.buttons.iter().find(|(button, _)| button == id) {
                    select_page(state, ui, index);
                }
            }
            Event::TextChanged { id } if *id == state.shell.search => apply_filter(state, ui),
            _ => {}
        }
    });

    state
}

/// Replaces the page list with the pages that match a search.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `state` - The application state; its button list is rebuilt.
/// * `query` - The text typed in the search field.
pub fn rebuild_list(cache: &mut WidgetCache, state: &mut ShowcaseState, query: &str) {
    for child in cache.children_of(state.shell.list).to_vec() {
        let _ = cache.remove(child);
    }

    state.buttons.clear();

    let heading = Font::new("", 13, FontStyle::BOLD);
    let mut shown = 0;

    for group in Group::ALL {
        let matches = state.registry.matching(group, query);

        if matches.is_empty() {
            continue;
        }

        cache.add_child(
            state.shell.list,
            Box::new(
                Text::new(group.title())
                    .size(LIST_W - 24, 20)
                    .font(heading.clone()),
            ),
        );

        for index in matches {
            let name = state.registry.get(index).map_or("", |page| page.name());
            let mut button = Button::new(name).size(LIST_W - 24, 26).toggle();

            button.set_layout_params(fill_width());
            button.set_checked(state.selected == Some(index));

            let id = cache.add_child(state.shell.list, Box::new(button));

            state.buttons.push((id, index));
            shown += 1;
        }
    }

    if shown == 0 {
        let mut note = Text::new("No pages match.").size(LIST_W - 24, 20);

        note.set_font_color(Color::GRAY);
        cache.add_child(state.shell.list, Box::new(note));
    }
}

/// Filters the page list by the text in the search field.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
pub fn apply_filter(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    let query = ui
        .get::<TextField>(state.shell.search)
        .map(|field| field.text())
        .unwrap_or_default();

    rebuild_list(ui.widgets(), state, &query);
}

/// Shows a page: removes the old page, builds the new one, and remembers its name.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `index` - The page's index in the registry.
pub fn select_page(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, index: usize) {
    if state.registry.get(index).is_none() {
        return;
    }

    property_panel::clear(state, ui);

    let Some(page) = state.registry.get(index) else {
        return;
    };

    if let Some(old) = state.page_root.take() {
        if let Some(previous) = state.selected.and_then(|i| state.registry.get(i)) {
            previous.leave(ui, old);
        }

        let _ = ui.widgets().remove(old);
    }

    let code = page.source();
    let root = if state.showing_source {
        source_view::add_code_view(ui.widgets(), state.shell.page_area, code)
    } else {
        let root = page.build(ui);

        // The page is built at the top of the tree; move it into the scrolling area.
        let _ = ui.widgets().reparent(root, state.shell.page_area);
        root
    };
    state.page_root = Some(root);
    state.selected = Some(index);

    if let Some(text) = ui.get_mut::<Text>(state.shell.title) {
        text.set_text(page.name().to_string());
    }

    if let Some(text) = ui.get_mut::<Text>(state.shell.description) {
        text.set_text(page.description().to_string());
    }

    for (id, button_index) in &state.buttons {
        if let Some(button) = ui.get_mut::<Button>(*id) {
            button.set_checked(*button_index == index);
        }
    }

    if let Some(copy) = ui.get_mut::<CustomPaint<CopyState>>(state.shell.copy) {
        copy.set_data(CopyState {
            text: code.to_string(),
            copied: false,
        });
        copy.set_hidden(!state.showing_source);
    }

    if let Some(path) = &state.settings_path {
        settings::save_last_page(path, page.name());
    }
}
