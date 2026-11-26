// RadioButton and RadioGroup tests: painting goes to a RecordingPainter and input through the
// WidgetCache, so no window is opened.

use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{Event, EventCtx, Propagation};
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::paint::Painter;
use pushrod::widget::{Behavior, VisualState, Widget, WidgetCore};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::prelude::RadioButton as PreludeRadio;
use pushrod_widgets::radio_button::{RadioButton, RadioGroup};

fn painted(widget: &dyn Widget) -> Vec<PaintCall> {
    let mut p = RecordingPainter::default();
    widget.paint(&mut p);
    p.calls
}

fn down(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn up(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseUp {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn motion(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseMove {
        position: point(x, y),
    }
}

fn key(key: Key, repeat: bool) -> InputEvent {
    InputEvent::KeyDown {
        key,
        modifiers: Modifiers::empty(),
        repeat,
    }
}

/// A root panel (id 0) holding a "Small / Medium / Large" group at (10, 10), 20 pixels per row
/// plus a 4 pixel gap; Medium is checked.
fn with_group() -> (WidgetCache, Vec<u32>) {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(300, 200)));

    let ids = RadioGroup::new(["Small", "Medium", "Large"])
        .at(10, 10)
        .selected(1)
        .add_to(&mut cache, 0);

    (cache, ids)
}

fn toggles(events: &[Event]) -> Vec<(u32, bool)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Toggled { id, checked } => Some((*id, *checked)),
            _ => None,
        })
        .collect()
}

fn click_at(cache: &mut WidgetCache, x: i32, y: i32) -> Vec<Event> {
    let mut events = cache.handle_event(motion(x, y));

    events.extend(cache.handle_event(down(x, y)));
    events.extend(cache.handle_event(up(x, y)));
    events
}

fn checked(cache: &WidgetCache, ids: &[u32]) -> Vec<bool> {
    ids.iter()
        .map(|&id| cache.get_as::<RadioButton>(id).unwrap().is_checked())
        .collect()
}

#[test]
fn a_group_starts_with_the_chosen_member_checked() {
    let (cache, ids) = with_group();

    assert_eq!(checked(&cache, &ids), vec![false, true, false]);
    assert_eq!(cache.checked_in_group(0), Some(ids[1]));
}

#[test]
fn a_group_can_start_with_nothing_checked() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().size(300, 200)));

    let ids = RadioGroup::new(["A", "B"])
        .none_selected()
        .add_to(&mut cache, 0);

    assert_eq!(checked(&cache, &ids), vec![false, false]);
    assert_eq!(cache.checked_in_group(0), None);
}

#[test]
fn group_members_are_stacked_with_the_gap() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().size(300, 200)));

    let ids = RadioGroup::new(["A", "B"])
        .at(5, 6)
        .item_size(100, 18)
        .gap(2)
        .add_to(&mut cache, 0);

    assert_eq!(cache.get(ids[0]).unwrap().get_origin(), point(5, 6));
    assert_eq!(cache.get(ids[1]).unwrap().get_origin(), point(5, 26));
    assert_eq!(cache.get(ids[1]).unwrap().get_size(), size(100, 18));
}

#[test]
fn clicking_one_checks_it_and_clears_its_sibling() {
    let (mut cache, ids) = with_group();

    // Large is the third row: y = 10 + 2 * 24 = 58.
    let events = click_at(&mut cache, 16, 68);

    assert_eq!(checked(&cache, &ids), vec![false, false, true]);
    assert_eq!(toggles(&events), vec![(ids[2], true), (ids[1], false)]);
    assert_eq!(cache.checked_in_group(0), Some(ids[2]));
}

#[test]
fn clicking_the_label_checks_it() {
    let (mut cache, ids) = with_group();

    click_at(&mut cache, 80, 20);

    assert_eq!(checked(&cache, &ids), vec![true, false, false]);
}

#[test]
fn clicking_a_checked_radio_does_nothing() {
    let (mut cache, ids) = with_group();
    let events = click_at(&mut cache, 16, 44);

    assert!(toggles(&events).is_empty());
    assert_eq!(checked(&cache, &ids), vec![false, true, false]);
}

#[test]
fn releasing_outside_cancels_the_click() {
    let (mut cache, ids) = with_group();

    cache.handle_event(motion(16, 68));
    cache.handle_event(down(16, 68));
    cache.handle_event(motion(250, 180));

    let events = cache.handle_event(up(250, 180));

    assert!(toggles(&events).is_empty());
    assert_eq!(checked(&cache, &ids), vec![false, true, false]);
}

#[test]
fn exactly_one_is_checked_after_any_sequence_of_clicks() {
    let (mut cache, ids) = with_group();

    for y in [20, 68, 44, 68, 20, 20] {
        click_at(&mut cache, 16, y);

        assert_eq!(checked(&cache, &ids).iter().filter(|&&c| c).count(), 1);
    }
}

#[test]
fn space_checks_the_focused_radio() {
    let (mut cache, ids) = with_group();

    assert!(cache.set_focus(ids[0]));

    let events = cache.handle_event(key(Key::Char(' '), false));

    assert_eq!(toggles(&events), vec![(ids[0], true), (ids[1], false)]);
    assert_eq!(checked(&cache, &ids), vec![true, false, false]);
}

#[test]
fn a_held_space_does_not_repeat() {
    let (mut cache, ids) = with_group();

    cache.set_focus(ids[0]);

    assert!(toggles(&cache.handle_event(key(Key::Char(' '), true))).is_empty());
}

#[test]
fn arrow_keys_move_within_the_group_and_check_the_new_member() {
    let (mut cache, ids) = with_group();

    cache.set_focus(ids[1]);

    let events = cache.handle_event(key(Key::Down, false));

    assert_eq!(cache.focused(), Some(ids[2]));
    assert_eq!(checked(&cache, &ids), vec![false, false, true]);
    assert_eq!(toggles(&events), vec![(ids[2], true), (ids[1], false)]);

    // Right wraps from the last to the first; Left and Up go back.
    cache.handle_event(key(Key::Right, false));
    assert_eq!(checked(&cache, &ids), vec![true, false, false]);

    cache.handle_event(key(Key::Up, false));
    assert_eq!(cache.focused(), Some(ids[2]));
    assert_eq!(checked(&cache, &ids), vec![false, false, true]);

    cache.handle_event(key(Key::Left, false));
    assert_eq!(checked(&cache, &ids), vec![false, true, false]);
}

#[test]
fn tab_visits_the_group_once_at_its_checked_member() {
    let (mut cache, ids) = with_group();

    cache.set_focus(ids[0]);
    cache.focus_next(false);

    // One group, so Tab wraps back to the checked member.
    assert_eq!(cache.focused(), Some(ids[1]));
}

#[test]
fn separate_parents_are_separate_groups() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().size(300, 200)));

    let left = cache.add_child(0, Box::new(Panel::new().at(0, 0).size(150, 200)));
    let right = cache.add_child(0, Box::new(Panel::new().at(150, 0).size(150, 200)));
    let a = RadioGroup::new(["A1", "A2"]).add_to(&mut cache, left);
    let b = RadioGroup::new(["B1", "B2"]).add_to(&mut cache, right);

    // Check A2 with the first group's A1 checked: B1 stays checked.
    click_at(&mut cache, 20, 34);

    assert_eq!(checked(&cache, &a), vec![false, true]);
    assert_eq!(checked(&cache, &b), vec![true, false]);
    assert_eq!(cache.checked_in_group(left), Some(a[1]));
    assert_eq!(cache.checked_in_group(right), Some(b[0]));
}

/// A stand-in for any widget with `EXCLUSIVE`: it checks itself and emits `Toggled` when it
/// receives `Activated`.
#[derive(Debug)]
struct Chip {
    core: WidgetCore,
}

impl Widget for Chip {
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    fn paint(&self, _p: &mut dyn Painter) {}

    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        if let Event::Activated { id } = event {
            let mut state = self.state();

            state.set(VisualState::CHECKED, true);
            self.set_state(state);
            ctx.emit(Event::Toggled {
                id: *id,
                checked: true,
            });
        }

        Propagation::Continue
    }
}

#[test]
fn any_widget_with_exclusive_clears_its_siblings() {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().size(300, 200)));

    let mut chips = Vec::new();

    for i in 0..2 {
        let mut chip = Chip {
            core: WidgetCore::new(point(0, i * 30), size(50, 20)),
        };

        chip.set_behavior(Behavior::FOCUSABLE | Behavior::EXCLUSIVE);
        chips.push(cache.add_child(0, Box::new(chip)));
    }

    cache.set_focus(chips[0]);
    cache.propagate(chips[0], &Event::Activated { id: chips[0] });

    let events = cache.propagate(chips[1], &Event::Activated { id: chips[1] });

    assert_eq!(toggles(&events), vec![(chips[1], true), (chips[0], false)]);
    assert_eq!(cache.checked_in_group(0), Some(chips[1]));
}

#[test]
fn a_disabled_radio_cannot_be_checked() {
    let (mut cache, ids) = with_group();

    cache.get_mut(ids[2]).unwrap().set_enabled(false);
    click_at(&mut cache, 16, 68);

    assert_eq!(checked(&cache, &ids), vec![false, true, false]);
}

#[test]
fn painting_draws_a_dot_only_when_checked() {
    let off = RadioButton::new("Small");
    let on = RadioButton::new("Small").checked(true);

    assert!(painted(&on).len() > painted(&off).len());
}

#[test]
fn the_dot_is_white_on_black_while_pressed() {
    let (mut cache, ids) = with_group();

    cache.handle_event(motion(16, 44));
    cache.handle_event(down(16, 44));

    let radio = cache.get_as::<RadioButton>(ids[1]).unwrap();

    assert!(radio.is_pressed());
    assert!(radio.is_checked());

    let calls = painted(radio);

    // The dot (the last fill) is white; the inside of the circle (the one before) is black.
    let fills: Vec<Color> = calls
        .iter()
        .filter_map(|c| match c {
            PaintCall::FillRect(_, color) => Some(*color),
            _ => None,
        })
        .collect();

    assert_eq!(fills.last(), Some(&Color::WHITE));
    assert!(fills.contains(&Color::BLACK));
}

#[test]
fn label_and_checked_state_accessors() {
    let mut radio = PreludeRadio::new("Medium");

    assert_eq!(radio.get_text(), "Medium");

    radio.set_text("Large");
    radio.set_checked(true);

    assert_eq!(radio.get_text(), "Large");
    assert!(radio.is_checked());
    assert!(radio.state().contains(VisualState::CHECKED));
    assert!(radio.behavior().contains(Behavior::EXCLUSIVE));
    assert!(radio.is_focusable());
}

#[test]
fn arrow_keys_skip_a_disabled_member() {
    let (mut cache, ids) = with_group();

    cache.get_mut(ids[2]).unwrap().set_enabled(false);
    cache.set_focus(ids[1]);
    cache.handle_event(key(Key::Down, false));

    assert_eq!(cache.focused(), Some(ids[0]));
    assert_eq!(checked(&cache, &ids), vec![true, false, false]);
}
