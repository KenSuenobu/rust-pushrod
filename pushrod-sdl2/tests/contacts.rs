//! Tests for the reference application, `examples/contacts` (issue #140).
//!
//! * The contact book is plain Rust and is tested directly.
//! * The window is built exactly as the example builds it, laid out with the headless backend,
//!   and driven with the keyboard and mouse.
//! * Its main screens are compared with golden images in `tests/golden/`, like the widgets in
//!   `visual.rs`: set `PUSHROD_GOLDEN=1` to compare and `UPDATE_GOLDEN=1` to rewrite.

#[path = "../examples/contacts/key_relay.rs"]
mod key_relay;
#[path = "../examples/contacts/model.rs"]
mod model;
mod support;
#[path = "../examples/contacts/window.rs"]
mod window;

use model::{Contact, ContactBook, EmailValidator, MAX_CONTACTS, Reach, RequiredValidator};
use pushrod::backend::headless::HeadlessBackend;
use pushrod::engine::Engine;
use pushrod::geometry::{Rect, size};
use pushrod::input::{InputEvent, Key, Modifiers};
use pushrod::layout::{ChildInfo, Layout, LayoutParams, SizeHint};
use pushrod::widget::{Widget, WidgetId};
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::radio_button::RadioButton;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;
use pushrod_widgets::text_field::validate::{Validator, Validity};
use support::golden::{check_golden, load_png};
use window::{App, Ids, Screen, WINDOW, WindowLayout};

/// The contacts example, built and laid out, with its state.
struct Harness {
    engine: Engine<App>,
    app: App,
    ids: Ids,
    backend: HeadlessBackend,
}

impl Harness {
    /// Builds the window around a contact book and lays it out.
    ///
    /// # Arguments
    ///
    /// * `book` - The contacts.
    ///
    /// # Returns
    ///
    /// The harness.
    fn with_book(book: ContactBook) -> Self {
        let app = App::new(book);
        let (mut engine, ids) = window::build(WINDOW, &app);
        let mut backend = HeadlessBackend::new(WINDOW);

        engine.widgets().draw_loop(&mut backend);

        Self {
            engine,
            app,
            ids,
            backend,
        }
    }

    /// Builds the window around the sample contacts.
    ///
    /// # Returns
    ///
    /// The harness, with Jane Doe selected.
    fn new() -> Self {
        Self::with_book(ContactBook::default())
    }

    /// Presses a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key.
    /// * `modifiers` - The modifier keys held.
    fn key_with(&mut self, key: Key, modifiers: Modifiers) {
        self.engine.dispatch(
            InputEvent::KeyDown {
                key,
                modifiers,
                repeat: false,
            },
            &mut self.app,
        );
    }

    /// Presses a key with no modifiers.
    ///
    /// # Arguments
    ///
    /// * `key` - The key.
    fn key(&mut self, key: Key) {
        self.key_with(key, Modifiers::empty());
    }

    /// Types text into the focused widget.
    ///
    /// # Arguments
    ///
    /// * `text` - What to type.
    fn type_text(&mut self, text: &str) {
        self.engine
            .dispatch(InputEvent::Text(text.to_string()), &mut self.app);
    }

    /// Focuses a text field and empties it, ready for typing.
    ///
    /// # Arguments
    ///
    /// * `id` - The field.
    fn clear_field(&mut self, id: WidgetId) {
        self.engine.widgets().set_focus(id);

        if let Some(field) = self.engine.widgets().get_as_mut::<TextField>(id) {
            field.set_text("");
        }
    }

    /// Clicks the middle of a widget.
    ///
    /// # Arguments
    ///
    /// * `id` - The widget.
    fn click(&mut self, id: WidgetId) {
        window::click(&mut self.engine, &mut self.app, id);
    }

    /// Lays the window out again and draws a frame.
    fn draw(&mut self) {
        self.engine.widgets().draw_loop(&mut self.backend);
    }

    /// Returns a text field's text.
    fn field(&mut self, id: WidgetId) -> String {
        self.engine
            .widgets()
            .get_as::<TextField>(id)
            .map(TextField::text)
            .unwrap_or_default()
    }

    /// Returns a `Text`'s text.
    fn label(&mut self, id: WidgetId) -> String {
        self.engine
            .widgets()
            .get_as::<Text>(id)
            .map(Text::get_text)
            .unwrap_or_default()
    }

    /// Returns whether a widget is enabled.
    fn enabled(&mut self, id: WidgetId) -> bool {
        self.engine
            .widgets()
            .get(id)
            .is_some_and(|w| w.is_enabled())
    }

    /// Returns whether a widget is hidden.
    fn hidden(&mut self, id: WidgetId) -> bool {
        self.engine.widgets().get(id).is_some_and(|w| w.is_hidden())
    }

    /// Returns the focused widget.
    fn focused(&mut self) -> Option<WidgetId> {
        self.engine.widgets().focused()
    }

    /// Returns whether a modal dialog (the delete alert) is open.
    fn dialog_open(&mut self) -> bool {
        self.engine.widgets().top_modal().is_some()
    }

    /// Returns the selected contact's name in the book.
    fn current_name(&self) -> String {
        self.app
            .book
            .current()
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------------------------
// The contact book.
// ---------------------------------------------------------------------------------------------

#[test]
fn the_sample_book_is_sorted_fictional_and_starts_on_jane_doe() {
    let book = ContactBook::default();
    let names: Vec<&str> = book.contacts.iter().map(|c| c.name.as_str()).collect();
    let mut sorted = names.clone();

    sorted.sort_unstable();

    assert_eq!(names, sorted);
    assert_eq!(book.contacts.len(), 14);
    assert!(book.contacts.len() <= MAX_CONTACTS);
    assert_eq!(
        book.current().map(|c| c.email.as_str()),
        Some("jane.doe@example.com")
    );
    assert!(book.contacts.iter().all(|c| c.check().is_ok()));
    assert!(book.contacts.iter().all(|c| {
        c.email.ends_with("@example.com")
            || c.email.ends_with("@example.org")
            || c.email.ends_with("@example.net")
    }));
}

#[test]
fn the_email_validator_judges_addresses_as_they_are_typed() {
    let v = EmailValidator;

    assert_eq!(v.validate(""), Validity::Valid);
    assert_eq!(v.validate("jane.doe@example.com"), Validity::Valid);
    assert_eq!(v.validate("jane"), Validity::Intermediate);
    assert_eq!(v.validate("jane@"), Validity::Intermediate);
    assert_eq!(v.validate("jane@example"), Validity::Intermediate);
    assert_eq!(v.validate("jane@example.c"), Validity::Intermediate);
    assert_eq!(v.validate("@example.com"), Validity::Intermediate);
    assert_eq!(v.validate("jane doe"), Validity::Invalid);
    assert_eq!(v.validate("jane@@example.com"), Validity::Invalid);
}

#[test]
fn a_required_field_may_be_emptied_but_not_saved_empty() {
    assert_eq!(RequiredValidator.validate("Jane"), Validity::Valid);
    assert_eq!(RequiredValidator.validate(""), Validity::Intermediate);
    assert_eq!(RequiredValidator.validate("   "), Validity::Intermediate);
}

#[test]
fn checking_a_contact_names_the_first_problem() {
    let good = Contact::new("Jane Doe", "jane.doe@example.com");

    assert_eq!(good.check(), Ok(()));
    assert_eq!(
        Contact::new("", "jane.doe@example.com").check(),
        Err("A name is required.")
    );
    assert_eq!(
        Contact::new("Jane Doe", "jane@").check(),
        Err("The email address is incomplete.")
    );
    assert_eq!(
        Contact {
            phone: "(555) 010".to_string(),
            ..good.clone()
        }
        .check(),
        Err("The phone number is incomplete.")
    );
    assert_eq!(
        Contact {
            phone: "(555) 010-0100".to_string(),
            ..good
        }
        .check(),
        Ok(())
    );
}

#[test]
fn favorites_are_starred_in_the_list() {
    let mut contact = Contact::new("Jane Doe", "");

    assert_eq!(contact.list_label(), "Jane Doe");
    contact.favorite = true;
    assert_eq!(contact.list_label(), "* Jane Doe");
}

#[test]
fn the_reach_choices_have_labels_in_order() {
    let labels: Vec<&str> = Reach::ALL.iter().map(|r| r.label()).collect();

    assert_eq!(labels, ["Email", "Phone", "Mail"]);
}

#[test]
fn saving_replaces_the_selected_contact_only_when_it_checks() {
    let mut book = ContactBook::default();
    let index = book.selected.unwrap();
    let edited = Contact {
        city: "Othertown".to_string(),
        ..book.current().unwrap().clone()
    };

    assert_eq!(book.save(edited), Ok(()));
    assert_eq!(book.contacts[index].city, "Othertown");
    assert_eq!(book.save(Contact::new("", "")), Err("A name is required."));
    assert_eq!(book.contacts[index].city, "Othertown");

    book.selected = None;
    assert_eq!(
        book.save(Contact::new("Jane Doe", "")),
        Err("No contact is selected.")
    );
}

#[test]
fn selecting_out_of_range_keeps_the_selection() {
    let mut book = ContactBook::default();
    let before = book.selected;

    assert!(!book.select(99));
    assert_eq!(book.selected, before);
    assert!(book.select(0));
    assert_eq!(book.selected, Some(0));
}

#[test]
fn new_contacts_are_added_until_the_book_is_full() {
    let mut book = ContactBook::default();

    while book.has_room() {
        let index = book.add_new().unwrap();

        assert_eq!(book.selected, Some(index));
        assert_eq!(book.contacts[index].name, "New contact");
    }

    assert_eq!(book.contacts.len(), MAX_CONTACTS);
    assert_eq!(book.add_new(), None);
}

#[test]
fn deleting_selects_the_next_contact_then_the_previous() {
    let mut book = ContactBook {
        contacts: vec![
            Contact::new("Alex Example", ""),
            Contact::new("Jane Doe", ""),
            Contact::new("John Doe", ""),
        ],
        selected: Some(1),
    };

    assert_eq!(
        book.delete_selected().map(|c| c.name),
        Some("Jane Doe".into())
    );
    assert_eq!(book.current().map(|c| c.name.as_str()), Some("John Doe"));
    assert_eq!(
        book.delete_selected().map(|c| c.name),
        Some("John Doe".into())
    );
    assert_eq!(
        book.current().map(|c| c.name.as_str()),
        Some("Alex Example")
    );
    assert!(book.delete_selected().is_some());
    assert_eq!(book.selected, None);
    assert_eq!(book.delete_selected(), None);
}

// ---------------------------------------------------------------------------------------------
// The window.
// ---------------------------------------------------------------------------------------------

#[test]
fn the_window_layout_fills_with_the_first_child_and_centers_the_rest() {
    let child = |id, w, h| ChildInfo {
        id,
        hint: SizeHint::fixed(size(w, h)),
        params: LayoutParams::default(),
    };
    let children = [child(1, 10, 10), child(2, 200, 100)];
    let rects = WindowLayout.arrange(&children, Rect::new(0, 0, 640, 440));

    assert_eq!(
        rects,
        vec![Rect::new(0, 0, 640, 440), Rect::new(220, 170, 200, 100)]
    );
    assert_eq!(
        WindowLayout.measure(&children, size(0, 0)),
        children[0].hint
    );
    assert_eq!(WindowLayout.measure(&[], size(0, 0)), SizeHint::default());
}

#[test]
fn the_form_shows_the_selected_contact() {
    let mut h = Harness::new();
    let ids = h.ids;

    assert_eq!(h.field(ids.name), "Jane Doe");
    assert_eq!(h.field(ids.email), "jane.doe@example.com");
    assert_eq!(h.field(ids.phone), "(555) 010-0100");
    assert_eq!(h.field(ids.city), "Anytown");
    assert!(
        h.engine
            .widgets()
            .get_as::<RadioButton>(ids.reach[0])
            .unwrap()
            .is_checked()
    );
    assert!(
        h.engine
            .widgets()
            .get_as::<CheckBox>(ids.favorite)
            .unwrap()
            .is_checked()
    );
    assert_eq!(
        h.engine
            .widgets()
            .get_as::<Slider>(ids.closeness)
            .unwrap()
            .get_value(),
        5.0
    );
    assert_eq!(h.label(ids.message), "");
    assert!(h.enabled(ids.save));
    assert_eq!(h.label(ids.window), "640 x 440 at 100%");
}

#[test]
fn the_list_has_one_row_per_contact_and_hides_the_rest() {
    let mut h = Harness::new();
    let ids = h.ids;
    let book = h.app.book.clone();

    for (index, &row) in ids.rows.iter().enumerate() {
        let radio = h.engine.widgets().get_as::<RadioButton>(row).unwrap();

        match book.contacts.get(index) {
            Some(contact) => {
                assert_eq!(radio.get_text(), contact.list_label());
                assert_eq!(radio.is_checked(), book.selected == Some(index));
                assert!(!radio.is_hidden());
            }
            None => assert!(radio.is_hidden()),
        }
    }
}

#[test]
fn clicking_a_row_shows_that_contact() {
    let mut h = Harness::new();
    let first = h.ids.rows[0];

    h.click(first);

    assert_eq!(h.app.book.selected, Some(0));
    assert_eq!(h.field(h.ids.name), "Alex Example");
    assert_eq!(h.field(h.ids.city), "Sampleville");
}

#[test]
fn tab_visits_the_list_the_form_and_the_buttons_in_order() {
    let mut h = Harness::new();
    let ids = h.ids;
    let selected_row = ids.rows[h.app.book.selected.unwrap()];
    let expected = [
        selected_row,
        ids.name,
        ids.email,
        ids.phone,
        ids.city,
        ids.reach[0],
        ids.favorite,
        ids.newsletter,
        ids.closeness,
        ids.new,
        ids.delete,
        ids.revert,
        ids.save,
    ];
    let mut visited = Vec::new();

    // One more stop than listed: the list's scrolling content takes the focus too, so the
    // arrow keys and Page Up / Page Down can scroll it.
    for _ in 0..=expected.len() {
        h.key(Key::Tab);
        visited.push(h.focused().unwrap());
    }

    let stops: Vec<WidgetId> = visited
        .iter()
        .copied()
        .filter(|id| expected.contains(id))
        .collect();

    assert_eq!(stops, expected, "Tab order (all stops: {visited:?})");
    assert_eq!(
        visited.len() - stops.len(),
        1,
        "one extra stop: {visited:?}"
    );

    // Shift-Tab goes back the same way.
    h.key_with(Key::Tab, Modifiers::SHIFT);
    assert_eq!(h.focused(), Some(ids.revert));
}

#[test]
fn the_arrow_keys_move_the_selection_through_the_list() {
    let mut h = Harness::new();
    let start = h.app.book.selected.unwrap();

    h.engine.widgets().set_focus(h.ids.rows[start]);
    h.key(Key::Down);

    assert_eq!(h.app.book.selected, Some(start + 1));
    assert_eq!(h.current_name(), "John Doe");
    assert_eq!(h.field(h.ids.name), "John Doe");

    h.key(Key::Up);
    assert_eq!(h.current_name(), "Jane Doe");
}

#[test]
fn an_incomplete_email_disables_save_until_it_is_finished() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.clear_field(ids.email);
    h.type_text("jane.doe@example");

    assert_eq!(h.label(ids.message), "The email address is incomplete.");
    assert!(!h.enabled(ids.save));

    // Return reaches the default button, which is disabled, so nothing is saved.
    h.key(Key::Return);
    assert_eq!(h.app.book.current().unwrap().email, "jane.doe@example.com");

    h.type_text(".org");
    assert_eq!(h.label(ids.message), "");
    assert!(h.enabled(ids.save));
}

#[test]
fn the_email_field_refuses_spaces() {
    let mut h = Harness::new();
    let email = h.ids.email;

    h.clear_field(email);
    h.type_text("jane doe");

    assert!(!h.field(email).contains(' '));
}

#[test]
fn an_empty_name_cannot_be_saved() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.clear_field(ids.name);
    h.type_text(" ");

    assert_eq!(h.label(ids.message), "A name is required.");
    assert!(!h.enabled(ids.save));
}

#[test]
fn return_saves_the_form_and_escape_reverts_it() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.clear_field(ids.city);
    h.type_text("Othertown");
    h.key(Key::Escape);

    assert_eq!(h.field(ids.city), "Anytown");
    assert_eq!(h.app.book.current().unwrap().city, "Anytown");
    assert_eq!(h.label(ids.status), "Changes reverted.");

    h.clear_field(ids.city);
    h.type_text("Othertown");
    h.key(Key::Return);

    assert_eq!(h.app.book.current().unwrap().city, "Othertown");
    assert_eq!(h.label(ids.status), "Saved Jane Doe.");
}

#[test]
fn every_control_is_saved_from_the_keyboard() {
    let mut h = Harness::new();
    let ids = h.ids;

    // Reach by: Email -> Phone with an arrow key.
    h.engine.widgets().set_focus(ids.reach[0]);
    h.key(Key::Right);
    // Favorite and Newsletter: Space toggles.
    h.engine.widgets().set_focus(ids.favorite);
    h.key(Key::Char(' '));
    h.engine.widgets().set_focus(ids.newsletter);
    h.key(Key::Char(' '));
    // Closeness: 5 -> 4.
    h.engine.widgets().set_focus(ids.closeness);
    h.key(Key::Left);
    // Save.
    h.engine.widgets().set_focus(ids.save);
    h.key(Key::Char(' '));

    let saved = h.app.book.current().unwrap().clone();

    assert_eq!(saved.reach, Reach::Phone);
    assert!(!saved.favorite);
    assert!(!saved.newsletter);
    assert_eq!(saved.closeness, 4);

    let row = ids.rows[h.app.book.selected.unwrap()];

    assert_eq!(
        h.engine
            .widgets()
            .get_as::<RadioButton>(row)
            .unwrap()
            .get_text(),
        "Jane Doe"
    );
}

#[test]
fn delete_asks_first_and_cancel_keeps_the_contact() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.engine.widgets().set_focus(ids.delete);
    h.key(Key::Char(' '));
    assert!(h.dialog_open());

    // While the alert is open the window is inert: Tab stays inside the alert.
    h.key(Key::Tab);
    assert!(h.focused().is_none_or(|f| !ids.rows.contains(&f)));

    h.key(Key::Escape);

    assert!(!h.dialog_open());
    assert_eq!(h.app.book.contacts.len(), 14);
    assert_eq!(h.current_name(), "Jane Doe");
    assert_eq!(h.label(ids.status), "Nothing was deleted.");
}

#[test]
fn delete_confirmed_with_return_removes_the_contact() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.engine.widgets().set_focus(ids.delete);
    h.key(Key::Char(' '));
    h.key(Key::Return);

    assert!(!h.dialog_open());
    assert_eq!(h.app.book.contacts.len(), 13);
    assert_eq!(h.current_name(), "John Doe");
    assert_eq!(h.field(ids.name), "John Doe");
    assert_eq!(h.label(ids.status), "Deleted Jane Doe.");
    assert!(h.hidden(ids.rows[13]));
}

#[test]
fn new_adds_a_contact_and_focuses_its_name() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.engine.widgets().set_focus(ids.new);
    h.key(Key::Char(' '));

    assert_eq!(h.app.book.contacts.len(), 15);
    assert_eq!(h.field(ids.name), "New contact");
    assert_eq!(h.focused(), Some(ids.name));
    assert!(!h.hidden(ids.rows[14]));
    assert_eq!(h.label(ids.status), "Added a new contact.");

    // The last free row: the book is full and New is disabled.
    h.engine.widgets().set_focus(ids.new);
    h.key(Key::Char(' '));
    assert_eq!(h.app.book.contacts.len(), MAX_CONTACTS);
    assert!(!h.enabled(ids.new));
}

#[test]
fn an_empty_book_disables_the_form() {
    let mut h = Harness::with_book(ContactBook {
        contacts: Vec::new(),
        selected: None,
    });
    let ids = h.ids;

    assert!(!h.enabled(ids.name));
    assert!(!h.enabled(ids.save));
    assert!(!h.enabled(ids.delete));
    assert!(!h.enabled(ids.revert));
    assert!(h.enabled(ids.new));
    assert_eq!(h.label(ids.message), "No contact is selected.");
    assert!(ids.rows.iter().all(|&row| h.hidden(row)));
}

#[test]
fn the_window_line_follows_resizing_and_the_scale_factor() {
    let mut h = Harness::new();
    let ids = h.ids;

    h.engine
        .dispatch(InputEvent::Resized(size(800, 520)), &mut h.app);
    h.draw();
    h.engine.dispatch(InputEvent::ScaleChanged(2.0), &mut h.app);
    h.draw();

    assert_eq!(h.app.scale_percent, 200);
    assert_eq!(h.label(ids.window), "800 x 520 at 200%");

    // The fields grew with the window.
    let wide = h.engine.widgets().get(ids.name).unwrap().get_size().w;

    h.engine.dispatch(InputEvent::Resized(WINDOW), &mut h.app);
    h.draw();

    let narrow = h.engine.widgets().get(ids.name).unwrap().get_size().w;

    assert!(wide > narrow, "{wide} should be wider than {narrow}");
}

#[test]
fn screens_have_distinct_names_and_words() {
    let names: Vec<&str> = Screen::ALL.iter().map(|s| s.name()).collect();

    assert_eq!(
        names,
        [
            "contacts-main",
            "contacts-invalid",
            "contacts-confirm-delete",
            "contacts-resized"
        ]
    );
    assert_eq!(Screen::from_word(None), Screen::Main);
    assert_eq!(Screen::from_word(Some("invalid")), Screen::Invalid);
    assert_eq!(Screen::from_word(Some("delete")), Screen::ConfirmDelete);
    assert_eq!(Screen::from_word(Some("resized")), Screen::Resized);
    assert_eq!(Screen::from_word(Some("other")), Screen::Main);
    assert!(Screen::Resized.window().w > WINDOW.w);
}

#[test]
fn acting_out_each_screen_leaves_the_window_in_that_state() {
    for screen in Screen::ALL {
        let mut h = Harness::new();
        let ids = h.ids;

        window::act_out(&mut h.engine, &mut h.app, ids, screen);

        assert_eq!(
            h.dialog_open(),
            screen == Screen::ConfirmDelete,
            "{screen:?}"
        );
        assert_eq!(h.enabled(ids.save), screen != Screen::Invalid, "{screen:?}");
    }
}

// ---------------------------------------------------------------------------------------------
// Golden images.
// ---------------------------------------------------------------------------------------------

#[test]
fn the_main_screens_match_their_golden_images() {
    // Text antialiasing differs between platforms, so the comparison runs where the goldens
    // were made (the `golden` CI job) and wherever it is asked for, as in `visual.rs`.
    if std::env::var_os("PUSHROD_GOLDEN").is_none() && std::env::var_os("UPDATE_GOLDEN").is_none() {
        eprintln!(
            "skipped: set PUSHROD_GOLDEN=1 to compare the contacts screens with tests/golden"
        );

        return;
    }

    // SAFETY: set before this test starts SDL; no other test in this binary reads them.
    unsafe {
        if std::env::var_os("SDL_VIDEODRIVER").is_none() {
            std::env::set_var("SDL_VIDEODRIVER", "dummy");
        }

        // The software renderer is deterministic, like the one `visual.rs` uses.
        if std::env::var_os("SDL_RENDER_DRIVER").is_none() {
            std::env::set_var("SDL_RENDER_DRIVER", "software");
        }
    }

    let dir = std::env::temp_dir().join(format!("pushrod-contacts-{}", std::process::id()));
    let mut failures = Vec::new();

    std::fs::create_dir_all(&dir).unwrap();

    for screen in Screen::ALL {
        let path = dir.join(format!("{}.png", screen.name()));

        window::render_screen(screen, path.to_str().unwrap()).unwrap();

        let pixels = load_png(&path).unwrap();

        failures.extend(check_golden(screen.name(), &pixels));
    }

    let _ = std::fs::remove_dir_all(&dir);

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
