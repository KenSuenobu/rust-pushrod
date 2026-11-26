// Per-state appearance tests
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

//! The `WidgetCache` sets hover and press flags, and `Widget::style` follows them.

use pushrod::appearance::StateStyle;
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{Event, EventCtx, EventMask, Propagation};
use pushrod::geometry::{point, size};
use pushrod::input::InputEvent;
use pushrod::input::MouseButton;
use pushrod::paint::Painter;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{VisualState, Widget, WidgetCore, WidgetId};

/// A widget that draws nothing and has a full event mask.
struct Plain {
    core: WidgetCore,
}

impl Widget for Plain {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventCtx) -> Propagation {
        Propagation::Continue
    }
}

/// Builds a root with two 20x20 widgets side by side, at x = 0 and x = 30.
fn form() -> (WidgetCache, WidgetId, WidgetId) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(RootWidget::new(size(100, 100))));

    let mut ids = Vec::new();

    for x in [0, 30] {
        let mut core = WidgetCore::new(point(x, 0), size(20, 20));

        core.set_event_mask(EventMask::ALL);
        ids.push(cache.add_child(0, Box::new(Plain { core })));
    }

    (cache, ids[0], ids[1])
}

fn motion(cache: &mut WidgetCache, x: i32) {
    cache.handle_event(InputEvent::MouseMove {
        position: point(x, 5),
    });
}

fn button(cache: &mut WidgetCache, x: i32, down: bool) {
    let event = if down {
        InputEvent::MouseDown {
            position: point(x, 5),
            button: MouseButton::Left,
            clicks: 1,
        }
    } else {
        InputEvent::MouseUp {
            position: point(x, 5),
            button: MouseButton::Left,
            clicks: 1,
        }
    };

    cache.handle_event(event);
}

fn state(cache: &WidgetCache, id: WidgetId) -> VisualState {
    cache.get(id).unwrap().state()
}

fn blue() -> StateStyle {
    StateStyle {
        background: Some(Color::BLUE),
        ..StateStyle::default()
    }
}

#[test]
fn hover_flag_follows_the_pointer() {
    let (mut cache, a, b) = form();

    motion(&mut cache, 5);
    assert!(state(&cache, a).contains(VisualState::HOVERED));

    motion(&mut cache, 35);
    assert!(!state(&cache, a).contains(VisualState::HOVERED));
    assert!(state(&cache, b).contains(VisualState::HOVERED));
}

#[test]
fn pressed_flag_is_set_on_press_and_cleared_on_release() {
    let (mut cache, a, _) = form();

    motion(&mut cache, 5);
    button(&mut cache, 5, true);
    assert!(state(&cache, a).contains(VisualState::PRESSED));

    button(&mut cache, 5, false);
    assert!(!state(&cache, a).contains(VisualState::PRESSED));
}

#[test]
fn pressed_flag_clears_when_the_pointer_leaves_while_held_and_returns_on_re_entry() {
    let (mut cache, a, b) = form();

    motion(&mut cache, 5);
    button(&mut cache, 5, true);
    motion(&mut cache, 35);
    assert!(!state(&cache, a).contains(VisualState::PRESSED));
    assert!(!state(&cache, b).contains(VisualState::PRESSED));

    motion(&mut cache, 5);
    assert!(state(&cache, a).contains(VisualState::PRESSED));

    button(&mut cache, 5, false);
    assert!(!state(&cache, a).contains(VisualState::PRESSED));
}

#[test]
fn hover_style_applies_while_hovered_and_pressed_style_wins_while_pressed() {
    let (mut cache, a, _) = form();
    let dark = StateStyle {
        background: Some(Color::RGB(0, 0, 128)),
        ..StateStyle::default()
    };

    cache
        .get_mut(a)
        .unwrap()
        .set_style(VisualState::HOVERED, blue());
    cache
        .get_mut(a)
        .unwrap()
        .set_style(VisualState::PRESSED, dark);

    assert_eq!(cache.get(a).unwrap().style().background, None);

    motion(&mut cache, 5);
    assert_eq!(cache.get(a).unwrap().style().background, Some(Color::BLUE));

    button(&mut cache, 5, true);
    assert_eq!(
        cache.get(a).unwrap().style().background,
        Some(Color::RGB(0, 0, 128))
    );

    button(&mut cache, 5, false);
    assert_eq!(cache.get(a).unwrap().style().background, Some(Color::BLUE));

    motion(&mut cache, 35);
    assert_eq!(cache.get(a).unwrap().style().background, None);
}

#[test]
fn hover_repaints_only_when_the_effective_style_changes() {
    let (mut cache, a, _) = form();

    // No hover style: entering the widget does not change how it looks.
    cache.get_mut(a).unwrap().set_invalidated(false);
    motion(&mut cache, 5);
    assert!(!cache.get(a).unwrap().is_invalidated());

    motion(&mut cache, 35);
    cache
        .get_mut(a)
        .unwrap()
        .set_style(VisualState::HOVERED, blue());
    cache.get_mut(a).unwrap().set_invalidated(false);

    // With a hover style, entering it repaints.
    motion(&mut cache, 5);
    assert!(cache.get(a).unwrap().is_invalidated());
}

#[test]
fn set_style_invalidates_only_when_the_current_look_changes() {
    let (mut cache, a, _) = form();
    let widget = cache.get_mut(a).unwrap();

    widget.set_invalidated(false);
    widget.set_style(VisualState::HOVERED, blue());
    assert!(
        !widget.is_invalidated(),
        "styling a state that is not active changes nothing"
    );

    widget.set_style(VisualState::empty(), blue());
    assert!(widget.is_invalidated(), "the normal look is active");
}

#[test]
fn disabled_style_beats_hover_and_unset_fields_fall_back_to_normal() {
    let (mut cache, a, _) = form();
    let widget = cache.get_mut(a).unwrap();

    widget.set_style(
        VisualState::empty(),
        StateStyle {
            border_width: Some(2),
            ..StateStyle::default()
        },
    );
    widget.set_style(VisualState::HOVERED, blue());
    widget.set_style(
        VisualState::DISABLED,
        StateStyle {
            background: Some(Color::GRAY),
            ..StateStyle::default()
        },
    );
    widget.set_state(VisualState::HOVERED | VisualState::DISABLED);

    let style = widget.style();

    assert_eq!(style.background, Some(Color::GRAY));
    assert_eq!(style.border_width, Some(2));
}
