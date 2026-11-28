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

//! The event log: a panel under the page that lists, newest last, every event the page's
//! widgets emit, with the widget's ID and the event's payload.
//!
//! * **Engine events** (`FocusChanged`, `PropertyChanged`, ...) can be switched on with the
//!   *Engine events* check box.  Only the ones that follow from what you do on the page are
//!   offered: the ones the log itself causes (widgets added, layout changed, idle ran) would
//!   fill the log with its own lines.
//! * **Kind** cycles a filter: *All*, one kind of event, or *Engine*.
//! * **Pause** stops new events being recorded; they are dropped, not saved for later.
//! * **Clear** empties the log.
//! * **Click a line** to select the widget that emitted it: it is outlined on the page and its
//!   properties are listed in the property panel.
//!
//! The log keeps the last [`MAX_ENTRIES`] events.

use crate::property_panel::{self, in_subtree};
use crate::shell::{ShowcaseState, ShowcaseUi, add_panel, add_scroller, fill_width};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{EngineEvent, EngineEventKind, Event};
use pushrod::geometry::{point, size};
use pushrod::layout::{Align, Alignment, Column, LayoutParams, Margin, Row};
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::container::{Container, ContentSize};
use pushrod_widgets::text::Text;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// The most events the log keeps; the oldest is dropped to make room.
pub const MAX_ENTRIES: usize = 200;
/// The height of the whole log panel, in pixels.
pub const LOG_H: u32 = 190;
/// The height of one line of the log, in pixels.
const LINE_H: u32 = 22;
/// The name of the filter that shows every event.
pub const FILTER_ALL: &str = "All";
/// The name of the filter that shows only engine events.
pub const FILTER_ENGINE: &str = "Engine";

/// The filters the **Kind** button cycles through: everything, each kind of widget event the
/// log records, and the engine events.
pub const FILTERS: [&str; 15] = [
    FILTER_ALL,
    "Clicked",
    "Activated",
    "Toggled",
    "ValueChanged",
    "SelectionChanged",
    "TextChanged",
    "EditingFinished",
    "FocusGained",
    "FocusLost",
    "KeyPressed",
    "TextEntered",
    "Triggered",
    "PropertyChanged",
    FILTER_ENGINE,
];

/// The engine events the log can show.  The rest (widget added or removed, layout, idle, ...)
/// are caused by the log's own lines, so showing them would never end.
pub const ENGINE_KINDS: [EngineEventKind; 10] = [
    EngineEventKind::FocusChanged,
    EngineEventKind::PropertyChanged,
    EngineEventKind::ActionTriggered,
    EngineEventKind::MnemonicActivated,
    EngineEventKind::CompositionStarted,
    EngineEventKind::CompositionUpdated,
    EngineEventKind::CompositionEnded,
    EngineEventKind::ModalOpened,
    EngineEventKind::ModalClosed,
    EngineEventKind::Error,
];

/// The names of the properties whose value is added to a `ValueChanged`, `TextChanged`, or
/// `SelectionChanged` line, when the widget has one.
const VALUE_PROPERTIES: [&str; 3] = ["value", "text", "selected"];

/// One recorded event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The event's number, counting up from 1.
    pub seq: u64,
    /// The time it was recorded, `hh:mm:ss` UTC.
    pub time: String,
    /// The kind of event, such as `Toggled`.
    pub kind: String,
    /// The widget that emitted it, if it names one.
    pub source: Option<WidgetId>,
    /// The event's data, such as `checked: true`; empty if it has none.
    pub payload: String,
    /// Whether it is an engine event.
    pub engine: bool,
}

impl Entry {
    /// Returns the line of text shown for the entry: time, kind, widget, and payload.
    ///
    /// # Returns
    ///
    /// The text, such as `12:01:07  Toggled  widget 14  checked: true`.
    pub fn text(&self) -> String {
        let source = self
            .source
            .map_or_else(|| "-".to_string(), |id| format!("widget {id}"));

        if self.payload.is_empty() {
            format!("{}  {}  {}", self.time, self.kind, source)
        } else {
            format!("{}  {}  {}  {}", self.time, self.kind, source, self.payload)
        }
    }

    /// Returns whether the entry passes a filter.
    ///
    /// # Arguments
    ///
    /// * `filter` - One of [`FILTERS`].
    ///
    /// # Returns
    ///
    /// `true` if it is shown.
    pub fn matches(&self, filter: &str) -> bool {
        match filter {
            FILTER_ALL => true,
            FILTER_ENGINE => self.engine,
            kind => !self.engine && self.kind == kind,
        }
    }
}

/// An event described for the log, before it is numbered and timed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Described {
    /// The kind of event.
    pub kind: String,
    /// The widgets the event names, first the one that emitted it.
    pub ids: Vec<WidgetId>,
    /// The event's data.
    pub payload: String,
}

/// The log's widgets and its entries.
#[derive(Debug, Default)]
pub struct EventLog {
    /// The **Kind** button.
    pub kind: WidgetId,
    /// The *Engine events* check box.
    pub engine: WidgetId,
    /// The **Pause** check box.
    pub pause: WidgetId,
    /// The **Clear** button.
    pub clear: WidgetId,
    /// The text that counts the events.
    pub count: WidgetId,
    /// The content of the scrolling list of lines.
    pub list: WidgetId,
    /// The recorded events, oldest first.
    pub entries: VecDeque<Entry>,
    /// The lines now in the list: each one's button, and the event number it shows.
    pub lines: Vec<(WidgetId, u64)>,
    /// Index into [`FILTERS`] of the filter now chosen.
    pub filter: usize,
    /// Whether recording is paused.
    pub paused: bool,
    /// Whether engine events are recorded.
    pub engine_on: bool,
    /// The number the next event gets.
    pub next_seq: u64,
    /// Whether the list should scroll to its newest line on the next idle pass.
    pub follow: bool,
}

impl EventLog {
    /// Returns the name of the filter now chosen.
    ///
    /// # Returns
    ///
    /// One of [`FILTERS`].
    pub fn filter_name(&self) -> &'static str {
        FILTERS[self.filter % FILTERS.len()]
    }
}

/// Describes a widget event for the log.
///
/// # Arguments
///
/// * `event` - The event.
///
/// # Returns
///
/// The description, or `None` for the events the log does not record: pointer movement, key
/// releases, lifecycle and window events, ticks, and engine and custom events.
pub fn describe(event: &Event) -> Option<Described> {
    let (id, payload) = match event {
        Event::Clicked { id, button, count } => {
            (*id, format!("button: {button:?}, count: {count}"))
        }
        Event::Activated { id }
        | Event::ValueChanged { id }
        | Event::SelectionChanged { id }
        | Event::TextChanged { id }
        | Event::EditingFinished { id }
        | Event::FocusGained { id }
        | Event::FocusLost { id } => (*id, String::new()),
        Event::Toggled { id, checked } => (*id, format!("checked: {checked}")),
        Event::KeyPressed {
            id, key, repeat, ..
        } => (*id, format!("key: {key:?}, repeat: {repeat}")),
        Event::TextEntered { id, text } => (*id, format!("text: {text:?}")),
        Event::Triggered { id, action } => (*id, format!("action: {action}")),
        Event::PropertyChanged { id, name } => (*id, format!("name: {name}")),
        _ => return None,
    };

    Some(Described {
        kind: format!("{:?}", event.kind()),
        ids: vec![id],
        payload,
    })
}

/// Describes an engine event for the log.
///
/// # Arguments
///
/// * `event` - The engine event.
///
/// # Returns
///
/// The description, or `None` for an engine event not in [`ENGINE_KINDS`].
pub fn describe_engine(event: &EngineEvent) -> Option<Described> {
    if !ENGINE_KINDS.contains(&event.kind()) {
        return None;
    }

    let (ids, payload) = match event {
        EngineEvent::FocusChanged { old, new } => (
            new.iter().chain(old.iter()).copied().collect(),
            format!("old: {old:?}, new: {new:?}"),
        ),
        EngineEvent::PropertyChanged { id, name } => (vec![*id], format!("name: {name}")),
        EngineEvent::ActionTriggered { id, cancel } => (vec![*id], format!("cancel: {cancel}")),
        EngineEvent::MnemonicActivated { id, key } => (vec![*id], format!("key: {key:?}")),
        EngineEvent::CompositionStarted { id } => (vec![*id], String::new()),
        EngineEvent::CompositionUpdated { id, text, .. } => (vec![*id], format!("text: {text:?}")),
        EngineEvent::CompositionEnded { id, committed } => {
            (vec![*id], format!("committed: {committed}"))
        }
        EngineEvent::ModalOpened { root } => (vec![*root], String::new()),
        EngineEvent::ModalClosed { root, dismissed_by } => {
            (vec![*root], format!("dismissed_by: {dismissed_by}"))
        }
        EngineEvent::Error { error } => (Vec::new(), format!("error: {error}")),
        _ => return None,
    };

    Some(Described {
        kind: format!("{:?}", event.kind()),
        ids,
        payload,
    })
}

/// Formats the time of day, UTC.
///
/// # Returns
///
/// `hh:mm:ss`.
fn clock() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600 % 24,
        secs / 60 % 60,
        secs % 60
    )
}

/// Builds the log's widgets, as in the roadmap mockup: a header with the title and the
/// controls, above the scrolling list of lines.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `parent` - The column the log goes at the bottom of.
/// * `width` - The preferred width.
///
/// # Returns
///
/// The log, with no entries.
pub fn build(cache: &mut WidgetCache, parent: WidgetId, width: u32) -> EventLog {
    let outer = add_panel(
        cache,
        parent,
        Box::new(Column::new().spacing(4).margins(Margin::all(0))),
        fill_width(),
        width,
    );

    if let Some(panel) = cache.get_mut(outer) {
        panel.set_size(size(width, LOG_H));
    }

    let header = add_panel(
        cache,
        outer,
        Box::new(Row::new().spacing(8)),
        fill_width(),
        width,
    );

    if let Some(panel) = cache.get_mut(header) {
        panel.set_size(size(width, 26));
    }

    let title = cache.add_child(header, Box::new(Text::new("Events").size(60, 22)));
    let mut count = Text::new("").size(90, 22);

    count.set_font_color(Color::GRAY);

    let count = cache.add_child(header, Box::new(count));

    // The title and count take the space the controls leave, so the controls sit on the right.
    for id in [title, count] {
        if let Some(widget) = cache.get_mut(id) {
            widget.set_layout_params(LayoutParams {
                stretch: u32::from(id == count),
                alignment: Alignment::new(Align::Fill, Align::Center),
                ..LayoutParams::default()
            });
        }
    }

    let kind = cache.add_child(
        header,
        Box::new(Button::new(format!("Kind: {FILTER_ALL}")).size(140, 24)),
    );
    let engine = cache.add_child(
        header,
        Box::new(CheckBox::new("Engine events").size(120, 24)),
    );
    let pause = cache.add_child(header, Box::new(CheckBox::new("Pause").size(64, 24)));
    let clear = cache.add_child(header, Box::new(Button::new("Clear").size(56, 24)));
    let list = add_scroller(cache, outer, ContentSize::FollowWidth { fit_height: true });

    if let Some(content) = cache.get_mut(list) {
        content.set_layout(Box::new(Column::new().spacing(2).margins(Margin::all(2))));
    }

    EventLog {
        kind,
        engine,
        pause,
        clear,
        count,
        list,
        next_seq: 1,
        ..EventLog::default()
    }
}

/// Updates the count text: how many events are shown, and how many are kept.
fn update_count(state: &ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    let log = &state.log;
    let shown = log.lines.len();
    let text = if shown == log.entries.len() {
        format!("{shown} events")
    } else {
        format!("{shown} of {} events", log.entries.len())
    };

    if let Some(widget) = ui.get_mut::<Text>(log.count) {
        widget.set_text(text);
    }
}

/// Adds the line for an entry at the end of the list.
fn add_line(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, entry: &Entry) {
    let mut line = Button::new(entry.text()).size(400, LINE_H);

    if entry.engine {
        line = line.font_color(Color::GRAY);
    }

    line.set_layout_params(fill_width());

    let id = ui.widgets().add_child(state.log.list, Box::new(line));

    state.log.lines.push((id, entry.seq));
    state.log.follow = true;
}

/// Removes every line and adds the ones that pass the filter again.
fn rebuild(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    for child in ui.widgets().children_of(state.log.list).to_vec() {
        let _ = ui.widgets().remove(child);
    }

    state.log.lines.clear();

    let filter = state.log.filter_name();
    let shown: Vec<Entry> = state
        .log
        .entries
        .iter()
        .filter(|entry| entry.matches(filter))
        .cloned()
        .collect();

    for entry in &shown {
        add_line(state, ui, entry);
    }

    update_count(state, ui);
}

/// Records an event: numbers it, keeps it (dropping the oldest past [`MAX_ENTRIES`]), and
/// shows it if it passes the filter.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `described` - The event.
/// * `engine` - Whether it is an engine event.
pub fn record(
    state: &mut ShowcaseState,
    ui: &mut ShowcaseUi<'_>,
    described: Described,
    engine: bool,
) {
    let source = described.ids.first().copied();
    let mut payload = described.payload;

    // `ValueChanged` and its kin carry no data, so show the widget's value after the change.
    if matches!(
        described.kind.as_str(),
        "ValueChanged" | "TextChanged" | "SelectionChanged"
    ) && let Some(id) = source
        && let Some(properties) = ui.widgets().get(id).map(|w| w.properties())
        && let Some(property) = properties
            .iter()
            .find(|p| VALUE_PROPERTIES.contains(&p.name.as_str()))
    {
        payload = format!(
            "{}: {}",
            property.name,
            property_panel::format_value(&property.value)
        );
    }

    let entry = Entry {
        seq: state.log.next_seq,
        time: clock(),
        kind: described.kind,
        source,
        payload,
        engine,
    };

    state.log.next_seq += 1;
    state.log.entries.push_back(entry.clone());

    if state.log.entries.len() > MAX_ENTRIES
        && let Some(old) = state.log.entries.pop_front()
        && let Some(at) = state.log.lines.iter().position(|(_, seq)| *seq == old.seq)
    {
        let (button, _) = state.log.lines.remove(at);

        let _ = ui.widgets().remove(button);
    }

    if entry.matches(state.log.filter_name()) {
        add_line(state, ui, &entry);
    }

    update_count(state, ui);
}

/// Returns whether an event is about the page: it names no widget, or a widget on the page.
fn on_page(state: &ShowcaseState, ui: &mut ShowcaseUi<'_>, ids: &[WidgetId]) -> bool {
    match state.page_root {
        Some(root) => ids.is_empty() || ids.iter().any(|id| in_subtree(ui, root, *id)),
        None => false,
    }
}

/// Handles an engine event: records it if engine events are on and it is about the page.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `event` - The engine event.
pub fn handle_engine(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, event: &EngineEvent) {
    if state.log.paused || !state.log.engine_on {
        return;
    }

    if let Some(described) = describe_engine(event)
        && on_page(state, ui, &described.ids)
    {
        record(state, ui, described, true);
    }
}

/// Handles a widget event: the log's own controls, and the recording of events from the page.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `event` - The event.
pub fn handle(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, event: &Event) {
    match event {
        Event::Toggled { id, checked } if *id == state.log.pause => state.log.paused = *checked,
        Event::Toggled { id, checked } if *id == state.log.engine => {
            state.log.engine_on = *checked;
        }
        Event::Clicked { id, .. } if *id == state.log.clear => {
            state.log.entries.clear();
            rebuild(state, ui);
        }
        Event::Clicked { id, .. } if *id == state.log.kind => {
            state.log.filter = (state.log.filter + 1) % FILTERS.len();

            let label = format!("Kind: {}", state.log.filter_name());

            if let Some(button) = ui.get_mut::<Button>(state.log.kind) {
                button.set_text(label);
            }

            rebuild(state, ui);
        }
        Event::Clicked { id, .. } if state.log.lines.iter().any(|(button, _)| button == id) => {
            let seq = state
                .log
                .lines
                .iter()
                .find(|(button, _)| button == id)
                .map(|(_, seq)| *seq);

            if let Some(seq) = seq {
                show_source(state, ui, seq);
            }
        }
        _ => {
            if !state.log.paused
                && let Some(described) = describe(event)
                && on_page(state, ui, &described.ids)
            {
                record(state, ui, described, false);
            }
        }
    }
}

/// Selects the widget that emitted an event, so it is outlined on the page.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
/// * `seq` - The event's number.
fn show_source(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>, seq: u64) {
    let source = state
        .log
        .entries
        .iter()
        .find(|entry| entry.seq == seq)
        .and_then(|entry| entry.source);

    if let (Some(id), Some(root)) = (source, state.page_root)
        && in_subtree(ui, root, id)
    {
        property_panel::select_widget(state, ui, id);
    }
}

/// Scrolls the list to its newest line, if lines were added since the last pass.  Call it on
/// the idle pass, after the new lines have been laid out.
///
/// # Arguments
///
/// * `state` - The application state.
/// * `ui` - The callback handle.
pub fn follow(state: &mut ShowcaseState, ui: &mut ShowcaseUi<'_>) {
    if !std::mem::take(&mut state.log.follow) {
        return;
    }

    let Some(frame) = ui.widgets().get(state.log.list).map(|w| w.get_parent()) else {
        return;
    };

    if let Some(container) = ui.get_mut::<Container>(frame) {
        container.scroll_to(point(0, i32::MAX));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pushrod::input::MouseButton;

    fn entry(kind: &str, engine: bool) -> Entry {
        Entry {
            seq: 1,
            time: "12:01:07".to_string(),
            kind: kind.to_string(),
            source: Some(14),
            payload: "checked: true".to_string(),
            engine,
        }
    }

    #[test]
    fn a_line_shows_time_kind_widget_and_payload() {
        assert_eq!(
            entry("Toggled", false).text(),
            "12:01:07  Toggled  widget 14  checked: true"
        );

        let bare = Entry {
            source: None,
            payload: String::new(),
            ..entry("Error", true)
        };

        assert_eq!(bare.text(), "12:01:07  Error  -");
    }

    #[test]
    fn filters_pick_kinds_and_engine_events() {
        let toggled = entry("Toggled", false);
        let focus = entry("FocusChanged", true);

        assert!(toggled.matches(FILTER_ALL) && focus.matches(FILTER_ALL));
        assert!(toggled.matches("Toggled") && !toggled.matches("Clicked"));
        assert!(!toggled.matches(FILTER_ENGINE) && focus.matches(FILTER_ENGINE));
        // An engine event never matches a widget-event filter of the same name.
        assert!(!entry("PropertyChanged", true).matches("PropertyChanged"));
    }

    #[test]
    fn widget_events_are_described_with_their_payload() {
        let clicked = describe(&Event::Clicked {
            id: 3,
            button: MouseButton::Left,
            count: 2,
        })
        .unwrap();

        assert_eq!(clicked.kind, "Clicked");
        assert_eq!(clicked.ids, vec![3]);
        assert_eq!(clicked.payload, "button: Left, count: 2");

        let toggled = describe(&Event::Toggled {
            id: 9,
            checked: true,
        })
        .unwrap();

        assert_eq!(toggled.payload, "checked: true");
    }

    #[test]
    fn noisy_events_are_not_recorded() {
        assert!(describe(&Event::PointerEntered { id: 1 }).is_none());
        assert!(
            describe(&Event::PointerMoved {
                id: 1,
                position: point(0, 0)
            })
            .is_none()
        );
        assert!(describe(&Event::CloseRequested).is_none());
    }

    #[test]
    fn only_engine_events_the_log_does_not_cause_are_described() {
        assert!(describe_engine(&EngineEvent::WidgetAdded { id: 2, parent: 1 }).is_none());
        assert!(describe_engine(&EngineEvent::LayoutChanged { container: 1 }).is_none());
        assert!(describe_engine(&EngineEvent::IdleRan { callbacks: 1 }).is_none());

        let focus = describe_engine(&EngineEvent::FocusChanged {
            old: Some(4),
            new: Some(5),
        })
        .unwrap();

        assert_eq!(focus.kind, "FocusChanged");
        assert_eq!(focus.ids, vec![5, 4]);
    }

    #[test]
    fn every_filter_name_is_distinct() {
        for (i, name) in FILTERS.iter().enumerate() {
            assert!(!FILTERS[i + 1..].contains(name), "{name} repeated");
        }
    }
}
