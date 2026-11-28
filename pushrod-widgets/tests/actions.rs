// Action tests: controls are bound to shared actions on an Engine, and input goes through
// `Engine::dispatch`, so no window is opened.

use pushrod::action::{Action, ShortcutScope, StandardShortcut};
use pushrod::engine::Engine;
use pushrod::event::EngineEvent;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::widget::Widget;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::panel::Panel;

#[derive(Default)]
struct App {
    triggered: u32,
    log: Vec<EngineEvent>,
}

fn click(engine: &mut Engine<App>, app: &mut App, x: i32, y: i32) {
    for event in [
        InputEvent::MouseMove {
            position: point(x, y),
        },
        InputEvent::MouseDown {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
        InputEvent::MouseUp {
            position: point(x, y),
            button: MouseButton::Left,
            clicks: 1,
        },
    ] {
        engine.dispatch(event, app);
    }
}

fn shortcut(engine: &mut Engine<App>, app: &mut App, key: char, modifiers: Modifiers) {
    engine.dispatch(
        InputEvent::KeyDown {
            key: Key::Char(key),
            modifiers,
            repeat: false,
        },
        app,
    );
}

fn command() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::GUI
    } else {
        Modifiers::CTRL
    }
}

fn engine() -> Engine<App> {
    let mut engine: Engine<App> = Engine::new(size(300, 200), 30);

    engine.on_engine_event(|app, _ui, event| app.log.push(event.clone()));
    engine
}

#[test]
fn a_bound_button_takes_the_action_state_and_triggers_it() {
    let mut engine = engine();
    let save = engine.add_action(Action::new("Save").tooltip("Save it"));
    let button = engine.add_widget(Button::new("x").at(10, 10).size(80, 24));

    assert!(engine.bind_widget(save, button));
    engine.on_action(save, |app, _ui| app.triggered += 1);

    let widget = engine.widgets().get_as::<Button>(button).unwrap();

    assert_eq!(widget.get_text(), "Save");
    assert_eq!(widget.tooltip(), "Save it");

    let mut app = App::default();

    click(&mut engine, &mut app, 20, 20);
    assert_eq!(app.triggered, 1);
    assert!(
        app.log
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionInvoked { action } if *action == save))
    );
}

#[test]
fn toggling_the_toolbar_check_box_checks_the_menu_one() {
    let mut engine = engine();
    let bold = engine.add_action(Action::new("Bold").checkable(false));
    let toolbar = engine.add_widget(CheckBox::new("x").at(10, 10).size(120, 20));
    let menu = engine.add_widget(CheckBox::new("x").at(10, 50).size(120, 20));

    engine.bind_widget(bold, toolbar);
    engine.bind_widget(bold, menu);

    let mut app = App::default();

    click(&mut engine, &mut app, 15, 20);
    assert!(engine.action(bold).unwrap().checked);
    assert!(
        engine
            .widgets()
            .get_as::<CheckBox>(menu)
            .unwrap()
            .is_checked()
    );

    click(&mut engine, &mut app, 15, 20);
    assert!(
        !engine
            .widgets()
            .get_as::<CheckBox>(menu)
            .unwrap()
            .is_checked()
    );
    assert!(
        app.log
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionChanged { action } if *action == bold))
    );
}

#[test]
fn disabling_an_action_disables_every_control_and_blocks_triggers() {
    let mut engine = engine();
    let save = engine.add_action(Action::new("Save").standard_shortcut(StandardShortcut::Save));
    let a = engine.add_widget(Button::new("x").at(10, 10).size(80, 24));
    let b = engine.add_widget(Button::new("x").at(10, 50).size(80, 24));

    engine.bind_widget(save, a);
    engine.bind_widget(save, b);
    engine.on_action(save, |app, _ui| app.triggered += 1);
    assert!(engine.set_action_enabled(save, false));
    assert!(!engine.widgets().get(a).unwrap().is_enabled());
    assert!(!engine.widgets().get(b).unwrap().is_enabled());

    let mut app = App::default();

    click(&mut engine, &mut app, 20, 20);
    shortcut(&mut engine, &mut app, 's', command());
    assert_eq!(app.triggered, 0);

    engine.set_action_enabled(save, true);
    assert!(engine.widgets().get(a).unwrap().is_enabled());
    shortcut(&mut engine, &mut app, 's', command());
    assert_eq!(app.triggered, 1);
}

#[test]
fn the_platform_standard_shortcut_triggers_and_other_keys_do_not() {
    let mut engine = engine();
    let save = engine.add_action(Action::new("Save").standard_shortcut(StandardShortcut::Save));

    engine.on_action(save, |app, _ui| app.triggered += 1);

    let mut app = App::default();

    shortcut(&mut engine, &mut app, 's', Modifiers::empty());
    shortcut(&mut engine, &mut app, 's', Modifiers::ALT);
    assert_eq!(app.triggered, 0);

    shortcut(&mut engine, &mut app, 's', command());
    assert_eq!(app.triggered, 1);

    engine.dispatch(
        InputEvent::KeyDown {
            key: Key::Char('s'),
            modifiers: command(),
            repeat: true,
        },
        &mut app,
    );
    assert_eq!(app.triggered, 1);
}

#[test]
fn a_shortcut_toggles_a_checkable_action_and_its_controls() {
    let mut engine = engine();
    let bold = engine.add_action(
        Action::new("Bold")
            .checkable(false)
            .shortcut(pushrod::action::Shortcut::new(Key::Char('b'), command())),
    );
    let check = engine.add_widget(CheckBox::new("x").at(10, 10).size(120, 20));

    engine.bind_widget(bold, check);

    let mut app = App::default();

    shortcut(&mut engine, &mut app, 'b', command());
    assert!(engine.action(bold).unwrap().checked);
    assert!(
        engine
            .widgets()
            .get_as::<CheckBox>(check)
            .unwrap()
            .is_checked()
    );
}

#[test]
fn exclusive_groups_check_one_member() {
    let mut engine = engine();
    let group = engine.add_action_group(true);
    let mut left = Action::new("Left").checkable(true);
    let mut right = Action::new("Right").checkable(false);

    left.group = Some(group);
    right.group = Some(group);

    let left = engine.add_action(left);
    let right = engine.add_action(right);
    let l = engine.add_widget(CheckBox::new("x").at(10, 10).size(120, 20));
    let r = engine.add_widget(CheckBox::new("x").at(10, 50).size(120, 20));

    engine.bind_widget(left, l);
    engine.bind_widget(right, r);

    let mut app = App::default();

    click(&mut engine, &mut app, 15, 60);
    assert!(engine.action(right).unwrap().checked);
    assert!(!engine.action(left).unwrap().checked);
    assert!(!engine.widgets().get_as::<CheckBox>(l).unwrap().is_checked());

    // Clicking the checked member again cannot leave the group empty.
    click(&mut engine, &mut app, 15, 60);
    assert!(engine.action(right).unwrap().checked);
    assert!(engine.widgets().get_as::<CheckBox>(r).unwrap().is_checked());
}

#[test]
fn subtree_shortcuts_need_focus_inside_the_subtree() {
    let mut engine = engine();
    let panel = engine.add_widget(Panel::new().at(0, 0).size(150, 40));
    let inside = engine.add_child_widget(panel, CheckBox::new("in").at(10, 10).size(80, 20));
    let outside = engine.add_widget(CheckBox::new("out").at(10, 100).size(80, 20));
    let find = engine.add_action(
        Action::new("Find")
            .standard_shortcut(StandardShortcut::Find)
            .scope(ShortcutScope::Subtree(panel)),
    );

    engine.on_action(find, |app, _ui| app.triggered += 1);

    let mut app = App::default();

    assert!(engine.widgets().set_focus(outside));
    shortcut(&mut engine, &mut app, 'f', command());
    assert_eq!(app.triggered, 0);

    assert!(engine.widgets().set_focus(inside));
    shortcut(&mut engine, &mut app, 'f', command());
    assert_eq!(app.triggered, 1);
}

#[test]
fn ui_changes_reach_bound_controls_and_emit_events() {
    let mut engine = engine();
    let undo = engine.add_action(Action::new("Undo").enabled(false));
    let button = engine.add_widget(Button::new("x").at(10, 10).size(80, 24));
    let other = engine.add_action(Action::new("Do"));
    let go = engine.add_widget(Button::new("x").at(10, 50).size(80, 24));

    engine.bind_widget(undo, button);
    engine.bind_widget(other, go);
    engine.on_action(other, move |_app, ui| {
        ui.set_action_enabled(undo, true);
        ui.set_action_label(undo, "Undo typing");
    });

    let mut app = App::default();

    click(&mut engine, &mut app, 20, 60);
    assert!(engine.widgets().get(button).unwrap().is_enabled());
    assert_eq!(
        engine
            .widgets()
            .get_as::<Button>(button)
            .unwrap()
            .get_text(),
        "Undo typing"
    );
    assert!(
        app.log
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionChanged { action } if *action == undo))
    );
}

#[test]
fn registering_an_action_emits_an_event_and_unknown_binds_fail() {
    let mut engine = engine();
    let save = engine.add_action(Action::new("Save"));

    assert!(!engine.bind_widget(save, 999));
    assert!(!engine.bind_widget(pushrod::action::ActionId(99), 0));
    assert!(!engine.trigger_action(pushrod::action::ActionId(99)));

    let mut app = App::default();

    engine.run_idle(&mut app);
    assert!(
        app.log
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionAdded { action } if *action == save))
    );
}
