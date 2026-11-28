// Undo and redo tests: the undo stack, command merging, text field edits, and the Undo and Redo
// actions.  Input goes through a headless `Engine`, so no window is opened.

use pushrod::action::StandardShortcut;
use pushrod::engine::{Engine, Ui};
use pushrod::event::{EngineEvent, EngineEventKind};
use pushrod::geometry::size;
use pushrod::input::{InputEvent, Key, Modifiers};
use pushrod::undo::{Command, TextEdit};
use pushrod_widgets::button::Button;
use pushrod_widgets::text_field::TextField;
use std::any::Any;

/// What the test application remembers.
#[derive(Default)]
struct App {
    total: i32,
    log: Vec<EngineEvent>,
}

/// Adds an amount to the total; two adds merge.
struct Add(i32);

impl Command<App> for Add {
    fn description(&self) -> String {
        format!("Add {}", self.0)
    }

    fn apply(&mut self, app: &mut App, _ui: &mut Ui<App>) {
        app.total += self.0;
    }

    fn revert(&mut self, app: &mut App, _ui: &mut Ui<App>) {
        app.total -= self.0;
    }

    fn merge(&mut self, next: &dyn Command<App>) -> bool {
        let Some(next) = next.as_any().and_then(|a| a.downcast_ref::<Add>()) else {
            return false;
        };

        self.0 += next.0;
        true
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

/// Sets the total; never merges.
struct Set {
    to: i32,
    from: i32,
}

impl Command<App> for Set {
    fn description(&self) -> String {
        "Set".to_string()
    }

    fn apply(&mut self, app: &mut App, _ui: &mut Ui<App>) {
        self.from = app.total;
        app.total = self.to;
    }

    fn revert(&mut self, app: &mut App, _ui: &mut Ui<App>) {
        app.total = self.from;
    }
}

fn engine() -> Engine<App> {
    let mut engine: Engine<App> = Engine::new(size(300, 200), 30);

    engine.on_engine_event(|app, _ui, event| app.log.push(event.clone()));
    engine
}

/// Types text into the focused field.
fn type_text(engine: &mut Engine<App>, app: &mut App, text: &str) {
    engine.dispatch(InputEvent::Text(text.to_string()), app);
}

/// Presses a key.
fn press(engine: &mut Engine<App>, app: &mut App, key: Key) {
    engine.dispatch(
        InputEvent::KeyDown {
            key,
            modifiers: Modifiers::empty(),
            repeat: false,
        },
        app,
    );
}

/// Adds a text field and focuses it.
fn focused_field(engine: &mut Engine<App>, app: &mut App, text: &str) -> u32 {
    let id = engine.add_widget(TextField::new().with_text(text).at(10, 10).size(200, 24));

    engine.widgets().set_focus(id);
    engine.run_idle(app);
    id
}

fn text_of(engine: &mut Engine<App>, id: u32) -> String {
    engine.widgets().get_as::<TextField>(id).unwrap().text()
}

#[test]
fn undo_and_redo_run_the_commands_in_order() {
    let mut engine = engine();
    let mut app = App::default();

    engine.push_command(Set { to: 5, from: 0 }, &mut app);
    engine.push_command(Set { to: 9, from: 0 }, &mut app);
    assert_eq!(app.total, 9);

    assert!(engine.undo(&mut app));
    assert_eq!(app.total, 5);
    assert!(engine.undo(&mut app));
    assert_eq!(app.total, 0);
    assert!(!engine.undo(&mut app));

    assert!(engine.redo(&mut app));
    assert_eq!(app.total, 5);
    assert!(engine.redo(&mut app));
    assert_eq!(app.total, 9);
    assert!(!engine.redo(&mut app));
}

#[test]
fn pushing_a_command_clears_the_redo_history() {
    let mut engine = engine();
    let mut app = App::default();

    engine.push_command(Set { to: 1, from: 0 }, &mut app);
    engine.undo(&mut app);
    assert!(engine.undo_stack().can_redo());

    engine.push_command(Set { to: 2, from: 0 }, &mut app);
    assert!(!engine.undo_stack().can_redo());
}

#[test]
fn a_recorded_command_is_not_applied_again() {
    let mut engine = engine();
    let mut app = App {
        total: 4,
        ..App::default()
    };

    engine.record_command(Add(4));
    assert_eq!(app.total, 4);

    engine.undo(&mut app);
    assert_eq!(app.total, 0);
}

#[test]
fn commands_that_merge_make_one_step_until_sealed() {
    let mut engine = engine();
    let mut app = App::default();

    engine.push_command(Add(1), &mut app);
    engine.push_command(Add(2), &mut app);
    assert_eq!(engine.undo_stack().undo_len(), 1);
    assert_eq!(engine.undo_stack().undo_description().unwrap(), "Add 3");

    engine.push_command(Set { to: 10, from: 0 }, &mut app);
    engine.push_command(Add(1), &mut app);
    assert_eq!(engine.undo_stack().undo_len(), 3);

    engine.undo(&mut app);
    engine.undo(&mut app);
    assert_eq!(app.total, 3);
    engine.undo(&mut app);
    assert_eq!(app.total, 0);
}

#[test]
fn the_stack_forgets_the_oldest_commands_past_its_limit() {
    let mut engine = engine();
    let mut app = App::default();

    engine.set_undo_limit(2);

    for to in 1..=4 {
        engine.push_command(Set { to, from: 0 }, &mut app);
    }

    assert_eq!(engine.undo_stack().undo_len(), 2);
    engine.clear_undo();
    assert!(!engine.undo_stack().can_undo());
}

#[test]
fn typing_in_a_text_field_is_one_undo_step() {
    let mut engine = engine();
    let mut app = App::default();
    let id = focused_field(&mut engine, &mut app, "");

    type_text(&mut engine, &mut app, "h");
    type_text(&mut engine, &mut app, "i");
    type_text(&mut engine, &mut app, "!");
    assert_eq!(text_of(&mut engine, id), "hi!");
    assert_eq!(engine.undo_stack().undo_len(), 1);

    assert!(engine.undo(&mut app));
    assert_eq!(text_of(&mut engine, id), "");

    assert!(engine.redo(&mut app));
    assert_eq!(text_of(&mut engine, id), "hi!");
}

#[test]
fn typing_then_deleting_are_separate_steps() {
    let mut engine = engine();
    let mut app = App::default();
    let id = focused_field(&mut engine, &mut app, "");

    type_text(&mut engine, &mut app, "abc");
    press(&mut engine, &mut app, Key::Backspace);
    assert_eq!(text_of(&mut engine, id), "ab");
    assert_eq!(engine.undo_stack().undo_len(), 2);

    engine.undo(&mut app);
    assert_eq!(text_of(&mut engine, id), "abc");
    engine.undo(&mut app);
    assert_eq!(text_of(&mut engine, id), "");
}

#[test]
fn losing_focus_ends_the_typing_run() {
    let mut engine = engine();
    let mut app = App::default();
    let id = focused_field(&mut engine, &mut app, "");
    let other = engine.add_widget(TextField::new().at(10, 50).size(200, 24));

    type_text(&mut engine, &mut app, "one");
    engine.widgets().set_focus(other);
    engine.run_idle(&mut app);
    engine.widgets().set_focus(id);
    engine.run_idle(&mut app);
    type_text(&mut engine, &mut app, "two");
    assert_eq!(text_of(&mut engine, id), "onetwo");
    assert_eq!(engine.undo_stack().undo_len(), 2);

    engine.undo(&mut app);
    assert_eq!(text_of(&mut engine, id), "one");
}

#[test]
fn an_undo_does_not_record_itself_as_an_edit() {
    let mut engine = engine();
    let mut app = App::default();

    focused_field(&mut engine, &mut app, "");
    type_text(&mut engine, &mut app, "abc");
    engine.undo(&mut app);
    engine.run_idle(&mut app);
    assert_eq!(engine.undo_stack().undo_len(), 0);
    assert_eq!(engine.undo_stack().redo_len(), 1);
}

#[test]
fn text_edits_and_application_commands_share_one_history() {
    let mut engine = engine();
    let mut app = App::default();
    let id = focused_field(&mut engine, &mut app, "");

    type_text(&mut engine, &mut app, "ab");
    engine.push_command(Set { to: 7, from: 0 }, &mut app);

    engine.undo(&mut app);
    assert_eq!(app.total, 0);
    assert_eq!(text_of(&mut engine, id), "ab");
    engine.undo(&mut app);
    assert_eq!(text_of(&mut engine, id), "");
}

#[test]
fn text_undo_can_be_turned_off() {
    let mut engine = engine();
    let mut app = App::default();

    engine.set_text_undo(false);
    focused_field(&mut engine, &mut app, "");
    type_text(&mut engine, &mut app, "abc");
    assert!(!engine.undo_stack().can_undo());
}

#[test]
fn password_edits_are_never_recorded() {
    let mut engine = engine();
    let mut app = App::default();
    let id = engine.add_widget(TextField::new().password().at(10, 10).size(200, 24));

    engine.widgets().set_focus(id);
    engine.run_idle(&mut app);
    type_text(&mut engine, &mut app, "secret");
    assert!(!engine.undo_stack().can_undo());
}

#[test]
fn text_edits_merge_only_when_they_continue_each_other() {
    let id = 3;
    let mut edit = TextEdit::new(id, "", "a");

    assert!(Command::<App>::merge(
        &mut edit,
        &TextEdit::new(id, "a", "ab")
    ));
    assert_eq!(edit.after, "ab");
    assert!(!Command::<App>::merge(
        &mut edit,
        &TextEdit::new(id, "x", "xy")
    ));
    assert!(!Command::<App>::merge(
        &mut edit,
        &TextEdit::new(4, "ab", "abc")
    ));
    assert!(!Command::<App>::merge(
        &mut edit,
        &TextEdit::new(id, "ab", "a")
    ));
}

#[test]
fn the_undo_and_redo_actions_follow_the_stack() {
    let mut engine = engine();
    let mut app = App::default();
    let (undo, redo) = engine.add_undo_actions();
    let undo_button = engine.add_widget(Button::new("x").at(10, 10).size(80, 24));

    engine.bind_widget(undo, undo_button);
    assert!(!engine.action(undo).unwrap().enabled);
    assert!(!engine.action(redo).unwrap().enabled);
    assert!(!engine.widgets().get(undo_button).unwrap().is_enabled());

    engine.push_command(Set { to: 1, from: 0 }, &mut app);
    assert!(engine.action(undo).unwrap().enabled);
    assert!(!engine.action(redo).unwrap().enabled);
    assert!(engine.widgets().get(undo_button).unwrap().is_enabled());

    engine.undo(&mut app);
    assert!(!engine.action(undo).unwrap().enabled);
    assert!(engine.action(redo).unwrap().enabled);
}

#[test]
fn triggering_the_actions_undoes_and_redoes() {
    let mut engine = engine();
    let mut app = App::default();
    let (undo, redo) = engine.add_undo_actions();

    engine.push_command(Set { to: 6, from: 0 }, &mut app);
    assert!(engine.trigger_action(undo));
    engine.run_idle(&mut app);
    assert_eq!(app.total, 0);

    assert!(engine.trigger_action(redo));
    engine.run_idle(&mut app);
    assert_eq!(app.total, 6);
}

#[test]
fn the_standard_shortcut_undoes_a_text_edit() {
    let mut engine = engine();
    let mut app = App::default();
    let id = focused_field(&mut engine, &mut app, "");

    engine.add_undo_actions();
    type_text(&mut engine, &mut app, "abc");

    let shortcut = StandardShortcut::Undo.shortcut();

    engine.dispatch(
        InputEvent::KeyDown {
            key: shortcut.key,
            modifiers: shortcut.modifiers,
            repeat: false,
        },
        &mut app,
    );
    engine.run_idle(&mut app);
    assert_eq!(text_of(&mut engine, id), "");
}

#[test]
fn undo_emits_engine_events() {
    let mut engine = engine();
    let mut app = App::default();

    engine.push_command(Set { to: 1, from: 0 }, &mut app);
    engine.undo(&mut app);
    engine.redo(&mut app);
    engine.run_idle(&mut app);

    let kinds: Vec<EngineEventKind> = app.log.iter().map(|e| e.kind()).collect();

    assert!(kinds.contains(&EngineEventKind::UndoStackChanged));
    assert!(kinds.contains(&EngineEventKind::CommandUndone));
    assert!(kinds.contains(&EngineEventKind::CommandRedone));
    assert!(app.log.iter().any(|e| matches!(
        e,
        EngineEvent::UndoStackChanged {
            can_undo: true,
            can_redo: false
        }
    )));
}
