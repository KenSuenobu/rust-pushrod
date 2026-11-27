// Keyboard mnemonic tests: Alt+letter clicks buttons, toggles check boxes, and moves focus from a
// label to the next focusable widget; scoped to the top modal dialog; duplicates are reported.
// Sample text is fictional.

use pushrod::cache::WidgetCache;
use pushrod::event::{EngineEvent, Event};
use pushrod::input::{InputEvent, Key, Modifiers};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::text::Text;

fn form() -> WidgetCache {
    let mut cache = WidgetCache::new();

    cache.add(Box::new(Panel::new().at(0, 0).size(400, 300)));
    cache
}

fn press(cache: &mut WidgetCache, c: char, modifiers: Modifiers) -> Vec<Event> {
    cache.handle_event(InputEvent::KeyDown {
        key: Key::Char(c),
        modifiers,
        repeat: false,
    })
}

fn engine(cache: &mut WidgetCache, events: Vec<Event>) -> Vec<EngineEvent> {
    events
        .into_iter()
        .chain(cache.take_pending_events())
        .filter_map(|event| match event {
            Event::Engine(engine) => Some(engine),
            _ => None,
        })
        .collect()
}

fn clicked(events: &[Event], id: u32) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::Clicked { id: i, .. } if *i == id))
}

#[test]
fn alt_letter_clicks_a_button() {
    let mut cache = form();
    let save = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));

    let events = press(&mut cache, 's', Modifiers::ALT);

    assert!(clicked(&events, save));
    assert!(
        engine(&mut cache, events)
            .iter()
            .any(|e| matches!(e, EngineEvent::MnemonicActivated { id, key: 's' } if *id == save))
    );
}

#[test]
fn the_letter_is_matched_without_regard_to_case() {
    let mut cache = form();
    let save = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));

    assert!(clicked(
        &press(&mut cache, 's', Modifiers::ALT | Modifiers::SHIFT),
        save
    ));
}

#[test]
fn alt_letter_toggles_a_check_box() {
    let mut cache = form();
    let id = cache.add_child(0, Box::new(CheckBox::new("_Remember me").at(10, 50)));

    let events = press(&mut cache, 'r', Modifiers::ALT);

    assert!(cache.get_as::<CheckBox>(id).unwrap().is_checked());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Toggled { checked: true, .. }))
    );

    press(&mut cache, 'r', Modifiers::ALT);

    assert!(!cache.get_as::<CheckBox>(id).unwrap().is_checked());
}

#[test]
fn a_label_moves_focus_to_the_next_focusable_widget() {
    let mut cache = form();

    cache.add_child(0, Box::new(CheckBox::new("Other").at(10, 100)));
    cache.add_child(0, Box::new(Text::new("_Name").at(10, 10)));

    let target = cache.add_child(0, Box::new(CheckBox::new("Field").at(100, 10)));

    let events = press(&mut cache, 'n', Modifiers::ALT);

    assert_eq!(cache.focused(), Some(target));
    assert!(
        engine(&mut cache, events)
            .iter()
            .any(|e| matches!(e, EngineEvent::MnemonicActivated { key: 'n', .. }))
    );
}

#[test]
fn the_bare_letter_works_when_no_text_editor_has_focus() {
    let mut cache = form();
    let save = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));

    assert!(clicked(&press(&mut cache, 's', Modifiers::empty()), save));
}

#[test]
fn control_and_command_do_not_activate_mnemonics() {
    let mut cache = form();
    let save = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));

    assert!(!clicked(&press(&mut cache, 's', Modifiers::CTRL), save));
    assert!(!clicked(&press(&mut cache, 's', Modifiers::GUI), save));
}

#[test]
fn disabled_and_hidden_widgets_are_skipped() {
    let mut cache = form();
    let save = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));

    cache.get_mut(save).unwrap().set_enabled(false);
    assert!(!clicked(&press(&mut cache, 's', Modifiers::ALT), save));

    cache.get_mut(save).unwrap().set_enabled(true);
    cache.get_mut(save).unwrap().set_hidden(true);
    assert!(!clicked(&press(&mut cache, 's', Modifiers::ALT), save));
}

#[test]
fn a_literal_underscore_is_not_a_mnemonic() {
    let mut cache = form();
    let id = cache.add_child(0, Box::new(Button::new("a__b").at(10, 10).size(80, 24)));

    assert!(!clicked(&press(&mut cache, 'b', Modifiers::ALT), id));
}

#[test]
fn only_the_top_modal_dialog_is_searched() {
    let mut cache = form();
    let behind = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));
    let dialog = cache.add_child(0, Box::new(Panel::new().at(100, 100).size(200, 100)));
    let inside = cache.add_child(
        dialog,
        Box::new(Button::new("_Save").at(10, 10).size(80, 24)),
    );

    cache.open_modal(dialog).unwrap();

    let events = press(&mut cache, 's', Modifiers::ALT);

    assert!(clicked(&events, inside));
    assert!(!clicked(&events, behind));
    assert!(cache.mnemonic_conflicts().is_empty());
}

#[test]
fn duplicates_are_reported_and_the_lowest_id_wins() {
    let mut cache = form();
    let first = cache.add_child(0, Box::new(Button::new("_Save").at(10, 10).size(80, 24)));
    let second = cache.add_child(0, Box::new(Button::new("Sa_ve").at(100, 10).size(80, 24)));

    assert_eq!(
        cache.mnemonic_conflicts(),
        vec![('s', vec![first]), ('v', vec![second])]
            .into_iter()
            .filter(|(_, ids)| ids.len() > 1)
            .collect::<Vec<_>>()
    );

    let third = cache.add_child(0, Box::new(Button::new("_Stop").at(200, 10).size(80, 24)));

    assert_eq!(cache.mnemonic_conflicts(), vec![('s', vec![first, third])]);

    let events = press(&mut cache, 's', Modifiers::ALT);

    assert!(clicked(&events, first));
    assert!(!clicked(&events, third));

    if cfg!(debug_assertions) {
        assert!(engine(&mut cache, events).iter().any(|e| matches!(
            e,
            EngineEvent::MnemonicConflict { key: 's', ids } if ids == &vec![first, third]
        )));
    }

    let _ = second;
}
