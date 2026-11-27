// Accessibility tests: roles, names, values and states of the standard widgets; incremental
// updates; lazy activation; and actions that run the same callbacks as user input.
// Sample text is fictional.

use pushrod::accessibility::{AccessAction, AccessActions, AccessNode, Role, Toggled};
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, EngineEventKind};
use pushrod::geometry::size;
use pushrod::input::InputEvent;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::progress_bar::ProgressBar;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text_field::TextField;

type Counts = (u32, u32, u32);

fn node(engine: &mut Engine<Counts>, id: u32) -> AccessNode {
    // Reset so the update is a full one and holds every node.
    engine.set_accessibility_active(false);
    engine.set_accessibility_active(true);

    engine
        .accessibility_update()
        .expect("full update")
        .nodes
        .into_iter()
        .find(|n| n.id == id)
        .expect("node")
}

fn engine() -> Engine<Counts> {
    Engine::new(size(300, 200), 30)
}

#[test]
fn nothing_is_built_until_assistive_technology_is_active() {
    let mut engine = engine();

    engine.add_widget(Button::new("OK").at(10, 10).size(60, 20));

    assert!(!engine.is_accessibility_active());
    assert!(engine.accessibility_update().is_none());
}

#[test]
fn a_button_is_announced_with_role_name_and_click() {
    let mut engine = engine();
    let ok = engine.add_widget(Button::new("_OK").at(10, 10).size(60, 20));
    let n = node(&mut engine, ok);

    assert_eq!(n.info.role, Role::Button);
    assert_eq!(n.info.name, "OK");
    assert!(n.info.actions.contains(AccessActions::CLICK));
    assert_eq!(
        (n.bounds.x, n.bounds.y, n.bounds.w, n.bounds.h),
        (10, 10, 60, 20)
    );
}

#[test]
fn a_check_box_reports_its_state() {
    let mut engine = engine();
    let id = engine.add_widget(CheckBox::new("Remember me").checked(true).at(10, 40));
    let n = node(&mut engine, id);

    assert_eq!(n.info.role, Role::CheckBox);
    assert_eq!(n.info.name, "Remember me");
    assert_eq!(n.info.toggled, Some(Toggled::True));
}

#[test]
fn a_text_field_exposes_text_and_selection_but_never_a_password() {
    let mut engine = engine();
    let mut field = TextField::new().with_label("Name").at(10, 70).size(120, 24);

    field.set_text("Ada Example");

    let plain = engine.add_widget(field);
    let secret = engine.add_widget(
        TextField::new()
            .with_label("Secret")
            .password()
            .at(10, 100)
            .size(120, 24),
    );
    let n = node(&mut engine, plain);

    assert_eq!(n.info.role, Role::TextInput);
    assert_eq!(n.info.name, "Name");
    assert_eq!(n.info.value.as_deref(), Some("Ada Example"));
    assert_eq!(n.info.selection.map(|s| s.caret), Some(11));
    assert_eq!(node(&mut engine, secret).info.role, Role::PasswordInput);

    engine
        .widgets()
        .get_as_mut::<TextField>(secret)
        .expect("field")
        .set_text("hunter2");

    assert_eq!(
        node(&mut engine, secret).info.value.as_deref(),
        Some("*******")
    );
}

#[test]
fn numeric_widgets_report_value_and_range() {
    let mut engine = engine();
    let slider = engine.add_widget(Slider::new(0.0, 1.0).at(10, 10).size(100, 20).value(0.0));
    let bar = engine.add_widget(ProgressBar::new(1.0).value(0.5).at(10, 40).size(100, 10));
    let s = node(&mut engine, slider)
        .info
        .numeric
        .expect("slider value");
    let b = node(&mut engine, bar).info.numeric.expect("bar value");

    assert!(s.min < s.max);
    assert_eq!(b.value, 0.5);
}

#[test]
fn updates_hold_only_the_changed_nodes() {
    let mut engine = engine();
    let a = engine.add_widget(CheckBox::new("A").at(10, 10));
    let b = engine.add_widget(CheckBox::new("B").at(10, 40));

    engine.set_accessibility_active(true);

    let full = engine.accessibility_update().expect("full");

    assert!(full.full);
    assert!(full.nodes.len() >= 3, "root and both check boxes");
    assert!(engine.accessibility_update().is_none(), "nothing changed");

    engine
        .widgets()
        .get_as_mut::<CheckBox>(b)
        .expect("check box")
        .set_checked(true);

    let delta = engine.accessibility_update().expect("delta");

    assert!(!delta.full);
    assert_eq!(
        delta.nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
        vec![b]
    );
    assert!(delta.nodes.iter().all(|n| n.id != a));
}

#[test]
fn hiding_a_widget_removes_its_node() {
    let mut engine = engine();
    let id = engine.add_widget(Button::new("Gone").at(10, 10).size(60, 20));

    engine.set_accessibility_active(true);
    let _ = engine.accessibility_update();

    engine
        .widgets()
        .get_mut(id)
        .expect("button")
        .set_hidden(true);

    let delta = engine.accessibility_update().expect("delta");

    assert_eq!(delta.removed, vec![id]);
}

#[test]
fn focus_changes_are_published() {
    let mut engine = engine();
    let id = engine.add_widget(CheckBox::new("OK").at(10, 10));

    engine.set_accessibility_active(true);
    let _ = engine.accessibility_update();

    assert!(engine.widgets().set_focus(id));
    assert_eq!(
        engine.accessibility_update().expect("delta").focus,
        Some(id)
    );
}

#[test]
fn click_action_runs_the_same_callbacks_as_a_click() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let ok = engine.add_widget(Button::new("OK").at(10, 10).size(60, 20));

    engine.on_click(ok, |c: &mut Counts, _| c.0 += 1);

    assert!(engine.perform_access_action(ok, AccessAction::Click, &mut counts));
    assert_eq!(counts.0, 1);
}

#[test]
fn click_action_toggles_a_check_box() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let id = engine.add_widget(CheckBox::new("Agree").at(10, 10));

    engine.on_toggle(id, |c: &mut Counts, _, _| c.1 += 1);
    engine.perform_access_action(id, AccessAction::Click, &mut counts);

    assert_eq!(counts.1, 1);
    assert_eq!(node(&mut engine, id).info.toggled, Some(Toggled::True));
}

#[test]
fn focus_action_moves_focus() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let id = engine.add_widget(CheckBox::new("OK").at(10, 10));

    assert!(engine.perform_access_action(id, AccessAction::Focus, &mut counts));
    assert_eq!(engine.widgets().focused(), Some(id));
}

#[test]
fn set_value_and_set_text_fire_change_callbacks() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let slider = engine.add_widget(Slider::new(0.0, 1.0).at(10, 10).size(100, 20));
    let field = engine.add_widget(TextField::new().at(10, 40).size(100, 24));

    engine.on_value_change(slider, |c: &mut Counts, _| c.0 += 1);
    engine.on_text_change(field, |c: &mut Counts, _| c.2 += 1);

    assert!(engine.perform_access_action(slider, AccessAction::SetValue(0.5), &mut counts));
    assert!(engine.perform_access_action(field, AccessAction::SetText("Hi".into()), &mut counts));
    assert_eq!((counts.0, counts.2), (1, 1));
    assert_eq!(
        engine
            .widgets()
            .get_as::<TextField>(field)
            .expect("field")
            .text(),
        "Hi"
    );
}

#[test]
fn actions_a_widget_cannot_take_are_refused() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let ok = engine.add_widget(Button::new("OK").at(10, 10).size(60, 20));

    // A button has no `value` property.
    assert!(!engine.perform_access_action(ok, AccessAction::SetValue(1.0), &mut counts));

    engine
        .widgets()
        .get_mut(ok)
        .expect("button")
        .set_enabled(false);

    assert!(!engine.perform_access_action(ok, AccessAction::Click, &mut counts));
    assert!(!engine.perform_access_action(999, AccessAction::Click, &mut counts));
}

#[test]
fn input_events_activate_and_act() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let ok = engine.add_widget(Button::new("OK").at(10, 10).size(60, 20));

    engine.on_click(ok, |c: &mut Counts, _| c.0 += 1);
    engine.dispatch(InputEvent::AccessibilityActivated(true), &mut counts);

    assert!(engine.is_accessibility_active());

    engine.dispatch(
        InputEvent::AccessAction {
            id: ok,
            action: AccessAction::Click,
        },
        &mut counts,
    );

    assert_eq!(counts.0, 1);
}

#[test]
fn accessibility_actions_emit_engine_events() {
    let mut engine = engine();
    let mut counts: Counts = (0, 0, 0);
    let ok = engine.add_widget(Button::new("OK").at(10, 10).size(60, 20));

    engine.on_engine_event_filtered(
        &[
            EngineEventKind::AccessibilityChanged,
            EngineEventKind::AccessActionPerformed,
        ],
        |c: &mut Counts, _, event| match event {
            EngineEvent::AccessibilityChanged { active: true } => c.1 += 1,
            EngineEvent::AccessActionPerformed { .. } => c.2 += 1,
            _ => {}
        },
    );
    engine.dispatch(InputEvent::AccessibilityActivated(true), &mut counts);
    engine.perform_access_action(ok, AccessAction::Click, &mut counts);

    assert_eq!((counts.1, counts.2), (1, 1));
}
