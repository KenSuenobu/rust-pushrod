// AccessKit conversion tests: the tree a platform adapter receives.  Run with
// `cargo test -p pushrod-widgets --features accesskit`.  Sample text is fictional.
#![cfg(feature = "accesskit")]

use accesskit::{Action, ActionData, ActionRequest, Role, TreeId};
use pushrod::accessibility::{AccessAction, from_request, node_id};
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::text_field::TextField;

#[test]
fn the_first_update_is_a_full_tree_with_a_root() {
    let mut engine: Engine = Engine::new(size(300, 200), 30);
    let check = engine.add_widget(CheckBox::new("Agree").at(10, 10));

    engine.set_accessibility_active(true);

    let update = engine
        .accessibility_update()
        .expect("update")
        .to_accesskit();

    assert!(update.tree.is_some());
    assert_eq!(update.tree_id, TreeId::ROOT);
    assert_eq!(
        update
            .nodes
            .iter()
            .find(|(id, _)| *id == node_id(0))
            .map(|(_, n)| n.role()),
        Some(Role::Window)
    );

    let (_, node) = update
        .nodes
        .iter()
        .find(|(id, _)| *id == node_id(check))
        .expect("check box");

    assert_eq!(node.role(), Role::CheckBox);
    assert_eq!(node.label(), Some("Agree"));
}

#[test]
fn a_later_update_converts_only_what_changed() {
    let mut engine: Engine = Engine::new(size(300, 200), 30);
    let a = engine.add_widget(CheckBox::new("A").at(10, 10));
    let b = engine.add_widget(CheckBox::new("B").at(10, 40));

    engine.set_accessibility_active(true);
    let _ = engine.accessibility_update();
    engine.widgets().set_focus(b);

    let update = engine
        .accessibility_update()
        .expect("update")
        .to_accesskit();

    assert!(update.tree.is_none());
    assert_eq!(update.focus, node_id(b));
    assert!(update.nodes.iter().all(|(id, _)| *id != node_id(a)));
}

#[test]
fn a_text_field_carries_a_text_run_and_selection() {
    let mut engine: Engine = Engine::new(size(300, 200), 30);
    let mut field = TextField::new().with_label("Name").at(10, 10).size(120, 24);

    field.set_text("héllo");

    let id = engine.add_widget(field);

    engine.set_accessibility_active(true);

    let update = engine
        .accessibility_update()
        .expect("update")
        .to_accesskit();
    let (_, node) = update
        .nodes
        .iter()
        .find(|(n, _)| *n == node_id(id))
        .expect("field");
    let selection = node.text_selection().expect("selection");
    let run = update
        .nodes
        .iter()
        .find(|(n, _)| *n == selection.focus.node)
        .expect("run");

    assert_eq!(run.1.role(), Role::TextRun);
    assert_eq!(run.1.value(), Some("héllo"));
    assert_eq!(
        run.1
            .character_lengths()
            .iter()
            .map(|&l| usize::from(l))
            .sum::<usize>(),
        "héllo".len()
    );
    assert_eq!(selection.focus.character_index, 5);
}

#[test]
fn requests_map_to_pushrod_actions() {
    let request = |action, data| ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node_id(7),
        data,
    };

    assert_eq!(
        from_request(&request(Action::Click, None)),
        Some((7, AccessAction::Click))
    );
    assert_eq!(
        from_request(&request(Action::Focus, None)),
        Some((7, AccessAction::Focus))
    );
    assert_eq!(
        from_request(&request(
            Action::SetValue,
            Some(ActionData::NumericValue(2.5))
        )),
        Some((7, AccessAction::SetValue(2.5)))
    );
    assert_eq!(
        from_request(&request(
            Action::SetValue,
            Some(ActionData::Value("Hi".into()))
        )),
        Some((7, AccessAction::SetText("Hi".into())))
    );
    assert_eq!(from_request(&request(Action::Collapse, None)), None);
}
