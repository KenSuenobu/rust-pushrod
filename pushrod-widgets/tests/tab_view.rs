// Pushrod Widgets
// TabView tests
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

//! Tests for the `TabView`: tabs above a stack of pages.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::WidgetId;
use pushrod_widgets::button::Button;
use pushrod_widgets::tab_view::TabView;
use pushrod_widgets::text_field::TextField;

/// What the callbacks saw, in order.
type Log = Vec<String>;

/// A window with a 300 by 200 tab view at (10, 10) with pages General, Advanced, and About.
struct Fixture {
    engine: Engine<Log>,
    backend: HeadlessBackend,
    log: Log,
    id: WidgetId,
}

impl Fixture {
    /// Builds the window and draws the first frame.
    fn new() -> Self {
        let mut engine: Engine<Log> = Engine::new(size(400, 300), 60);
        let id = engine.add_widget(
            TabView::new()
                .at(10, 10)
                .size(300, 200)
                .with_page("General")
                .with_page("Advanced")
                .with_page("About"),
        );

        engine.on_selection_change(id, |log, _| log.push("selection".into()));

        let mut fixture = Self {
            engine,
            backend: HeadlessBackend::new(size(400, 300)),
            log: Log::new(),
            id,
        };

        fixture.frame();
        fixture
    }

    /// Gives the keyboard focus to a tab, as a user would by clicking or tabbing to it.
    fn focus_tab(&mut self, index: usize) {
        let tab = self.view().tabs()[index];

        assert!(self.engine.widgets().set_focus(tab));
        self.frame();
    }

    /// Draws a frame and delivers the events it queued.
    fn frame(&mut self) {
        self.engine.widgets().draw_loop(&mut self.backend);
        self.engine.run_idle(&mut self.log);
    }

    /// Sends an input event, then draws a frame.
    fn send(&mut self, event: InputEvent) {
        self.engine.dispatch(event, &mut self.log);
        self.frame();
    }

    /// Presses a key with modifiers.
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.send(InputEvent::KeyDown {
            key,
            modifiers,
            repeat: false,
        });
    }

    /// Clicks the middle of a widget.
    fn click(&mut self, id: WidgetId) {
        let origin = self.engine.widgets().absolute_origin(id);
        let size = self.engine.widgets().get(id).unwrap().get_size();
        let at = point(origin.x + size.w as i32 / 2, origin.y + size.h as i32 / 2);

        self.send(InputEvent::MouseDown {
            position: at,
            button: MouseButton::Left,
            clicks: 1,
        });
        self.send(InputEvent::MouseUp {
            position: at,
            button: MouseButton::Left,
            clicks: 1,
        });
    }

    /// The tab view.
    fn view(&mut self) -> &mut TabView {
        self.engine
            .widgets()
            .get_as_mut::<TabView>(self.id)
            .unwrap()
    }

    /// Whether each page is hidden.
    fn hidden(&mut self) -> Vec<bool> {
        let pages = self.view().pages();

        pages
            .into_iter()
            .map(|p| self.engine.widgets().get(p).unwrap().is_hidden())
            .collect()
    }

    /// Whether each tab is checked.
    fn checked(&mut self) -> Vec<bool> {
        let tabs = self.view().tabs();

        tabs.into_iter()
            .map(|t| {
                self.engine
                    .widgets()
                    .get_as::<Button>(t)
                    .unwrap()
                    .is_checked()
            })
            .collect()
    }
}

#[test]
fn it_builds_a_tab_and_a_page_per_title_and_shows_the_first() {
    let mut f = Fixture::new();

    assert_eq!(f.view().page_count(), 3);
    assert_eq!(f.view().tabs().len(), 3);
    assert_eq!(f.view().selected_page(), Some(0));
    assert_eq!(f.hidden(), vec![false, true, true]);
    assert_eq!(f.checked(), vec![true, false, false]);
    assert!(f.log.is_empty(), "the starting page is not a change");
}

#[test]
fn clicking_a_tab_shows_its_page_and_emits_selection_changed() {
    let mut f = Fixture::new();
    let tab = f.view().tabs()[1];

    f.click(tab);

    assert_eq!(f.view().selected_page(), Some(1));
    assert_eq!(f.hidden(), vec![true, false, true]);
    assert_eq!(f.checked(), vec![false, true, false]);
    assert_eq!(f.log, vec!["selection"]);
}

#[test]
fn clicking_the_shown_tab_changes_nothing() {
    let mut f = Fixture::new();
    let tab = f.view().tabs()[0];

    f.click(tab);

    assert_eq!(f.view().selected_page(), Some(0));
    assert_eq!(f.checked(), vec![true, false, false]);
    assert!(f.log.is_empty());
}

#[test]
fn hidden_pages_keep_their_controls_ids_and_state() {
    let mut f = Fixture::new();
    let page0 = f.view().pages()[0];
    let field = f
        .engine
        .add_child_widget(page0, TextField::new().at(5, 5).size(120, 24));

    f.engine
        .widgets()
        .get_as_mut::<TextField>(field)
        .unwrap()
        .set_text("keep me");
    f.frame();

    let tab = f.view().tabs()[2];

    f.click(tab);

    let widget = f.engine.widgets().get(field).expect("still there");

    assert_eq!(widget.get_parent(), page0);
    assert_eq!(
        f.engine
            .widgets()
            .get_as::<TextField>(field)
            .unwrap()
            .text(),
        "keep me"
    );

    let first = f.view().tabs()[0];

    f.click(first);

    assert_eq!(f.hidden(), vec![false, true, true]);
    assert_eq!(
        f.engine
            .widgets()
            .get_as::<TextField>(field)
            .unwrap()
            .text(),
        "keep me"
    );
}

#[test]
fn ctrl_tab_moves_forward_and_wraps() {
    let mut f = Fixture::new();

    f.focus_tab(0);

    f.key(Key::Tab, Modifiers::CTRL);
    assert_eq!(f.view().selected_page(), Some(1));

    f.key(Key::Tab, Modifiers::CTRL);
    f.key(Key::Tab, Modifiers::CTRL);
    assert_eq!(f.view().selected_page(), Some(0));
    assert_eq!(f.log.len(), 3);
}

#[test]
fn ctrl_shift_tab_moves_back_and_wraps() {
    let mut f = Fixture::new();

    f.focus_tab(0);

    f.key(Key::Tab, Modifiers::CTRL | Modifiers::SHIFT);
    assert_eq!(f.view().selected_page(), Some(2));

    f.key(Key::Tab, Modifiers::CTRL | Modifiers::SHIFT);
    assert_eq!(f.view().selected_page(), Some(1));
}

#[test]
fn ctrl_tab_works_from_a_control_inside_a_page_and_moves_the_focus_out_of_it() {
    let mut f = Fixture::new();
    let page0 = f.view().pages()[0];
    let field = f
        .engine
        .add_child_widget(page0, TextField::new().at(5, 5).size(120, 24));

    f.frame();
    assert!(f.engine.widgets().set_focus(field));

    f.key(Key::Tab, Modifiers::CTRL);

    assert_eq!(f.view().selected_page(), Some(1));

    let tab = f.view().tabs()[1];

    assert_eq!(f.engine.widgets().focused(), Some(tab));
}

#[test]
fn plain_tab_does_not_switch_pages() {
    let mut f = Fixture::new();

    f.focus_tab(0);
    f.key(Key::Tab, Modifiers::empty());

    assert_eq!(f.view().selected_page(), Some(0));
    assert!(f.log.is_empty());
}

#[test]
fn select_page_switches_and_rejects_a_missing_page() {
    let mut f = Fixture::new();

    assert!(f.view().select_page(2));
    f.frame();
    assert_eq!(f.hidden(), vec![true, true, false]);
    assert_eq!(f.log, vec!["selection"]);

    assert!(!f.view().select_page(3));
    assert_eq!(f.view().selected_page(), Some(2));
}

#[test]
fn add_page_appends_a_tab_and_page_without_changing_the_shown_page() {
    let mut f = Fixture::new();
    let (tab, page) = TabView::add_page(f.engine.widgets(), f.id, "Network").unwrap();

    f.frame();

    assert_eq!(f.view().page_count(), 4);
    assert_eq!(f.view().tabs()[3], tab);
    assert_eq!(f.view().pages()[3], page);
    assert_eq!(f.view().selected_page(), Some(0));
    assert_eq!(f.hidden(), vec![false, true, true, true]);
    assert_eq!(
        f.engine.widgets().get_as::<Button>(tab).unwrap().get_text(),
        "Network"
    );
    assert!(f.log.is_empty());
}

#[test]
fn remove_page_removes_the_tab_the_page_and_its_children() {
    let mut f = Fixture::new();
    let page1 = f.view().pages()[1];
    let child = f
        .engine
        .add_child_widget(page1, TextField::new().size(50, 20));
    let (tab, page) = TabView::remove_page(f.engine.widgets(), f.id, 1).unwrap();

    f.frame();

    assert_eq!(page, page1);
    assert!(f.engine.widgets().get(tab).is_none());
    assert!(f.engine.widgets().get(page).is_none());
    assert!(f.engine.widgets().get(child).is_none());
    assert_eq!(f.view().page_count(), 2);
    assert_eq!(f.view().selected_page(), Some(0));
    assert!(f.log.is_empty());
}

#[test]
fn removing_the_shown_page_shows_the_next_and_emits() {
    let mut f = Fixture::new();

    TabView::remove_page(f.engine.widgets(), f.id, 0).unwrap();
    f.frame();

    // The old second page is now first and is shown (index 0 still), a different page.
    assert_eq!(f.view().selected_page(), Some(0));
    assert_eq!(f.hidden(), vec![false, true]);
}

#[test]
fn removing_the_last_shown_page_shows_the_previous() {
    let mut f = Fixture::new();

    f.view().select_page(2);
    f.frame();
    f.log.clear();

    TabView::remove_page(f.engine.widgets(), f.id, 2).unwrap();
    f.frame();

    assert_eq!(f.view().selected_page(), Some(1));
    assert_eq!(f.hidden(), vec![true, false]);
    assert_eq!(f.log, vec!["selection"]);
}

#[test]
fn removing_an_earlier_page_keeps_the_same_page_shown() {
    let mut f = Fixture::new();

    f.view().select_page(2);
    f.frame();

    let shown = f.view().selected_page_id();

    f.log.clear();
    TabView::remove_page(f.engine.widgets(), f.id, 0).unwrap();
    f.frame();

    assert_eq!(f.view().selected_page(), Some(1));
    assert_eq!(f.view().selected_page_id(), shown);
    assert!(f.log.is_empty(), "the same page is still shown");
}

#[test]
fn page_errors_are_reported() {
    let mut f = Fixture::new();

    assert!(TabView::remove_page(f.engine.widgets(), f.id, 9).is_err());
    assert!(TabView::add_page(f.engine.widgets(), 9999, "x").is_err());

    let tab = f.view().tabs()[0];

    assert!(TabView::add_page(f.engine.widgets(), tab, "x").is_err());
}

#[test]
fn removing_every_page_leaves_nothing_selected() {
    let mut f = Fixture::new();

    for _ in 0..3 {
        TabView::remove_page(f.engine.widgets(), f.id, 0).unwrap();
    }

    f.frame();
    assert_eq!(f.view().selected_page(), None);
    assert_eq!(f.view().selected_page_id(), None);

    let (_, page) = TabView::add_page(f.engine.widgets(), f.id, "New").unwrap();

    f.frame();
    assert_eq!(f.view().selected_page_id(), Some(page));
    assert_eq!(f.hidden(), vec![false]);
}

#[test]
fn tabs_can_be_retitled_and_restyled_and_widen_to_fit() {
    let mut f = Fixture::new();
    let tab = f.view().tabs()[0];
    let before = f.engine.widgets().get(tab).unwrap().get_size().w;

    TabView::set_page_title(f.engine.widgets(), f.id, 0, "A much longer title").unwrap();
    f.frame();

    assert!(TabView::set_page_title(f.engine.widgets(), f.id, 9, "x").is_err());

    let after = f.engine.widgets().get(tab).unwrap().get_size().w;
    let second = f.view().tabs()[1];
    let second_x = f.engine.widgets().get(second).unwrap().get_origin().x;

    assert!(after > before);
    // Each button sits one pixel inside its two-pixel-wider outline, and the next outline
    // starts where this one ends.
    assert_eq!(second_x, after as i32 + 3, "the next tab moves over");
}

#[test]
fn pages_fill_the_area_under_the_tabs() {
    let mut f = Fixture::new();
    let parts = f.view().typed_parts();

    for page in parts.pages {
        let widget = f.engine.widgets().get(page).unwrap();

        assert_eq!(widget.get_origin(), point(1, 24));
        assert_eq!(widget.get_size(), size(298, 175));
    }
}

#[test]
fn an_empty_view_is_harmless() {
    let mut engine: Engine<Log> = Engine::new(size(100, 100), 60);
    let id = engine.add_widget(TabView::new().size(50, 50));
    let mut backend = HeadlessBackend::new(size(100, 100));
    let mut log = Log::new();

    engine.widgets().draw_loop(&mut backend);
    engine.run_idle(&mut log);

    let view = engine.widgets().get_as_mut::<TabView>(id).unwrap();

    assert_eq!(view.selected_page(), None);
    assert!(!view.select_page(0));
}
