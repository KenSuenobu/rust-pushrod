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

//! Input-method (IME) composition: routing to the focused widget, the composition engine events,
//! and the caret rectangle handed to the backend.  Sample text is fictional.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::cache::WidgetCache;
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{Rect, point, rect, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Behavior, Widget, WidgetCore, WidgetId};
use std::cell::RefCell;
use std::rc::Rc;

type Log = Rc<RefCell<Vec<String>>>;

/// A focusable text editor stand-in that logs composition and typed text.
struct Editor {
    core: WidgetCore,
    log: Log,
}

impl Editor {
    /// Creates an editor at `x`, `y`, 40 by 20 pixels.
    fn new(x: i32, y: i32, log: &Log, mask: EventMask) -> Self {
        let mut core = WidgetCore::new(point(x, y), size(40, 20));

        core.set_event_mask(mask);

        let mut editor = Self {
            core,
            log: log.clone(),
        };

        editor.set_behavior(Behavior::FOCUSABLE);
        editor
    }
}

impl Widget for Editor {
    /// Returns the shared state.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the shared state for modification.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Draws nothing.
    ///
    /// # Arguments
    ///
    /// * `_p` - The painter.
    fn paint(&self, _p: &mut dyn Painter) {}

    /// Takes text input while focused.
    fn accepts_text_input(&self) -> bool {
        true
    }

    /// Reports a fixed caret rectangle.
    fn text_input_rect(&self) -> Option<Rect> {
        Some(rect(3, 4, 1, 12))
    }

    /// Logs composition and typed text.
    ///
    /// # Arguments
    ///
    /// * `event` - The event.
    /// * `_ctx` - The context.
    ///
    /// # Returns
    ///
    /// `Stop` for text events, otherwise `Continue`.
    fn handle_event(&mut self, event: &Event, _ctx: &mut EventCtx) -> Propagation {
        match event {
            Event::TextComposing { id, text, cursor } => {
                self.log
                    .borrow_mut()
                    .push(format!("composing {id} {text} {cursor:?}"));
                Propagation::Stop
            }
            Event::TextEntered { id, text } => {
                self.log.borrow_mut().push(format!("entered {id} {text}"));
                Propagation::Stop
            }
            _ => Propagation::Continue,
        }
    }
}

/// A cache with two editors, side by side under a root widget.
fn form(mask: EventMask) -> (WidgetCache, Log, WidgetId, WidgetId) {
    let log = Log::default();
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(200, 100))));

    let a = cache.add_child(0, Box::new(Editor::new(0, 0, &log, mask)));
    let b = cache.add_child(0, Box::new(Editor::new(60, 0, &log, mask)));

    (cache, log, a, b)
}

fn composition(text: &str, start: i32, length: i32) -> InputEvent {
    InputEvent::Composition {
        text: text.to_string(),
        start,
        length,
    }
}

/// The composition engine events queued so far.
fn engine_events(cache: &mut WidgetCache) -> Vec<EngineEvent> {
    cache
        .take_pending_events()
        .into_iter()
        .filter_map(|event| match event {
            Event::Engine(engine) => Some(engine),
            _ => None,
        })
        .filter(|event| {
            matches!(
                event,
                EngineEvent::CompositionStarted { .. }
                    | EngineEvent::CompositionUpdated { .. }
                    | EngineEvent::CompositionEnded { .. }
            )
        })
        .collect()
}

#[test]
fn composition_goes_to_the_focused_widget_only() {
    let (mut cache, log, a, _b) = form(EventMask::ALL);

    cache.handle_event(composition("にほ", 2, 0));
    assert!(log.borrow().is_empty(), "nothing is focused");

    cache.set_focus(a);
    cache.handle_event(composition("にほ", 2, 0));
    assert_eq!(*log.borrow(), [format!("composing {a} にほ Some((2, 0))")]);
}

#[test]
fn an_empty_composition_reports_no_cursor() {
    let (mut cache, log, a, _b) = form(EventMask::ALL);

    cache.set_focus(a);
    cache.handle_event(composition("", 0, 0));
    assert_eq!(*log.borrow(), [format!("composing {a}  None")]);
}

#[test]
fn a_widget_without_the_text_class_does_not_receive_composition() {
    let (mut cache, log, a, _b) = form(EventMask::KEYBOARD);

    cache.set_focus(a);
    cache.handle_event(composition("に", 1, 0));
    assert!(log.borrow().is_empty());
}

#[test]
fn the_run_loop_mask_drops_composition() {
    let log = Log::default();
    let mut engine: Engine<()> = Engine::new(size(200, 100), 30);
    let id = engine.add_widget(Editor::new(0, 0, &log, EventMask::ALL));

    engine.widgets().set_focus(id);
    engine.set_event_mask(EventMask::KEYBOARD);
    engine.dispatch(composition("に", 1, 0), &mut ());
    assert!(log.borrow().is_empty());

    engine.set_event_mask(EventMask::ALL);
    engine.dispatch(composition("に", 1, 0), &mut ());
    assert_eq!(log.borrow().len(), 1);
}

#[test]
fn a_commit_after_composing_ends_it_as_committed() {
    let (mut cache, _log, a, _b) = form(EventMask::ALL);

    cache.set_focus(a);
    engine_events(&mut cache);
    cache.handle_event(composition("に", 1, 0));
    cache.handle_event(composition("にほ", 2, 0));
    // The platform clears the composition, then sends the committed text.
    cache.handle_event(composition("", 0, 0));
    cache.handle_event(InputEvent::Text("日".to_string()));

    assert_eq!(
        engine_events(&mut cache),
        vec![
            EngineEvent::CompositionStarted { id: a },
            EngineEvent::CompositionUpdated {
                id: a,
                text: "に".to_string(),
                cursor: Some((1, 0))
            },
            EngineEvent::CompositionUpdated {
                id: a,
                text: "にほ".to_string(),
                cursor: Some((2, 0))
            },
            EngineEvent::CompositionEnded {
                id: a,
                committed: true
            },
        ]
    );
    assert_eq!(cache.composing(), None);
}

#[test]
fn a_cleared_composition_followed_by_another_event_was_cancelled() {
    let (mut cache, _log, a, _b) = form(EventMask::ALL);

    cache.set_focus(a);
    engine_events(&mut cache);
    cache.handle_event(composition("に", 1, 0));
    cache.handle_event(composition("", 0, 0));
    assert_eq!(cache.composing(), Some(a), "undecided until the next event");

    cache.handle_event(InputEvent::KeyDown {
        key: Key::Escape,
        modifiers: Modifiers::empty(),
        repeat: false,
    });

    let events = engine_events(&mut cache);

    assert_eq!(
        events.last(),
        Some(&EngineEvent::CompositionEnded {
            id: a,
            committed: false
        })
    );
    assert_eq!(cache.composing(), None);
}

#[test]
fn keys_while_composing_do_not_cancel_the_composition() {
    let (mut cache, _log, a, _b) = form(EventMask::ALL);

    cache.set_focus(a);
    cache.handle_event(composition("に", 1, 0));
    cache.handle_event(InputEvent::KeyDown {
        key: Key::Left,
        modifiers: Modifiers::empty(),
        repeat: false,
    });
    assert_eq!(cache.composing(), Some(a));
}

#[test]
fn moving_the_focus_cancels_the_composition() {
    let (mut cache, _log, a, b) = form(EventMask::ALL);

    cache.set_focus(a);
    engine_events(&mut cache);
    cache.handle_event(composition("に", 1, 0));
    cache.set_focus(b);

    let events = engine_events(&mut cache);

    assert_eq!(
        events.last(),
        Some(&EngineEvent::CompositionEnded {
            id: a,
            committed: false
        })
    );
}

#[test]
fn typing_without_composing_emits_no_composition_events() {
    let (mut cache, _log, a, _b) = form(EventMask::ALL);

    cache.set_focus(a);
    engine_events(&mut cache);
    cache.handle_event(InputEvent::Text("a".to_string()));
    assert!(engine_events(&mut cache).is_empty());
}

#[test]
fn the_focused_editors_caret_is_in_window_coordinates() {
    let (mut cache, _log, _a, b) = form(EventMask::ALL);

    assert_eq!(cache.focused_text_input_rect(), None);
    cache.set_focus(b);
    assert_eq!(cache.focused_text_input_rect(), Some(rect(63, 4, 1, 12)));
}

#[test]
fn the_engine_gives_the_backend_the_caret_rectangle() {
    let log = Log::default();
    let mut engine: Engine<()> = Engine::new(size(200, 100), 30);
    let mut backend = HeadlessBackend::new(size(200, 100));

    engine.add_widget(Editor::new(10, 10, &log, EventMask::ALL));
    assert_eq!(backend.text_input_rect, None);

    backend.push_event(InputEvent::MouseDown {
        position: point(12, 12),
        button: MouseButton::Left,
        clicks: 1,
    });
    backend.push_event(InputEvent::MouseUp {
        position: point(12, 12),
        button: MouseButton::Left,
        clicks: 1,
    });
    engine.run(&mut backend, ());

    assert_eq!(backend.text_input_rect, Some(rect(13, 14, 1, 12)));
}
