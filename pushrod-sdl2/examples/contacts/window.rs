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

//! The contact editor's window, built in code (MVP v0.1).
//!
//! [`build`] makes every `Widget` and callback; the helpers below copy between the
//! [`ContactBook`] and the widgets, and work on a [`WidgetCache`] so they run the same while
//! the window is being built and inside callbacks.  [`render_screen`] draws one of the
//! [`Screen`]s into an image, for the documentation and the golden-image tests.

use crate::key_relay::KeyRelay;
use crate::model::{
    Contact, ContactBook, EmailValidator, MAX_CONTACTS, PHONE_MASK, Reach, RequiredValidator,
};
use pushrod::backend::Backend;
use pushrod::cache::WidgetCache;
use pushrod::color::{Color, FontStyle};
use pushrod::composite::TypedParts;
use pushrod::engine::{Engine, Ui};
use pushrod::event::EngineEvent;
use pushrod::font::Font;
use pushrod::geometry::{Point, Rect, Size, point, size};
use pushrod::input::{InputEvent, Key, MouseButton};
use pushrod::layout::{
    Align, Alignment, ChildInfo, Column, FormLayout, Layout, LayoutParams, Margin, Row, SizeHint,
    Spacer,
};
use pushrod::paint::TextJustify;
use pushrod::widget::{Behavior, ROOT_WIDGET_ID, Widget, WidgetId};
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::alert::{AlertExt, AlertKind};
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::container::{Container, ContentSize};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::radio_button::RadioButton;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The window's starting size, in logical pixels.
pub const WINDOW: Size = Size { w: 640, h: 440 };

/// The window's title.
pub const TITLE: &str = "pushrod: contacts";

/// The list's width, in pixels.
const LIST_W: u32 = 200;

/// The form's starting width, in pixels; the layout widens it to fill the window.
const FORM_W: u32 = 400;

/// The list's background, which its rows share.
const LIST_COLOR: Color = Color::RGBA(240, 240, 240, 255);

/// The color of the validation message.
const ERROR_COLOR: Color = Color::RGBA(170, 20, 20, 255);

/// The engine's application state: the contact book, and what the window line shows.
#[derive(Debug, Clone)]
pub struct App {
    /// The contacts.
    pub book: ContactBook,
    /// The display's scale factor, in percent; `100` until the backend reports another.
    pub scale_percent: u32,
    /// The window line's current text, so it is only rewritten when it changes.
    pub window_line: String,
}

impl App {
    /// Creates the application state around a contact book.
    ///
    /// # Arguments
    ///
    /// * `book` - The contacts to edit.
    ///
    /// # Returns
    ///
    /// The state, at a scale of 100%.
    pub fn new(book: ContactBook) -> Self {
        Self {
            book,
            scale_percent: 100,
            window_line: String::new(),
        }
    }
}

/// The IDs of every `Widget` the application reads or changes.
#[derive(Debug, Clone, Copy)]
pub struct Ids {
    /// The scrolling list.
    pub list: WidgetId,
    /// One radio button per list row, top to bottom; rows past the last contact are hidden.
    pub rows: [WidgetId; MAX_CONTACTS],
    /// The name field.
    pub name: WidgetId,
    /// The e-mail field.
    pub email: WidgetId,
    /// The telephone field.
    pub phone: WidgetId,
    /// The city field.
    pub city: WidgetId,
    /// One radio button per [`Reach`], in [`Reach::ALL`] order.
    pub reach: [WidgetId; 3],
    /// The **Favorite** check box.
    pub favorite: WidgetId,
    /// The **Newsletter** check box.
    pub newsletter: WidgetId,
    /// The closeness slider.
    pub closeness: WidgetId,
    /// The validation message under the form.
    pub message: WidgetId,
    /// **New**.
    pub new: WidgetId,
    /// **Delete...**.
    pub delete: WidgetId,
    /// **Revert** (the cancel button: Escape).
    pub revert: WidgetId,
    /// **Save** (the default button: Return).
    pub save: WidgetId,
    /// The status line: the last thing that happened.
    pub status: WidgetId,
    /// The window line: size and scale factor.
    pub window: WidgetId,
}

/// The window root's layout: the first child (the main panel) fills the window, and every later
/// child (an alert opened with `ui.alert`, which is added to the root) is centered at its
/// preferred size, so a dialog is never arranged as part of the form.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowLayout;

impl Layout for WindowLayout {
    /// Reports the main panel's needs as the window's.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children; the first is the main panel.
    /// * `_available` - The window's size; not used.
    ///
    /// # Returns
    ///
    /// The main panel's hint, or a zero hint with no children.
    fn measure(&self, children: &[ChildInfo], _available: Size) -> SizeHint {
        children
            .first()
            .map_or_else(SizeHint::default, |main| main.hint)
    }

    /// Fills the window with the main panel and centers everything else.
    ///
    /// # Arguments
    ///
    /// * `children` - The visible children; the first is the main panel.
    /// * `bounds` - The window's area.
    ///
    /// # Returns
    ///
    /// One rectangle per child.
    fn arrange(&self, children: &[ChildInfo], bounds: Rect) -> Vec<Rect> {
        children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                if index == 0 {
                    return bounds;
                }

                let Size { w, h } = child.hint.preferred;

                Rect::new(
                    bounds.x + (bounds.w as i32 - w as i32) / 2,
                    bounds.y + (bounds.h as i32 - h as i32) / 2,
                    w,
                    h,
                )
            })
            .collect()
    }
}

/// Describes the window for the lower right corner.
///
/// # Arguments
///
/// * `window` - The window's size, in logical pixels.
/// * `scale_percent` - The display's scale factor, in percent.
///
/// # Returns
///
/// For example `640 x 440 at 200%`.
pub fn window_line(window: Size, scale_percent: u32) -> String {
    format!("{} x {} at {}%", window.w, window.h, scale_percent)
}

/// Layout parameters for a child that fills its slot across and keeps its height.
///
/// # Returns
///
/// Horizontal fill, top alignment, no stretch.
fn fill_width() -> LayoutParams {
    LayoutParams {
        alignment: Alignment::new(Align::Fill, Align::Start),
        ..LayoutParams::default()
    }
}

/// Layout parameters for a child that takes a share of the extra space and fills its slot.
///
/// # Arguments
///
/// * `stretch` - Its share of the extra space; `0` keeps its preferred size along the axis.
///
/// # Returns
///
/// Fill in both directions, with that stretch.
fn fill(stretch: u32) -> LayoutParams {
    LayoutParams {
        stretch,
        alignment: Alignment::new(Align::Fill, Align::Fill),
        ..LayoutParams::default()
    }
}

/// Adds a `Widget` to a parent and gives it layout parameters.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `parent` - The parent's ID.
/// * `widget` - The widget.
/// * `params` - How it sits in the parent's layout.
///
/// # Returns
///
/// The widget's ID.
fn add<W: Widget>(
    engine: &mut Engine<App>,
    parent: WidgetId,
    widget: W,
    params: LayoutParams,
) -> WidgetId {
    let id = engine.add_child_widget(parent, widget);

    if let Some(widget) = engine.widgets().get_mut(id) {
        widget.set_layout_params(params);
    }

    id
}

/// Adds an empty panel that arranges its children with a layout.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `parent` - The parent's ID.
/// * `layout` - The panel's layout.
/// * `params` - How the panel sits in the parent's layout.
///
/// # Returns
///
/// The panel's ID.
fn add_panel(
    engine: &mut Engine<App>,
    parent: WidgetId,
    layout: Box<dyn Layout>,
    params: LayoutParams,
) -> WidgetId {
    let id = add(engine, parent, Panel::new().size(0, 0), params);

    if let Some(widget) = engine.widgets().get_mut(id) {
        widget.set_layout(layout);
    }

    id
}

/// Adds the scrolling list of contacts: a `Container` whose content is a column of radio
/// buttons, so Tab visits the list once and the arrow keys move the selection.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `parent` - The panel the list sits in.
///
/// # Returns
///
/// The container's ID and the rows' IDs.
fn add_list(engine: &mut Engine<App>, parent: WidgetId) -> (WidgetId, [WidgetId; MAX_CONTACTS]) {
    let list = add(
        engine,
        parent,
        Container::new(point(0, 0), size(LIST_W, 200), size(0, 0))
            .content_mode(ContentSize::FollowWidth { fit_height: true }),
        LayoutParams {
            alignment: Alignment::new(Align::Start, Align::Fill),
            ..LayoutParams::default()
        },
    );
    let content = engine
        .widgets()
        .get_as::<Container>(list)
        .map_or(list, |c| c.typed_parts().content);

    // The list scrolls, so it needs only a little height, and takes all it is given.
    if let Some(widget) = engine.widgets().get_mut(list) {
        widget.set_size_hint(SizeHint::new(
            size(LIST_W, 80),
            size(LIST_W, 200),
            size(LIST_W, u32::MAX),
        ));
    }

    if let Some(widget) = engine.widgets().get_mut(content) {
        widget.set_color(LIST_COLOR);
        widget.set_layout(Box::new(Column::new().spacing(2).margins(Margin::all(4))));
    }

    let mut rows = [0; MAX_CONTACTS];

    for row in &mut rows {
        let mut radio = RadioButton::new("").size(170, 22);

        radio.set_color(LIST_COLOR);
        *row = add(engine, content, radio, fill_width());
    }

    (list, rows)
}

/// Adds one labelled row to the form.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `form` - The form panel.
/// * `layout` - The form's layout, which records the row.
/// * `label` - The label's text, such as `"Name:"`.
/// * `field` - The field's ID (already a child of `form`).
fn add_row(
    engine: &mut Engine<App>,
    form: WidgetId,
    layout: &mut FormLayout,
    label: &str,
    field: WidgetId,
) {
    let label = engine.add_child_widget(form, Text::new(label).size(90, 22));

    layout.add_row(label, field);
}

/// Adds a text field to the form.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `form` - The form panel.
/// * `field` - The field, with its mask or validator.
///
/// # Returns
///
/// The field's ID.
fn add_field(engine: &mut Engine<App>, form: WidgetId, field: TextField) -> WidgetId {
    engine.add_child_widget(form, field.size(220, 24))
}

/// Builds the window: the list on the left, the form on the right, buttons and a status line
/// underneath.  Every container uses a layout, so the window can be resized.
///
/// # Arguments
///
/// * `window` - The window's size, in logical pixels.
/// * `app` - The state to show first.
///
/// # Returns
///
/// The engine, with every callback registered, and the IDs of the widgets it changes.
pub fn build(window: Size, app: &App) -> (Engine<App>, Ids) {
    let mut engine: Engine<App> = Engine::new(window, 30);

    if let Some(root) = engine.widgets().get_mut(ROOT_WIDGET_ID) {
        root.set_layout(Box::new(WindowLayout));
    }

    // Everything but dialogs is in one panel that fills the window.
    let main = add_panel(
        &mut engine,
        ROOT_WIDGET_ID,
        Box::new(Column::new().spacing(8).margins(Margin::all(12))),
        LayoutParams::default(),
    );

    add(
        &mut engine,
        main,
        Text::new("Contacts")
            .font(Font::new("", 16, FontStyle::BOLD))
            .size(200, 24),
        fill_width(),
    );

    let body = add_panel(&mut engine, main, Box::new(Row::new().spacing(12)), fill(1));

    // Built in tab order: the list, the fields, then the buttons (Tab follows `WidgetId`s).
    let (list, rows) = add_list(&mut engine, body);
    // A form measures itself at its current width, and stacks its labels above the fields when
    // that is narrow, so it starts at a width that fits labels and fields side by side.
    let form = add(&mut engine, body, Panel::new().size(FORM_W, 300), fill(1));
    let mut layout = FormLayout::new().spacing(8).column_spacing(8);

    let name = add_field(
        &mut engine,
        form,
        TextField::new().with_validator(RequiredValidator),
    );
    let email = add_field(
        &mut engine,
        form,
        TextField::new().with_validator(EmailValidator),
    );
    let phone = add_field(&mut engine, form, TextField::new().with_mask(PHONE_MASK));
    let city = add_field(&mut engine, form, TextField::new().max_length(40));

    // The radio buttons share a panel of their own, so they form one group apart from the list.
    let reach_panel = engine.add_child_widget(form, Panel::new().size(0, 0));

    if let Some(widget) = engine.widgets().get_mut(reach_panel) {
        widget.set_layout(Box::new(Row::new().spacing(8)));
    }

    let reach = Reach::ALL
        .map(|r| engine.add_child_widget(reach_panel, RadioButton::new(r.label()).size(76, 22)));
    let favorite = engine.add_child_widget(form, CheckBox::new("Favorite").size(200, 22));
    let newsletter =
        engine.add_child_widget(form, CheckBox::new("Send the newsletter").size(200, 22));
    let closeness = engine.add_child_widget(
        form,
        Slider::new(0.0, 5.0)
            .step(1.0)
            .ticks(1.0)
            .show_value(true)
            .size(220, 40),
    );
    let message =
        engine.add_child_widget(form, Text::new("").font_color(ERROR_COLOR).size(220, 20));

    add_row(&mut engine, form, &mut layout, "Name:", name);
    add_row(&mut engine, form, &mut layout, "Email:", email);
    add_row(&mut engine, form, &mut layout, "Phone:", phone);
    add_row(&mut engine, form, &mut layout, "City:", city);
    add_row(&mut engine, form, &mut layout, "Reach by:", reach_panel);
    add_row(&mut engine, form, &mut layout, "Options:", favorite);
    add_row(&mut engine, form, &mut layout, "", newsletter);
    add_row(&mut engine, form, &mut layout, "Closeness:", closeness);
    layout.add_full_row(message);

    if let Some(widget) = engine.widgets().get_mut(form) {
        widget.set_layout(Box::new(layout));
    }

    // The button bar passes key presses on, so Space can activate the focused button.
    let buttons = add(&mut engine, main, KeyRelay::new(), fill_width());

    if let Some(widget) = engine.widgets().get_mut(buttons) {
        widget.set_layout(Box::new(Row::new().spacing(8)));
    }

    let new = engine.add_child_widget(buttons, Button::new("New").size(80, 24));
    let delete = engine.add_child_widget(buttons, Button::new("Delete...").size(90, 24));

    engine.add_child_widget(buttons, Spacer::stretch());

    let revert =
        engine.add_child_widget(buttons, Button::new("Revert").size(80, 24).cancel_action());
    let save = engine.add_child_widget(buttons, Button::new("Save").size(80, 24).default_action());

    let footer = add_panel(&mut engine, main, Box::new(Row::new()), fill_width());
    let status = add(
        &mut engine,
        footer,
        Text::new("Ready.").size(300, 20),
        fill(1),
    );
    let window_info = engine.add_child_widget(
        footer,
        Text::new(window_line(window, app.scale_percent))
            .justify(TextJustify::Right)
            .size(200, 20),
    );

    let ids = Ids {
        list,
        rows,
        name,
        email,
        phone,
        city,
        reach,
        favorite,
        newsletter,
        closeness,
        message,
        new,
        delete,
        revert,
        save,
        status,
        window: window_info,
    };

    show_selected(engine.widgets(), ids, &app.book);
    connect(&mut engine, ids);

    (engine, ids)
}

/// Registers every callback.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
/// * `ids` - The widgets the callbacks change.
fn connect(engine: &mut Engine<App>, ids: Ids) {
    // A row is checked by a click, by Space, or by an arrow key within the list.
    for (index, &row) in ids.rows.iter().enumerate() {
        engine.on_toggle(row, move |app, ui, checked| {
            if checked && app.book.select(index) {
                show_selected(ui.widgets(), ids, &app.book);
            }
        });
    }

    for field in [ids.name, ids.email, ids.phone] {
        engine.on_text_change(field, move |_app, ui| {
            refresh_controls(ui.widgets(), ids);
        });
    }

    // Buttons do not take the keyboard focus by default; these four do, so Tab reaches them
    // and Space activates the focused one.  Return and Escape still mean Save and Revert.
    let actions: [(WidgetId, Action); 4] = [
        (ids.new, add_contact),
        (ids.delete, confirm_delete),
        (ids.revert, revert),
        (ids.save, save),
    ];

    for (button, action) in actions {
        if let Some(widget) = engine.widgets().get_mut(button) {
            widget.set_behavior(widget.behavior() | Behavior::FOCUSABLE);
        }

        engine.on_click(button, move |app, ui| action(app, ui, ids));
        engine.on_key_press(button, move |app, ui, key, _modifiers| {
            if key == Key::Char(' ') {
                action(app, ui, ids);
            }
        });
    }

    engine.on_engine_event(move |app, ui, event| {
        match event {
            EngineEvent::ScaleFactorChanged { new_percent, .. } => {
                app.scale_percent = *new_percent;
            }
            EngineEvent::LayoutChanged { container } if *container == ROOT_WIDGET_ID => {}
            _ => return,
        }

        let window = ui
            .widgets()
            .get(ROOT_WIDGET_ID)
            .map_or(WINDOW, |root| root.get_size());
        let line = window_line(window, app.scale_percent);

        if line != app.window_line {
            set_text(ui.widgets(), ids.window, line.clone());
            app.window_line = line;
        }
    });
}

/// What a button does: called with the application state, the callback's `Ui`, and the IDs.
type Action = fn(&mut App, &mut Ui<App>, Ids);

/// **Save**: checks the form and copies it into the selected contact.
///
/// # Arguments
///
/// * `app` - The application state.
/// * `ui` - The callback's `Ui`.
/// * `ids` - The application's widgets.
fn save(app: &mut App, ui: &mut Ui<App>, ids: Ids) {
    let contact = read_form(ui.widgets(), ids);
    let name = contact.name.clone();
    let line = match app.book.save(contact) {
        Ok(()) => {
            show_selected(ui.widgets(), ids, &app.book);
            format!("Saved {name}.")
        }
        Err(message) => message.to_string(),
    };

    set_text(ui.widgets(), ids.status, line);
}

/// **Revert**: shows the selected contact as it was last saved.
///
/// # Arguments
///
/// * `app` - The application state.
/// * `ui` - The callback's `Ui`.
/// * `ids` - The application's widgets.
fn revert(app: &mut App, ui: &mut Ui<App>, ids: Ids) {
    show_selected(ui.widgets(), ids, &app.book);
    set_text(ui.widgets(), ids.status, "Changes reverted.".to_string());
}

/// **New**: adds a contact, selects it, and puts the keyboard in its name.
///
/// # Arguments
///
/// * `app` - The application state.
/// * `ui` - The callback's `Ui`.
/// * `ids` - The application's widgets.
fn add_contact(app: &mut App, ui: &mut Ui<App>, ids: Ids) {
    if app.book.add_new().is_some() {
        show_selected(ui.widgets(), ids, &app.book);
        ui.set_focus(ids.name);
        set_text(ui.widgets(), ids.status, "Added a new contact.".to_string());
    }
}

/// Opens the alert that confirms deleting the selected contact.  **Delete** (Return) deletes
/// it; **Cancel** (Escape) keeps it.
///
/// # Arguments
///
/// * `app` - The application state.
/// * `ui` - The callback's `Ui`.
/// * `ids` - The widgets to update afterward.
fn confirm_delete(app: &mut App, ui: &mut Ui<App>, ids: Ids) {
    let Some(contact) = app.book.current() else {
        return;
    };

    ui.alert(
        AlertKind::Warning,
        &format!("Delete \"{}\"?", contact.name),
        "The contact card is removed from the list. This cannot be undone.",
        &["Delete", "Cancel"],
        0,
        move |app, ui, choice| {
            let removed = if choice == 0 {
                app.book.delete_selected()
            } else {
                None
            };
            let line = removed.map_or("Nothing was deleted.".to_string(), |contact| {
                format!("Deleted {}.", contact.name)
            });

            show_selected(ui.widgets(), ids, &app.book);
            set_text(ui.widgets(), ids.status, line);
        },
    );
}

/// Replaces a `Text`'s text.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `id` - The `Text`'s ID.
/// * `text` - The new text.
fn set_text(cache: &mut WidgetCache, id: WidgetId, text: String) {
    if let Some(label) = cache.get_as_mut::<Text>(id) {
        label.set_text(text);
    }
}

/// Shows the selected contact: relabels the list, checks its row, fills the form, and updates
/// the buttons.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `ids` - The application's widgets.
/// * `book` - The contacts.
pub fn show_selected(cache: &mut WidgetCache, ids: Ids, book: &ContactBook) {
    refresh_list(cache, ids, book);
    load_form(cache, ids, book.current());

    if let Some(button) = cache.get_mut(ids.new) {
        button.set_enabled(book.has_room());
    }

    if let Some(button) = cache.get_mut(ids.delete) {
        button.set_enabled(book.current().is_some());
    }

    refresh_controls(cache, ids);
}

/// Relabels the list's rows from the book, hides the unused ones, and checks the selected one.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `ids` - The application's widgets.
/// * `book` - The contacts.
fn refresh_list(cache: &mut WidgetCache, ids: Ids, book: &ContactBook) {
    for (index, &row) in ids.rows.iter().enumerate() {
        let contact = book.contacts.get(index);

        if let Some(radio) = cache.get_as_mut::<RadioButton>(row) {
            radio.set_text(contact.map(Contact::list_label).unwrap_or_default());
            radio.set_checked(book.selected == Some(index));
            radio.set_hidden(contact.is_none());
        }
    }

    // Hidden rows take no space, so the list's content is measured again.
    let content = cache
        .get_as::<Container>(ids.list)
        .map(|c| c.typed_parts().content);

    if let Some(content) = content {
        cache.invalidate_layout(content);
    }
}

/// Fills the form from a contact, or empties and disables it.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `ids` - The application's widgets.
/// * `contact` - The contact to show; `None` when the book is empty.
fn load_form(cache: &mut WidgetCache, ids: Ids, contact: Option<&Contact>) {
    let blank = Contact::default();
    let shown = contact.unwrap_or(&blank);
    let fields = [
        (ids.name, &shown.name),
        (ids.email, &shown.email),
        (ids.phone, &shown.phone),
        (ids.city, &shown.city),
    ];

    for (id, text) in fields {
        if let Some(field) = cache.get_as_mut::<TextField>(id) {
            field.set_text(text);
        }
    }

    for (reach, &id) in Reach::ALL.iter().zip(&ids.reach) {
        if let Some(radio) = cache.get_as_mut::<RadioButton>(id) {
            radio.set_checked(*reach == shown.reach);
        }
    }

    if let Some(check) = cache.get_as_mut::<CheckBox>(ids.favorite) {
        check.set_checked(shown.favorite);
    }

    if let Some(check) = cache.get_as_mut::<CheckBox>(ids.newsletter) {
        check.set_checked(shown.newsletter);
    }

    if let Some(slider) = cache.get_as_mut::<Slider>(ids.closeness) {
        slider.set_value(f64::from(shown.closeness));
    }

    let editable = contact.is_some();
    let controls = [
        ids.name,
        ids.email,
        ids.phone,
        ids.city,
        ids.favorite,
        ids.newsletter,
    ]
    .into_iter()
    .chain(ids.reach)
    .chain([ids.closeness, ids.revert]);

    for id in controls {
        if let Some(widget) = cache.get_mut(id) {
            widget.set_enabled(editable);
        }
    }
}

/// Reads the form back into a contact.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `ids` - The application's widgets.
///
/// # Returns
///
/// The contact as the form shows it.
pub fn read_form(cache: &WidgetCache, ids: Ids) -> Contact {
    let text = |id| {
        cache
            .get_as::<TextField>(id)
            .map(TextField::text)
            .unwrap_or_default()
    };
    let checked = |id| {
        cache
            .get_as::<CheckBox>(id)
            .is_some_and(CheckBox::is_checked)
    };
    let reach = Reach::ALL
        .iter()
        .zip(&ids.reach)
        .find(|(_, id)| {
            cache
                .get_as::<RadioButton>(**id)
                .is_some_and(RadioButton::is_checked)
        })
        .map_or(Reach::default(), |(reach, _)| *reach);
    let closeness = cache
        .get_as::<Slider>(ids.closeness)
        .map_or(0.0, Slider::get_value);

    Contact {
        name: text(ids.name),
        email: text(ids.email),
        phone: text(ids.phone),
        city: text(ids.city),
        reach,
        favorite: checked(ids.favorite),
        newsletter: checked(ids.newsletter),
        closeness: closeness.round().clamp(0.0, 5.0) as u8,
    }
}

/// Checks the form as it is typed: shows what is missing and enables **Save** only when the
/// contact could be saved.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `ids` - The application's widgets.
pub fn refresh_controls(cache: &mut WidgetCache, ids: Ids) {
    let editable = cache.get(ids.name).is_some_and(|w| w.is_enabled());
    let problem = if editable {
        read_form(cache, ids).check().err().unwrap_or("")
    } else {
        "No contact is selected."
    };

    set_text(cache, ids.message, problem.to_string());

    if let Some(button) = cache.get_mut(ids.save) {
        button.set_enabled(problem.is_empty());
    }
}

/// The screens shown in the documentation and covered by golden images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Jane Doe selected, at the starting size.
    Main,
    /// An incomplete e-mail address typed: the message shows and **Save** is disabled.
    Invalid,
    /// **Delete...** clicked: the confirmation alert over the window.
    ConfirmDelete,
    /// The same window made wider and taller: the fields and the list stretch.
    Resized,
}

impl Screen {
    /// Every screen, in the order the documentation shows them.
    pub const ALL: [Screen; 4] = [
        Screen::Main,
        Screen::Invalid,
        Screen::ConfirmDelete,
        Screen::Resized,
    ];

    /// Returns the screen's name, used for its image file.
    ///
    /// # Returns
    ///
    /// For example `"contacts-main"`.
    pub fn name(self) -> &'static str {
        match self {
            Screen::Main => "contacts-main",
            Screen::Invalid => "contacts-invalid",
            Screen::ConfirmDelete => "contacts-confirm-delete",
            Screen::Resized => "contacts-resized",
        }
    }

    /// Finds a screen from the word given after `--screenshot <file>`.
    ///
    /// # Arguments
    ///
    /// * `word` - `"invalid"`, `"delete"`, or `"resized"`; anything else is the main screen.
    ///
    /// # Returns
    ///
    /// The screen.
    pub fn from_word(word: Option<&str>) -> Self {
        match word {
            Some("invalid") => Screen::Invalid,
            Some("delete") => Screen::ConfirmDelete,
            Some("resized") => Screen::Resized,
            _ => Screen::Main,
        }
    }

    /// Returns the window size the screen is drawn at.
    ///
    /// # Returns
    ///
    /// [`WINDOW`], or a larger size for [`Screen::Resized`].
    pub fn window(self) -> Size {
        match self {
            Screen::Resized => size(800, 520),
            _ => WINDOW,
        }
    }
}

/// Clicks the left mouse button in the middle of a `Widget`.
///
/// # Arguments
///
/// * `engine` - The application, laid out.
/// * `app` - The application state.
/// * `id` - The `Widget` to click.
pub fn click(engine: &mut Engine<App>, app: &mut App, id: WidgetId) {
    let origin = engine.widgets().absolute_origin(id);
    let bounds = engine
        .widgets()
        .get(id)
        .map_or(size(0, 0), |w| w.get_size());
    let position = Point::new(
        origin.x + bounds.w as i32 / 2,
        origin.y + bounds.h as i32 / 2,
    );

    for down in [true, false] {
        let event = if down {
            InputEvent::MouseDown {
                position,
                button: MouseButton::Left,
                clicks: 1,
            }
        } else {
            InputEvent::MouseUp {
                position,
                button: MouseButton::Left,
                clicks: 1,
            }
        };

        engine.dispatch(event, app);
    }
}

/// Puts the application into a screen's state, as a user would: by clicking and typing.
///
/// # Arguments
///
/// * `engine` - The application, laid out (drawn once).
/// * `app` - The application state.
/// * `ids` - The application's widgets.
/// * `screen` - The screen to show.
pub fn act_out(engine: &mut Engine<App>, app: &mut App, ids: Ids, screen: Screen) {
    match screen {
        Screen::Main | Screen::Resized => {}
        Screen::Invalid => {
            click(engine, app, ids.email);

            if let Some(field) = engine.widgets().get_as_mut::<TextField>(ids.email) {
                field.set_text("");
            }

            engine.dispatch(InputEvent::Text("jane.doe@example".to_string()), app);
        }
        Screen::ConfirmDelete => click(engine, app, ids.delete),
    }
}

/// Draws a screen in a new window and saves it as an image.
///
/// # Arguments
///
/// * `screen` - The screen to draw.
/// * `path` - Where to save the image; the format follows the extension (`.png`).
///
/// # Returns
///
/// Nothing on success.
///
/// # Errors
///
/// Returns the backend's error if the window cannot be created or the image cannot be saved.
pub fn render_screen(screen: Screen, path: &str) -> Result<(), pushrod::error::Error> {
    let mut app = App::new(ContactBook::default());
    let (mut engine, ids) = build(screen.window(), &app);
    let mut backend = Sdl2Backend::new(&WindowSettings::new(TITLE, screen.window()))?;

    // Without the run loop, nothing reports the display's scale; tell the engine as it would.
    engine.dispatch(InputEvent::ScaleChanged(backend.scale_factor()), &mut app);

    // The first frame lays the window out, so the scripted clicks land on the right widgets.
    engine.widgets().draw_loop(&mut backend);
    act_out(&mut engine, &mut app, ids, screen);
    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);
    backend.save_screenshot(path)
}
