// Pushrod Widgets
// ListView tests
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

//! Tests for the `ListView`: a virtualized list of rows from a `ListModel`.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::color::Color;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::list_view::{
    ListModel, ListRow, ListView, ModelChange, RowStyle, SelectionMode, VecModel,
};
use std::time::Duration;

/// What the callbacks saw, in order.
type Log = Vec<String>;

/// A window with one 200 by 100 list (five 20-pixel rows) at (10, 10).
struct Fixture {
    engine: Engine<Log>,
    backend: HeadlessBackend,
    log: Log,
    id: WidgetId,
}

impl Fixture {
    /// Builds the window around a list and draws the first frame.
    fn new(list: ListView) -> Self {
        let mut engine: Engine<Log> = Engine::new(size(400, 400), 60);
        let id = engine.add_widget(list);

        engine.on_selection_change(id, |log, _| log.push("selection".into()));
        engine.on_activate(id, |log, _| log.push("activate".into()));
        engine.on_viewport_change(id, |log, _, o| log.push(format!("top {}", o.y)));

        let mut fixture = Self {
            engine,
            backend: HeadlessBackend::new(size(400, 400)),
            log: Log::new(),
            id,
        };

        fixture.frame();
        fixture.log.clear();
        fixture
    }

    /// A list of `rows` numbered rows.
    fn numbered(rows: usize, mode: SelectionMode) -> Self {
        let model = VecModel::new((0..rows).map(|n| format!("Row {n}")));

        Self::new(
            ListView::new(model)
                .at(10, 10)
                .size(200, 100)
                .selection_mode(mode),
        )
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

    /// Clicks the visible slot `slot` (0 is the top row) with modifiers held.
    fn click(&mut self, slot: i32, modifiers: Modifiers, count: u8) {
        let at = pushrod::geometry::point(30, 10 + slot * 20 + 5);

        self.send(InputEvent::ModifiersChanged(modifiers));
        self.send(InputEvent::MouseDown {
            position: at,
            button: MouseButton::Left,
            clicks: count,
        });
        self.send(InputEvent::MouseUp {
            position: at,
            button: MouseButton::Left,
            clicks: count,
        });
        self.send(InputEvent::ModifiersChanged(Modifiers::empty()));
    }

    /// Presses a key with modifiers.
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.send(InputEvent::KeyDown {
            key,
            modifiers,
            repeat: false,
        });
    }

    /// The list.
    fn list(&mut self) -> &mut ListView {
        self.engine
            .widgets()
            .get_as_mut::<ListView>(self.id)
            .unwrap()
    }

    /// The text and selection of each pooled row that is showing a model row.
    fn shown(&mut self) -> Vec<(usize, String, bool)> {
        let rows = self.list().typed_parts().rows;

        rows.into_iter()
            .filter_map(|id| {
                let row = self.engine.widgets().get_as::<ListRow>(id)?;

                Some((row.index()?, row.text().to_string(), row.is_selected()))
            })
            .collect()
    }

    /// The selected rows.
    fn selected(&mut self) -> Vec<usize> {
        self.list().selected_rows()
    }
}

fn shift() -> Modifiers {
    Modifiers::SHIFT
}

fn ctrl() -> Modifiers {
    Modifiers::CTRL
}

#[test]
fn only_the_visible_rows_exist_for_ten_thousand_rows() {
    let mut f = Fixture::numbered(10_000, SelectionMode::Single);
    let rows = f.list().typed_parts().rows;

    assert_eq!(rows.len(), 5);
    assert_eq!(f.list().row_count(), 10_000);
    assert_eq!(
        f.shown().iter().map(|r| r.1.clone()).collect::<Vec<_>>(),
        ["Row 0", "Row 1", "Row 2", "Row 3", "Row 4"]
    );
}

#[test]
fn scrolling_recycles_the_same_row_widgets() {
    let mut f = Fixture::numbered(10_000, SelectionMode::Single);
    let before = f.list().typed_parts().rows;

    f.send(InputEvent::MouseMove {
        position: pushrod::geometry::point(30, 30),
    });
    f.send(InputEvent::Wheel {
        delta: pushrod::geometry::point(0, -10),
    });

    assert_eq!(f.list().top_row(), 30);
    assert_eq!(f.list().typed_parts().rows, before);
    assert_eq!(f.shown()[0], (30, "Row 30".to_string(), false));
    assert_eq!(f.log, vec!["top 30"]);
}

#[test]
fn a_click_selects_and_emits_once() {
    let mut f = Fixture::numbered(20, SelectionMode::Single);

    f.click(2, Modifiers::empty(), 1);

    assert_eq!(f.selected(), vec![2]);
    assert_eq!(f.log, vec!["selection"]);
    assert_eq!(f.shown()[2], (2, "Row 2".to_string(), true));

    // Clicking the selected row again changes nothing.
    f.log.clear();
    f.click(2, Modifiers::empty(), 1);

    assert!(f.log.is_empty());
}

#[test]
fn a_double_click_activates() {
    let mut f = Fixture::numbered(20, SelectionMode::Single);

    f.click(1, Modifiers::empty(), 2);

    assert_eq!(f.selected(), vec![1]);
    assert_eq!(f.log, vec!["selection", "activate"]);
}

#[test]
fn single_mode_ignores_shift_and_ctrl() {
    let mut f = Fixture::numbered(20, SelectionMode::Single);

    f.click(0, Modifiers::empty(), 1);
    f.click(3, shift(), 1);
    assert_eq!(f.selected(), vec![3]);

    f.click(1, ctrl(), 1);
    assert_eq!(f.selected(), vec![1]);
}

#[test]
fn shift_click_extends_a_range_from_the_anchor() {
    let mut f = Fixture::numbered(20, SelectionMode::Range);

    f.click(1, Modifiers::empty(), 1);
    f.click(3, shift(), 1);
    assert_eq!(f.selected(), vec![1, 2, 3]);

    // A range follows the anchor, so shrinking back works.
    f.click(2, shift(), 1);
    assert_eq!(f.selected(), vec![1, 2]);

    // Ctrl does not make a second range in range mode.
    f.click(4, ctrl(), 1);
    assert_eq!(f.selected(), vec![4]);
}

#[test]
fn ctrl_click_toggles_in_multiple_mode() {
    let mut f = Fixture::numbered(20, SelectionMode::Multiple);

    f.click(0, Modifiers::empty(), 1);
    f.click(2, ctrl(), 1);
    f.click(4, ctrl(), 1);
    assert_eq!(f.selected(), vec![0, 2, 4]);

    f.click(2, ctrl(), 1);
    assert_eq!(f.selected(), vec![0, 4]);

    // Shift extends from the last row clicked (the anchor), row 2.
    f.click(3, shift(), 1);
    assert_eq!(f.selected(), vec![2, 3]);
}

#[test]
fn arrows_move_the_selection_and_scroll_it_into_view() {
    let mut f = Fixture::numbered(20, SelectionMode::Single);

    f.click(0, Modifiers::empty(), 1);
    f.log.clear();

    for _ in 0..6 {
        f.key(Key::Down, Modifiers::empty());
    }

    assert_eq!(f.selected(), vec![6]);
    assert_eq!(f.list().top_row(), 2);

    f.key(Key::Up, Modifiers::empty());
    assert_eq!(f.selected(), vec![5]);

    f.key(Key::Home, Modifiers::empty());
    assert_eq!((f.selected(), f.list().top_row()), (vec![0], 0));

    f.key(Key::End, Modifiers::empty());
    assert_eq!((f.selected(), f.list().top_row()), (vec![19], 15));

    f.key(Key::PageUp, Modifiers::empty());
    assert_eq!(f.selected(), vec![15]);

    f.key(Key::PageDown, Modifiers::empty());
    assert_eq!(f.selected(), vec![19]);
}

#[test]
fn shift_arrows_extend_and_ctrl_a_selects_all() {
    let mut f = Fixture::numbered(20, SelectionMode::Multiple);

    f.click(1, Modifiers::empty(), 1);
    f.key(Key::Down, shift());
    f.key(Key::Down, shift());
    assert_eq!(f.selected(), vec![1, 2, 3]);

    f.key(Key::Char('a'), ctrl());
    assert_eq!(f.selected().len(), 20);
}

#[test]
fn return_activates_the_cursor_row() {
    let mut f = Fixture::numbered(20, SelectionMode::Single);

    f.key(Key::Return, Modifiers::empty());
    assert!(f.log.is_empty(), "no row has the cursor yet");

    f.click(1, Modifiers::empty(), 1);
    f.log.clear();
    f.key(Key::Return, Modifiers::empty());

    assert_eq!(f.log, vec!["activate"]);
}

#[test]
fn type_ahead_jumps_to_matching_rows() {
    let model = VecModel::new(["Apple", "Apricot", "Banana", "Blueberry", "Cherry", "Date"]);
    let mut f = Fixture::new(ListView::new(model).at(10, 10).size(200, 100));

    f.click(0, Modifiers::empty(), 1);
    f.key(Key::Char('b'), Modifiers::empty());
    assert_eq!(f.selected(), vec![2]);

    f.key(Key::Char('l'), Modifiers::empty());
    assert_eq!(f.selected(), vec![3], "'bl' finds Blueberry");

    f.key(Key::Char('z'), Modifiers::empty());
    assert_eq!(f.selected(), vec![3], "no match keeps the selection");
}

#[test]
fn type_ahead_restarts_after_a_pause() {
    let model = VecModel::new(["Apple", "Banana", "Blueberry", "Cherry"]);
    let mut f = Fixture::new(
        ListView::new(model)
            .at(10, 10)
            .size(200, 100)
            .type_ahead_timeout(Duration::ZERO),
    );

    f.click(0, Modifiers::empty(), 1);
    f.key(Key::Char('b'), Modifiers::empty());
    assert_eq!(f.selected(), vec![1]);

    std::thread::sleep(Duration::from_millis(2));
    f.key(Key::Char('c'), Modifiers::empty());
    assert_eq!(f.selected(), vec![3], "a new search, not 'bc'");
}

#[test]
fn inserting_rows_shifts_the_selection_and_updates_the_view() {
    let mut f = Fixture::numbered(10, SelectionMode::Multiple);

    f.click(2, Modifiers::empty(), 1);
    f.log.clear();

    f.list().model_mut::<VecModel>().unwrap().insert(0, "New");
    f.frame();

    assert_eq!(f.selected(), vec![3]);
    assert_eq!(f.list().row_count(), 11);
    assert_eq!(f.shown()[0].1, "New");
    assert_eq!(f.shown()[3], (3, "Row 2".to_string(), true));
    assert_eq!(f.log, vec!["selection"]);
}

#[test]
fn removing_rows_drops_their_selection() {
    let mut f = Fixture::numbered(10, SelectionMode::Multiple);

    f.click(1, Modifiers::empty(), 1);
    f.click(3, ctrl(), 1);
    f.log.clear();

    f.list().model_mut::<VecModel>().unwrap().remove(0..2);
    f.frame();

    assert_eq!(
        f.selected(),
        vec![1],
        "row 1 went with the removal; row 3 is now row 1"
    );
    assert_eq!(f.shown()[0].1, "Row 2");
    assert_eq!(f.log, vec!["selection"]);
}

#[test]
fn changing_a_row_updates_its_text_without_touching_the_selection() {
    let mut f = Fixture::numbered(10, SelectionMode::Single);

    f.click(1, Modifiers::empty(), 1);
    f.log.clear();

    f.list().model_mut::<VecModel>().unwrap().set(1, "Edited");
    f.frame();

    assert_eq!(f.shown()[1], (1, "Edited".to_string(), true));
    assert!(f.log.is_empty());
}

#[test]
fn a_reset_clears_the_selection_and_scrolls_to_the_top() {
    let mut f = Fixture::numbered(50, SelectionMode::Single);

    f.click(0, Modifiers::empty(), 1);
    f.key(Key::End, Modifiers::empty());
    f.log.clear();

    f.list().model_mut::<VecModel>().unwrap().reset(["a", "b"]);
    f.frame();

    assert!(f.selected().is_empty());
    assert_eq!(f.list().top_row(), 0);
    assert_eq!(f.shown().len(), 2);
    assert!(f.log.contains(&"selection".to_string()));
}

#[test]
fn removing_rows_above_the_view_keeps_the_view_in_place() {
    let mut f = Fixture::numbered(50, SelectionMode::Single);

    f.list().scroll_to_row(20);
    f.frame();
    f.list().model_mut::<VecModel>().unwrap().remove(0..5);
    f.frame();

    assert_eq!(f.list().top_row(), 15);
    assert_eq!(f.shown()[0].1, "Row 20");
}

#[test]
fn the_scroll_bar_scrolls_the_list_only_when_rows_overflow() {
    let mut f = Fixture::numbered(3, SelectionMode::Single);
    let scroll = f.list().typed_parts().scroll;

    assert!(f.engine.widgets().get(scroll).unwrap().is_hidden());

    let mut f = Fixture::numbered(50, SelectionMode::Single);
    let scroll = f.list().typed_parts().scroll;

    assert!(!f.engine.widgets().get(scroll).unwrap().is_hidden());
}

#[test]
fn unused_pool_rows_are_hidden() {
    let mut f = Fixture::numbered(2, SelectionMode::Single);
    let rows = f.list().typed_parts().rows;
    let hidden: Vec<bool> = rows
        .iter()
        .map(|&id| f.engine.widgets().get(id).unwrap().is_hidden())
        .collect();

    assert_eq!(hidden, [false, false, true, true, true]);
}

#[test]
fn parts_can_be_retrieved_and_restyled() {
    let mut f = Fixture::numbered(10, SelectionMode::Single);
    let rows = f.list().typed_parts().rows;
    let first = rows[0];

    f.engine
        .widgets()
        .get_mut(first)
        .unwrap()
        .set_color(Color::RGB(255, 0, 0));

    assert_eq!(
        f.engine.widgets().get(first).unwrap().get_color(),
        Color::RGB(255, 0, 0)
    );
    assert_eq!(f.engine.widgets().part(f.id, "row:0"), Some(first));
}

#[test]
fn row_style_callback_styles_model_rows() {
    let model = VecModel::new(["a", "b", "c"]);
    let mut f = Fixture::new(
        ListView::new(model)
            .at(10, 10)
            .size(200, 100)
            .row_style(|row| RowStyle {
                color: (row % 2 == 1).then_some(Color::RGB(200, 0, 0)),
                ..RowStyle::default()
            }),
    );

    f.list().scroll_to_row(0);
    f.frame();

    // The callback ran for each shown row; the engine repaints the odd one.
    assert_eq!(f.shown().len(), 3);
}

#[test]
fn programmatic_selection_emits_nothing_and_scrolls() {
    let mut f = Fixture::numbered(50, SelectionMode::Single);

    f.list().select(30);
    f.frame();

    assert!(f.log.iter().all(|e| e.starts_with("top")), "{:?}", f.log);
    assert_eq!(f.selected(), vec![30]);
    assert!(f.list().top_row() > 0);
    assert!(f.shown().iter().any(|r| r.0 == 30 && r.2));
}

#[test]
fn an_empty_model_is_harmless() {
    let mut f = Fixture::new(ListView::new(VecModel::default()).at(10, 10).size(200, 100));

    f.key(Key::Down, Modifiers::empty());
    f.key(Key::Return, Modifiers::empty());
    f.key(Key::Char('x'), Modifiers::empty());
    f.click(0, Modifiers::empty(), 1);

    assert!(f.selected().is_empty());
    assert!(f.log.is_empty());
}

#[test]
fn a_custom_model_reports_changes_by_range() {
    #[derive(Default)]
    struct Counting {
        len: usize,
        pending: Vec<ModelChange>,
    }

    impl ListModel for Counting {
        fn len(&self) -> usize {
            self.len
        }

        fn item(&self, index: usize) -> String {
            format!("#{index}")
        }

        fn take_changes(&mut self) -> Vec<ModelChange> {
            std::mem::take(&mut self.pending)
        }
    }

    let mut f = Fixture::new(
        ListView::new(Counting {
            len: 3,
            pending: Vec::new(),
        })
        .at(10, 10)
        .size(200, 100),
    );

    {
        let model = f.list().model_mut::<Counting>().unwrap();

        model.len = 4;
        model.pending.push(ModelChange::Inserted(3..4));
    }

    f.frame();

    assert_eq!(f.shown().len(), 4);
    assert_eq!(f.shown()[3].1, "#3");
}

#[test]
fn row_height_is_a_property() {
    use pushrod::registry::PropertyValue;

    let mut f = Fixture::numbered(10, SelectionMode::Single);

    assert!(
        f.list()
            .set_property("row_height", &PropertyValue::Int(30))
            .unwrap()
    );
    assert_eq!(f.list().get_row_height(), 30);
    assert!(
        f.list()
            .set_property("row_height", &PropertyValue::Int(0))
            .is_err()
    );
}
