// Undo and redo example: a text field whose edits can be undone, and an application command that
// shares the same history.

//! Shows the undo stack.  A text field pushes its own edits, so typing "Hello" is one undo step;
//! the application pushes a `SetCount` command of its own.  The Undo and Redo actions enable and
//! disable themselves from the stack.  Run it headlessly with
//! `cargo run -p pushrod-widgets --example undo`.

use pushrod::engine::{Engine, Ui};
use pushrod::geometry::size;
use pushrod::input::InputEvent;
use pushrod::undo::Command;
use pushrod_widgets::text_field::TextField;

/// What the application remembers.
#[derive(Default)]
struct Document {
    count: u32,
}

/// An application command: sets the count, and takes it back.
struct SetCount {
    to: u32,
    from: u32,
}

impl Command<Document> for SetCount {
    fn description(&self) -> String {
        format!("Set count to {}", self.to)
    }

    fn apply(&mut self, doc: &mut Document, _ui: &mut Ui<Document>) {
        self.from = doc.count;
        doc.count = self.to;
    }

    fn revert(&mut self, doc: &mut Document, _ui: &mut Ui<Document>) {
        doc.count = self.from;
    }
}

/// Prints the field's text and what the Undo and Redo actions say.
///
/// # Arguments
///
/// * `title` - What just happened.
/// * `engine` - The engine.
/// * `field` - The text field's ID.
/// * `actions` - The Undo and Redo action IDs.
/// * `doc` - The application state.
fn show(
    title: &str,
    engine: &mut Engine<Document>,
    field: u32,
    actions: (u32, u32),
    doc: &Document,
) {
    let text = engine
        .widgets()
        .get_as::<TextField>(field)
        .map(|f| f.text())
        .unwrap_or_default();
    let can_undo = engine.action(pushrod::action::ActionId(actions.0));
    let can_redo = engine.action(pushrod::action::ActionId(actions.1));

    println!(
        "{title}: text = {text:?}, count = {}, Undo enabled = {}, Redo enabled = {}",
        doc.count,
        can_undo.is_some_and(|a| a.enabled),
        can_redo.is_some_and(|a| a.enabled)
    );
}

fn main() {
    let mut engine: Engine<Document> = Engine::new(size(320, 160), 30);
    let (undo, redo) = engine.add_undo_actions();
    let field = engine.add_widget(TextField::new().at(10, 10).size(200, 24));
    let mut doc = Document::default();
    let actions = (undo.0, redo.0);

    engine.widgets().set_focus(field);
    engine.run_idle(&mut doc);
    show("Start", &mut engine, field, actions, &doc);

    for part in ["H", "e", "l", "l", "o"] {
        engine.dispatch(InputEvent::Text(part.to_string()), &mut doc);
    }
    show("Typed Hello", &mut engine, field, actions, &doc);

    engine.push_command(SetCount { to: 3, from: 0 }, &mut doc);
    show("Set count", &mut engine, field, actions, &doc);

    engine.trigger_action(undo);
    engine.run_idle(&mut doc);
    show("Undo", &mut engine, field, actions, &doc);

    engine.trigger_action(undo);
    engine.run_idle(&mut doc);
    show("Undo", &mut engine, field, actions, &doc);

    engine.trigger_action(redo);
    engine.run_idle(&mut doc);
    show("Redo", &mut engine, field, actions, &doc);
}
