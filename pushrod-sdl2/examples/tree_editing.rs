// Editing the widget tree at run time (issue #23).
//
// "Add box" adds a colored box to the left panel; "Remove box" removes the newest one (its
// texture is destroyed and its ID is never reused).  "Bring to front" moves the bottom box in
// its panel to the top of the stacking order, and "Move to other panel" moves the newest box
// between the two panels.  The status line shows the engine event each action emits.
//
// Run with: cargo run -p pushrod-sdl2 --example tree_editing

use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::size;
use pushrod::widget::{Widget, WidgetId};
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::button::Button;
use pushrod_widgets::frame::Frame;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;

/// The colors the boxes cycle through.
const COLORS: [Color; 4] = [Color::CYAN, Color::MAGENTA, Color::YELLOW, Color::GREEN];

/// Application state, passed mutably to every callback.
#[derive(Default)]
struct AppState {
    /// The boxes still in the window, oldest first.
    boxes: Vec<WidgetId>,
    /// How many boxes were ever added; sets each new box's color and position.
    added: usize,
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(size(640, 400), 30);

    let mut left = Panel::new().at(20, 60).size(290, 260);
    left.set_color(Color::RGB(200, 200, 200));
    let left_id = engine.add_widget(left);

    let mut right = Panel::new().at(330, 60).size(290, 260);
    right.set_color(Color::RGB(170, 170, 170));
    let right_id = engine.add_widget(right);

    let add_id = engine.add_widget(Button::new("Add box").at(20, 20).size(130, 24));
    let remove_id = engine.add_widget(Button::new("Remove box").at(160, 20).size(130, 24));
    let front_id = engine.add_widget(Button::new("Bring to front").at(300, 20).size(130, 24));
    let move_id = engine.add_widget(Button::new("Move to other panel").at(440, 20).size(180, 24));

    let status_id = engine.add_widget(
        Text::new("Engine events appear here.")
            .at(20, 340)
            .size(600, 18)
            .font_color(Color::BLACK),
    );

    // Each new box overlaps the previous one, so restacking is easy to see.
    engine.on_click(add_id, move |state, ui| {
        let offset = (state.added % 6) as i32 * 25;
        let mut frame = Frame::new()
            .at(20 + offset, 20 + offset)
            .size(120, 90)
            .border_color(Color::BLACK)
            .border_width(2);

        frame.set_color(COLORS[state.added % COLORS.len()]);
        state.added += 1;
        state
            .boxes
            .push(ui.widgets().add_child(left_id, Box::new(frame)));
    });

    engine.on_click(remove_id, |state, ui| {
        if let Some(id) = state.boxes.pop() {
            // Removing a box removes its children too, and leaves every other ID unchanged.
            let _ = ui.widgets().remove(id);
        }
    });

    engine.on_click(front_id, move |_state, ui| {
        for panel in [left_id, right_id] {
            if let Some(&bottom) = ui.widgets().children_of(panel).first() {
                // Past the end means top-most.
                let _ = ui.widgets().set_stacking_index(bottom, usize::MAX);
            }
        }
    });

    engine.on_click(move_id, move |state, ui| {
        if let Some(&id) = state.boxes.last() {
            let target = match ui.widgets().get(id).map(|w| w.get_parent()) {
                Some(parent) if parent == left_id => right_id,
                _ => left_id,
            };

            let _ = ui.widgets().reparent(id, target);
        }
    });

    // Every tree edit emits an engine event.
    engine.on_engine_event(move |_state, ui, event| {
        let text = match event {
            EngineEvent::WidgetAdded { id, parent } => format!("WidgetAdded: {id} in {parent}"),
            EngineEvent::WidgetRemoved { id, parent } => {
                format!("WidgetRemoved: {id} from {parent}")
            }
            EngineEvent::WidgetReparented {
                id,
                old_parent,
                new_parent,
            } => format!("WidgetReparented: {id} from {old_parent} to {new_parent}"),
            EngineEvent::WidgetRestacked {
                id,
                old_index,
                new_index,
                ..
            } => format!("WidgetRestacked: {id} from {old_index} to {new_index}"),
            _ => return,
        };

        if let Some(status) = ui.get_mut::<Text>(status_id) {
            status.set_text(text);
        }
    });

    pushrod_sdl2::run(
        engine,
        WindowSettings::new("pushrod: tree editing", size(640, 400)),
        AppState::default(),
    )?;

    Ok(())
}
