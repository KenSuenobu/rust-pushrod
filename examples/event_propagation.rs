//! Shows the capture, target, and bubble phases without opening a window.
//!
//! A dialog (which captures) contains a group that contains a text field.  A Return key press for
//! the field travels: dialog (capture), field (target), group (bubble), dialog (bubble, where the
//! dialog "presses its default button" and stops delivery).
//!
//! Run it with `cargo run --example event_propagation`.

use pushrod::cache::WidgetCache;
use pushrod::event::{Event, EventCtx, EventMask, EventPhase, Propagation};
use pushrod::geometry::{point, size};
use pushrod::input::{Key, Modifiers};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetCore};

/// A widget that prints each phase it sees; the dialog also captures and handles Return.
struct Node {
    core: WidgetCore,
    name: &'static str,
    is_dialog: bool,
}

impl Widget for Node {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Only the dialog captures.
    fn handles_capture(&self) -> bool {
        self.is_dialog
    }

    /// Prints the phase; the dialog stops the Return key when it bubbles to it.
    fn handle_event(&mut self, _event: &Event, ctx: &mut EventCtx) -> Propagation {
        println!("{:>7}: {:?}", self.name, ctx.phase());

        if self.is_dialog && ctx.phase() == EventPhase::Bubble {
            println!("{:>7}: default button pressed, stop", self.name);
            return Propagation::Stop;
        }

        Propagation::Continue
    }
}

/// Adds a node with every event class enabled.
fn node(cache: &mut WidgetCache, parent: u32, name: &'static str, is_dialog: bool) -> u32 {
    let mut core = WidgetCore::new(point(0, 0), size(50, 50));

    core.set_event_mask(EventMask::ALL);
    cache.add_child(
        parent,
        Box::new(Node {
            core,
            name,
            is_dialog,
        }),
    )
}

fn main() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 200))));

    let dialog = node(&mut cache, 0, "dialog", true);
    let group = node(&mut cache, dialog, "group", false);
    let field = node(&mut cache, group, "field", false);

    cache.propagate(
        field,
        &Event::KeyPressed {
            id: field,
            key: Key::Return,
            modifiers: Modifiers::empty(),
            repeat: false,
        },
    );
}
