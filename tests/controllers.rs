// Controller tests.  A widget that only sets up controllers and implements
// `on_controller_event` gets them fed by the default `handle_event`.

use pushrod::cache::WidgetCache;
use pushrod::controllers::{
    ClickController, ControllerEvent, DragController, HoverController, KeyController,
    LongPressController, ScrollController,
};
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{point, size};
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, WidgetCore};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Records every controller result, and stops them if `stop` is set.
struct Probe {
    core: WidgetCore,
    seen: Rc<RefCell<Vec<ControllerEvent>>>,
    stop: bool,
}

impl Widget for Probe {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn on_controller_event(&mut self, event: &ControllerEvent, _ctx: &mut EventCtx) -> Propagation {
        self.seen.borrow_mut().push(event.clone());

        if self.stop {
            Propagation::Stop
        } else {
            Propagation::Continue
        }
    }
}

fn setup(
    stop: bool,
    build: impl FnOnce(&mut WidgetCore),
) -> (WidgetCache, u32, Rc<RefCell<Vec<ControllerEvent>>>) {
    let mut cache = WidgetCache::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut core = WidgetCore::new(point(0, 0), size(50, 50));

    cache.add(Box::new(RootWidget::new(size(100, 100))));
    build(&mut core);

    let id = cache.add_child(
        0,
        Box::new(Probe {
            core,
            seen: seen.clone(),
            stop,
        }),
    );

    (cache, id, seen)
}

#[test]
fn adding_controllers_adds_their_event_classes() {
    let (cache, id, _) = setup(false, |core| {
        core.add_controller(ScrollController::new());
        core.add_controller(KeyController::new());
    });
    let mask = cache.get(id).expect("widget").event_mask();

    assert!(mask.contains(EventMask::SCROLL | EventMask::KEYBOARD));
    assert!(!mask.intersects(EventMask::HOVER));
}

#[test]
fn default_handle_event_feeds_every_controller() {
    let (mut cache, id, seen) = setup(false, |core| {
        core.add_controller(ClickController::new());
        core.add_controller(HoverController::new());
        core.add_controller(LongPressController::with_delay(Duration::from_millis(10)));
    });

    cache.propagate(id, &Event::PointerEntered { id });
    cache.propagate(
        id,
        &Event::PointerPressed {
            id,
            button: MouseButton::Left,
            position: point(1, 1),
            count: 1,
        },
    );
    cache.propagate(
        id,
        &Event::Tick {
            elapsed: Duration::from_millis(10),
        },
    );

    assert_eq!(
        *seen.borrow(),
        vec![
            ControllerEvent::HoverEntered,
            ControllerEvent::ClickPressed,
            ControllerEvent::LongPressed {
                position: point(1, 1)
            }
        ]
    );
}

#[test]
fn drag_through_the_cache_reports_start_updates_and_end() {
    let (mut cache, id, seen) = setup(true, |core| core.add_controller(DragController::new()));
    let start = point(5, 5);

    cache.propagate(
        id,
        &Event::PointerPressed {
            id,
            button: MouseButton::Left,
            position: start,
            count: 1,
        },
    );
    cache.propagate(
        id,
        &Event::PointerMoved {
            id,
            position: point(15, 5),
        },
    );
    cache.propagate(
        id,
        &Event::PointerReleased {
            id,
            button: MouseButton::Left,
            position: point(15, 5),
            count: 1,
        },
    );

    assert_eq!(
        *seen.borrow(),
        vec![
            ControllerEvent::DragStarted { start },
            ControllerEvent::DragUpdated {
                start,
                offset: point(10, 0)
            },
            ControllerEvent::DragEnded {
                start,
                offset: point(10, 0)
            },
        ]
    );
}

#[test]
fn scroll_and_key_controllers_report_through_the_hook() {
    let (mut cache, id, seen) = setup(false, |core| {
        core.add_controller(ScrollController::new());
        core.add_controller(KeyController::for_keys(&[Keycode::Return]));
    });

    cache.propagate(
        id,
        &Event::Scrolled {
            id,
            delta: point(0, 2),
        },
    );
    cache.propagate(
        id,
        &Event::KeyPressed {
            id,
            key: Keycode::Return,
            modifiers: Mod::NOMOD,
            repeat: false,
        },
    );

    assert_eq!(seen.borrow().len(), 2);
}

#[test]
fn a_widget_without_controllers_ignores_events() {
    let (mut cache, id, seen) = setup(false, |_| {});

    cache.propagate(id, &Event::PointerEntered { id });

    assert!(seen.borrow().is_empty());
}
