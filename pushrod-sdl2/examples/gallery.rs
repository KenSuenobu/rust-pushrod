//! The widget gallery: one window showing every merged widget, each on a labeled tile.
//!
//! Run it from the repository root with `cargo run -p pushrod-sdl2 --example gallery`.
//! Click **Quit** (or close the window) to exit.  Pass `--screenshot <file.png>` to draw one frame,
//! save it, and exit without waiting for input; this is how the documentation screenshot is made.
//!
//! Every time a widget issue merges, add its tile here: call [`add_tile`] for the label and
//! frame, then add the widget (and its states) inside the tile's content area.  All sample
//! data in this gallery is fictional.

use pushrod::color::Color;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::event::{Event, Propagation};
use pushrod::geometry::{Point, Rect, Size, point, size};
use pushrod::layout::{Align, Alignment, Column, FormLayout, LayoutParams, Margin, Row, Spacer};
use pushrod::paint::{self, Painter, Truncate, Wrap};
use pushrod::widget::{Behavior, Widget, WidgetId};
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::boxed_label::BoxedLabel;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::container::Container;
use pushrod_widgets::custom_paint::CustomPaint;
use pushrod_widgets::frame::Frame;
use pushrod_widgets::glyph_button::GlyphButton;
use pushrod_widgets::group_box::GroupBox;
use pushrod_widgets::image_widget::{FitMode, ImageWidget};
use pushrod_widgets::list_view::{ListView, SelectionMode, VecModel};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::progress_bar::ProgressBar;
use pushrod_widgets::radio_button::RadioGroup;
use pushrod_widgets::scroll_bar::ScrollBar;
use pushrod_widgets::separator::Separator;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::tab_view::TabView;
use pushrod_widgets::text::{Text, TextJustify};
use pushrod_widgets::text_field::{Accessory, IntegerField, TextField};

/// Window size in pixels: three columns of tiles.
const WINDOW_W: u32 = 950;
const WINDOW_H: u32 = 1420;

/// Tile frame size in pixels, and the gap between tiles.
const TILE_W: u32 = 290;
const TILE_H: u32 = 150;
const GAP: i32 = 20;

/// Gray used for the window and tile backgrounds.
const BACKGROUND: Color = Color::RGBA(127, 127, 127, 255);

/// Application state: the gallery needs none.
#[derive(Default)]
struct AppState;

/// Creates a plain text label at `origin` (relative to `parent`) and adds it to the tree.
///
/// * `engine` - the engine that owns the widget tree.
/// * `parent` - the widget the label is a child of.
/// * `origin` - the label's upper left-hand corner, relative to the parent.
/// * `width` - the label's width in pixels.
/// * `text` - the label's text.
///
/// Returns the new label's ID.
fn add_label(
    engine: &mut Engine<AppState>,
    parent: WidgetId,
    origin: Point,
    width: u32,
    text: &str,
) -> WidgetId {
    let mut label = Text::new(text)
        .at(origin.x, origin.y)
        .size(width, 18)
        .font_color(Color::WHITE);
    label.set_color(BACKGROUND);
    engine.add_child_widget(parent, label)
}

/// Adds a labeled tile: a framed panel with a title strip.
///
/// * `engine` - the engine that owns the widget tree.
/// * `parent` - the gallery panel.
/// * `column`, `row` - the tile's grid position (zero-based).
/// * `title` - the widget's name, shown at the top of the tile.
///
/// Returns the ID of the tile's panel; add the widget samples as children of it, with origins
/// relative to the tile (content starts at `y = 30`).
fn add_tile(
    engine: &mut Engine<AppState>,
    parent: WidgetId,
    column: u32,
    row: u32,
    title: &str,
) -> WidgetId {
    let origin = point(
        GAP + column as i32 * (TILE_W as i32 + GAP),
        GAP + row as i32 * (TILE_H as i32 + GAP),
    );
    let mut frame = Frame::new()
        .at(origin.x, origin.y)
        .size(TILE_W, TILE_H)
        .border_color(Color::WHITE);
    frame.set_color(BACKGROUND);
    let tile = engine.add_child_widget(parent, frame);

    add_label(engine, tile, point(10, 6), TILE_W - 20, title);
    tile
}

/// Creates a standard 14 point button.
///
/// * `origin` - the button's upper left-hand corner, relative to its parent.
/// * `text` - the button's label.
///
/// Returns the button, not yet added to the tree.
fn button(origin: Point, text: &str) -> Button {
    Button::new(text).at(origin.x, origin.y).size(120, 24)
}

/// Draws the gallery's bar chart: white background, one blue bar per value (black when
/// highlighted), and a one-pixel border.
///
/// * `p` - the painter, targeting the chart widget.
/// * `size` - the widget's size in pixels.
/// * `data` - the bar heights (percent of the chart height) and the highlighted bar's index.
fn draw_chart(p: &mut dyn Painter, size: Size, data: &(Vec<u32>, usize)) {
    let (bars, selected) = data;
    let slot = size.w / bars.len().max(1) as u32;

    paint::fill(p, Color::WHITE);

    for (i, value) in bars.iter().enumerate() {
        let height = size.h * (*value).min(100) / 100;
        let color = if i == *selected {
            Color::BLACK
        } else {
            Color::BLUE
        };

        p.fill_rect(
            Rect::new(
                i as i32 * slot as i32 + 6,
                (size.h - height) as i32,
                slot - 12,
                height,
            ),
            color,
        );
    }

    paint::border(p, size, Color::BLACK, 1);
}

/// Draws one frame of the widget tree into a fresh window and saves it as an image.
///
/// * `engine` - the engine holding the fully built gallery.
/// * `settings` - how to create the window to draw into.
/// * `path` - where to save the image; the format follows the extension (`.png`).
fn save_screenshot(
    engine: &mut Engine<AppState>,
    settings: &WindowSettings,
    path: &str,
) -> Result<(), pushrod::error::Error> {
    let mut backend = Sdl2Backend::new(settings)?;

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);
    backend.save_screenshot(path)
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let settings = WindowSettings::new("pushrod gallery", size(WINDOW_W, WINDOW_H));
    let mut engine: Engine<AppState> = Engine::new(size(WINDOW_W, WINDOW_H), 30);

    let mut panel = Panel::new().size(WINDOW_W, WINDOW_H);
    panel.set_color(Color::RGBA(80, 80, 80, 255));
    let panel_id = engine.add_widget(panel);

    // Base: a plain filled rectangle.
    let base_tile = add_tile(&mut engine, panel_id, 0, 0, "Panel");
    let mut base = Panel::new().at(10, 36).size(120, 60);
    base.set_color(Color::RGBA(30, 90, 160, 255));
    engine.add_child_widget(base_tile, base);

    // Box: a fill with a border, at three border widths.
    let box_tile = add_tile(&mut engine, panel_id, 1, 0, "Frame");
    for (i, width) in [1u8, 3, 6].iter().enumerate() {
        let mut sample = Frame::new()
            .at(10 + i as i32 * 90, 36)
            .size(80, 60)
            .border_color(Color::BLUE)
            .border_width(*width);
        sample.set_color(Color::CYAN);
        engine.add_child_widget(box_tile, sample);
    }

    // Button: normal and disabled.
    let button_tile = add_tile(&mut engine, panel_id, 2, 0, "Button");
    engine.add_child_widget(button_tile, button(point(10, 36), "Normal"));
    let mut disabled = button(point(150, 36), "Disabled");
    disabled.set_enabled(false);
    engine.add_child_widget(button_tile, disabled);

    // Behavior flags: a default (ring) and a cancel button, and a toggle that is checked.
    let small = |label: &str, x: i32, y: i32| Button::new(label).at(x, y).size(80, 24);
    engine.add_child_widget(button_tile, small("OK", 10, 76).default_action());
    engine.add_child_widget(button_tile, small("Cancel", 100, 76).cancel_action());

    let mut bold = small("Bold", 190, 76).toggle();
    bold.set_checked(true);
    engine.add_child_widget(button_tile, bold);
    add_label(
        &mut engine,
        button_tile,
        point(10, 110),
        TILE_W - 20,
        "Default, cancel, toggle",
    );

    // Text: left, centered, and right justified labels (fictional sample text).
    let text_tile = add_tile(&mut engine, panel_id, 0, 1, "Text");
    for (i, (justify, text)) in [
        (TextJustify::Left, "Jane Doe"),
        (TextJustify::Center, "Anytown"),
        (TextJustify::Right, "jane.doe@example.com"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut sample = Text::new(text)
            .at(10, 36 + i as i32 * 26)
            .size(TILE_W - 20, 18)
            .font_color(Color::WHITE)
            .justify(justify);
        sample.set_color(Color::RGBA(30, 90, 160, 255));
        engine.add_child_widget(text_tile, sample);
    }

    // Truncation: the same long (fictional) file name cut at the end, start and middle.
    let truncate_tile = add_tile(&mut engine, panel_id, 1, 1, "Truncate (End, Start, Middle)");
    for (i, mode) in [Truncate::End, Truncate::Start, Truncate::Middle]
        .into_iter()
        .enumerate()
    {
        let mut sample = Text::new("Quarterly sales report for the western region.txt")
            .at(10, 36 + i as i32 * 26)
            .size(TILE_W - 20, 18)
            .font_color(Color::WHITE);
        sample.set_color(Color::RGBA(30, 90, 160, 255));
        sample.set_truncate(mode);
        engine.add_child_widget(truncate_tile, sample);
    }

    // Wrapped text: a (fictional) paragraph that reflows, limited to four lines.
    let wrap_tile = add_tile(&mut engine, panel_id, 2, 1, "Wrap (Word, max 4 lines)");
    let mut paragraph = Text::new(
        "Your changes to the quarterly report have not been saved.\nClose the document anyway and discard them?",
    )
    .at(10, 32)
    .size(TILE_W - 20, 110)
    .font_color(Color::WHITE);
    paragraph.set_color(Color::RGBA(30, 90, 160, 255));
    paragraph.set_wrap(Wrap::Word);
    paragraph.set_max_lines(Some(4));
    engine.add_child_widget(wrap_tile, paragraph);

    // Custom paint: a bar chart of fictional monthly totals drawn by a closure.  Each press
    // highlights the next bar, which the event closure records in the widget's data.
    let chart_tile = add_tile(
        &mut engine,
        panel_id,
        0,
        2,
        "Custom Paint (click to highlight)",
    );
    engine.add_child_widget(
        chart_tile,
        CustomPaint::with_data(
            point(10, 36),
            size(TILE_W - 20, 100),
            (vec![40u32, 65, 30, 80, 55, 90], 0usize),
            draw_chart,
        )
        .on_event(|(bars, selected), event, _ctx| {
            if matches!(event, Event::PointerPressed { .. }) {
                *selected = (*selected + 1) % bars.len();
                return Propagation::Stop;
            }

            Propagation::Continue
        }),
    );

    // Boxed label: text with a fill and a border; the second is disabled (fictional status text).
    let boxed_tile = add_tile(&mut engine, panel_id, 1, 2, "Boxed Label");
    let mut ready = BoxedLabel::new("Ready")
        .at(10, 36)
        .size(TILE_W - 20, 28)
        .border_color(Color::BLUE);
    ready.set_color(Color::CYAN);
    engine.add_child_widget(boxed_tile, ready);
    let mut offline = BoxedLabel::new("Offline").at(10, 76).size(TILE_W - 20, 28);
    offline.set_color(Color::WHITE);
    offline.set_enabled(false);
    engine.add_child_widget(boxed_tile, offline);

    // Separator: a horizontal line, a vertical line, and an etched pair (fictional layout).
    let separator_tile = add_tile(&mut engine, panel_id, 2, 2, "Separator");
    engine.add_child_widget(
        separator_tile,
        Separator::horizontal(TILE_W - 20).at(10, 50),
    );
    engine.add_child_widget(
        separator_tile,
        Separator::horizontal(TILE_W - 20).etched().at(10, 70),
    );
    engine.add_child_widget(
        separator_tile,
        Separator::vertical(60).line_color(Color::BLUE).at(70, 85),
    );
    engine.add_child_widget(separator_tile, Separator::vertical(60).etched().at(140, 85));

    // GroupBox: a titled frame that holds child widgets (fictional dialog).
    let group_tile = add_tile(&mut engine, panel_id, 0, 3, "Group Box");
    let group = engine.add_child_widget(
        group_tile,
        GroupBox::new("Size").at(10, 36).size(TILE_W - 20, 104),
    );
    engine.add_child_widget(
        group,
        Text::new("Small, medium, large")
            .at(12, 24)
            .size(TILE_W - 44, 18),
    );
    engine.add_child_widget(group, button(point(12, 56), "Apply"));

    // CheckBox: unchecked, checked, indeterminate, and disabled (fictional options).
    let check_tile = add_tile(&mut engine, panel_id, 0, 4, "Check Box");
    let mut disabled_box = CheckBox::new("Disabled")
        .at(150, 70)
        .size(130, 20)
        .checked(true);
    disabled_box.set_enabled(false);
    engine.add_child_widget(
        check_tile,
        CheckBox::new("Send email").at(10, 40).size(130, 20),
    );
    engine.add_child_widget(
        check_tile,
        CheckBox::new("Checked")
            .at(150, 40)
            .size(130, 20)
            .checked(true),
    );
    engine.add_child_widget(
        check_tile,
        CheckBox::new("All items")
            .at(10, 70)
            .size(130, 20)
            .indeterminate(),
    );
    engine.add_child_widget(check_tile, disabled_box);

    // RadioButton: a size picker (Medium checked) and a disabled pair (fictional options).
    let radio_tile = add_tile(&mut engine, panel_id, 1, 4, "Radio Button");
    RadioGroup::new(["Small", "Medium", "Large"])
        .at(10, 40)
        .item_size(130, 20)
        .selected(1)
        .add_to(engine.widgets(), radio_tile);
    let disabled = RadioGroup::new(["Disabled", "Disabled"])
        .at(150, 40)
        .item_size(130, 20)
        .selected(0)
        .add_to(engine.widgets(), radio_tile);
    for id in disabled {
        engine.widgets().get_mut(id).unwrap().set_enabled(false);
    }

    // ProgressBar: empty, 40%, busy (animated), and busy in outline mode (static stripes).
    let progress_tile = add_tile(&mut engine, panel_id, 2, 4, "Progress Bar");
    for (row, value) in [0.0, 40.0].into_iter().enumerate() {
        let bar = ProgressBar::new(100.0)
            .at(10, 40 + row as i32 * 24)
            .size(270, 16)
            .value(value);
        engine.add_child_widget(progress_tile, bar);
    }
    let busy = engine.add_child_widget(
        progress_tile,
        ProgressBar::indeterminate().at(10, 88).size(270, 16),
    );
    engine.widgets().request_animation(busy);
    let mut outline = ProgressBar::indeterminate().at(10, 112).size(270, 16);
    outline.set_outline_mode(true);
    engine.add_child_widget(progress_tile, outline);

    // Slider: a volume slider with its value, one with ticks and snapping, a disabled one, and a
    // vertical one.
    let slider_tile = add_tile(&mut engine, panel_id, 0, 5, "Slider");
    engine.add_child_widget(
        slider_tile,
        Slider::new(0.0, 100.0)
            .at(10, 36)
            .size(210, 24)
            .value(40.0)
            .show_value(true),
    );
    engine.add_child_widget(
        slider_tile,
        Slider::new(0.0, 100.0)
            .at(10, 68)
            .size(210, 28)
            .value(50.0)
            .step(10.0)
            .ticks(10.0)
            .show_value(true),
    );
    let disabled = engine.add_child_widget(
        slider_tile,
        Slider::new(0.0, 100.0)
            .at(10, 104)
            .size(210, 24)
            .value(40.0)
            .show_value(true),
    );
    engine
        .widgets()
        .get_mut(disabled)
        .unwrap()
        .set_enabled(false);
    engine.add_child_widget(
        slider_tile,
        Slider::new(0.0, 100.0)
            .vertical()
            .at(240, 32)
            .size(30, 108)
            .value(75.0)
            .show_value(true),
    );

    // TextField: a labelled field, a date mask, a search field, a password with reveal, a
    // whole-number field, and a disabled field.
    let field_tile = add_tile(&mut engine, panel_id, 1, 5, "Text Field");
    engine.add_child_widget(
        field_tile,
        TextField::new()
            .at(10, 34)
            .size(270, 24)
            .with_label("Name:")
            .placeholder("Your name")
            .with_text("Jane"),
    );
    engine.add_child_widget(
        field_tile,
        TextField::new()
            .at(10, 64)
            .size(270, 24)
            .with_label("Date:")
            .with_mask("99/99/9999")
            .placeholder("MM/DD/YYYY"),
    );
    engine.add_child_widget(field_tile, TextField::search().at(10, 94).size(270, 24));
    let disabled_field = engine.add_child_widget(
        field_tile,
        TextField::new()
            .at(10, 122)
            .size(270, 22)
            .with_text("Disabled"),
    );
    engine
        .widgets()
        .get_mut(disabled_field)
        .unwrap()
        .set_enabled(false);

    let restricted_tile = add_tile(&mut engine, panel_id, 2, 5, "Text Field (restricted)");
    engine.add_child_widget(
        restricted_tile,
        TextField::new()
            .at(10, 34)
            .size(270, 24)
            .with_label("Password:")
            .password()
            .with_text("sample-pass")
            .with_trailing(Accessory::RevealPassword),
    );
    engine.add_child_widget(
        restricted_tile,
        IntegerField::range(0, 120)
            .at(10, 64)
            .size(270, 24)
            .with_label("Age (0-120):")
            .placeholder("0"),
    );
    engine.add_child_widget(
        restricted_tile,
        TextField::new()
            .at(10, 94)
            .size(270, 24)
            .with_label("Code (4):")
            .max_length(4)
            .with_trailing(Accessory::Clear)
            .with_text("AB12"),
    );

    // GlyphButton: four small symbol buttons; the last two fire on press and repeat while held.
    let glyph_tile = add_tile(&mut engine, panel_id, 2, 3, "Glyph Button");
    let glyphs = [
        ('←', "Click", false),
        ('→', "Click", false),
        ('↑', "Repeats", true),
        ('↓', "Repeats", true),
    ];

    for (column, (glyph, note, repeats)) in glyphs.into_iter().enumerate() {
        let x = 10 + column as i32 * 64;
        let mut glyph_button = GlyphButton::new(glyph).at(x, 44).size(24, 24);

        if repeats {
            glyph_button = glyph_button.activate_on_press();
        }

        engine.add_child_widget(glyph_tile, glyph_button);
        engine.add_child_widget(glyph_tile, Text::new(note).at(x, 76).size(60, 20));
    }

    // ImageWidget: the four fit modes, then a missing file (a crossed box).
    let image_tile = add_tile(&mut engine, panel_id, 1, 3, "Image");
    let sample = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/sample.png");
    let samples = [
        ("Center", FitMode::Center, sample),
        ("Stretch", FitMode::Stretch, sample),
        ("Fit", FitMode::Fit, sample),
        ("Tile", FitMode::Tile, sample),
        ("Missing", FitMode::Fit, "missing.png"),
    ];

    for (column, (name, fit, path)) in samples.into_iter().enumerate() {
        let x = 10 + column as i32 * 54;
        let mut image = ImageWidget::new(path).at(x, 40).size(46, 70).fit(fit);

        image.set_color(Color::RGBA(225, 225, 225, 255));
        engine.add_child_widget(image_tile, image);
        engine.add_child_widget(image_tile, Text::new(name).at(x, 112).size(54, 20));
    }

    // Row and Column: a column of fields above a row whose spacer pushes OK and Cancel right.
    let layout_tile = add_tile(&mut engine, panel_id, 0, 6, "Row and Column");
    let mut form = Panel::new().at(10, 30).size(270, 112);

    form.set_color(Color::RGBA(225, 225, 225, 255));

    let form = engine.add_child_widget(layout_tile, form);
    let fill = LayoutParams {
        alignment: Alignment::new(Align::Fill, Align::Start),
        ..LayoutParams::default()
    };

    for (label, text) in [("Name:", "Jane Doe"), ("City:", "Anytown")] {
        let field = engine.add_child_widget(
            form,
            TextField::new()
                .size(120, 24)
                .with_label(label)
                .with_text(text),
        );

        engine
            .widgets()
            .get_mut(field)
            .unwrap()
            .set_layout_params(fill);
    }

    engine.add_child_widget(form, Spacer::stretch());

    let mut strip = Panel::new();

    strip.set_color(Color::RGBA(225, 225, 225, 255));

    let buttons = engine.add_child_widget(form, strip);

    engine.add_child_widget(buttons, Spacer::stretch());
    engine.add_child_widget(buttons, Button::new("OK").size(70, 24));
    engine.add_child_widget(buttons, Button::new("Cancel").size(70, 24));

    let row = engine.widgets().get_mut(buttons).unwrap();

    row.set_layout_params(fill);
    row.set_layout(Box::new(Row::new().spacing(6)));
    engine
        .widgets()
        .get_mut(form)
        .unwrap()
        .set_layout(Box::new(Column::new().spacing(6).margins(Margin::all(6))));

    // Form Layout: labels right-aligned in a column as wide as the widest, fields filling the
    // rest, and a full-width check box.
    let form_tile = add_tile(&mut engine, panel_id, 1, 6, "Form Layout");
    let mut sheet = Panel::new().at(10, 30).size(270, 112);

    sheet.set_color(Color::RGBA(225, 225, 225, 255));

    let sheet = engine.add_child_widget(form_tile, sheet);
    let mut form_layout = FormLayout::new()
        .spacing(6)
        .column_spacing(6)
        .margins(Margin::all(6));

    for (label, text) in [
        ("Name:", "Jane Doe"),
        ("Server address:", "mail.example.com"),
    ] {
        let mut label = Text::new(label);

        // Labels share the form's background, so they read as text drawn on the form.
        label.set_color(Color::RGBA(225, 225, 225, 255));

        let label = engine.add_child_widget(sheet, label);
        let field = engine.add_child_widget(sheet, TextField::new().size(100, 24).with_text(text));

        form_layout.add_row(label, field);
    }

    let mut check = CheckBox::new("Remember me").checked(true);

    // The check box shares the form's background too.
    check.set_color(Color::RGBA(225, 225, 225, 255));

    let remember = engine.add_child_widget(sheet, check);

    form_layout.add_full_row(remember);
    engine
        .widgets()
        .get_mut(sheet)
        .unwrap()
        .set_layout(Box::new(form_layout));

    // Scroll Bar: two vertical bars (a quarter and a tenth of the content visible) and two
    // horizontal ones; moving any of them shows its value below.
    let scroll_tile = add_tile(&mut engine, panel_id, 2, 6, "Scroll Bar");
    let scroll_bars = [
        ScrollBar::vertical().at(10, 32).size(16, 108).value(0.3),
        ScrollBar::vertical()
            .at(34, 32)
            .size(16, 108)
            .page_size(0.1)
            .value(0.8),
        ScrollBar::horizontal().at(62, 36).size(218, 16),
        ScrollBar::horizontal()
            .at(62, 62)
            .size(218, 16)
            .page_size(0.6)
            .value(0.5),
    ];
    let scroll_value = add_label(
        &mut engine,
        scroll_tile,
        point(62, 100),
        218,
        "Move a scroll bar",
    );

    for bar in scroll_bars {
        let id = engine.add_child_widget(scroll_tile, bar);

        engine.on_value_change(id, move |_state, ui| {
            let value = ui.get::<ScrollBar>(id).map_or(0.0, |bar| bar.get_value());

            if let Some(label) = ui.get_mut::<Text>(scroll_value) {
                label.set_text(format!("Scroll bar {id}: {value:.2}"));
            }
        });
    }

    // Container: a 270 by 104 viewport onto 420 by 260 of content, panned a little so both
    // scroll bars show their thumbs away from the ends.  Scroll it with the wheel or the bars.
    let container_tile = add_tile(&mut engine, panel_id, 0, 7, "Container");
    let mut container = Container::new(point(10, 36), size(TILE_W - 20, 104), size(420, 260));

    container.set_color(Color::WHITE);
    container.scroll_to(point(30, 20));

    let container = engine.add_child_widget(container_tile, container);
    let content = engine
        .widgets()
        .get_as::<Container>(container)
        .map_or(container, |c| c.typed_parts().content);

    for (index, text) in ["Inbox", "Drafts", "Sent", "Archive", "Trash"]
        .into_iter()
        .enumerate()
    {
        let offset = index as i32;

        engine.add_child_widget(
            content,
            button(point(20 + offset * 70, 20 + offset * 44), text),
        );
    }

    add_label(
        &mut engine,
        content,
        point(160, 24),
        220,
        "Content: 420 by 260",
    );

    // List View: 10,000 fictional rows from a model; only the six that fit exist as widgets.
    // Click, Shift-click, Ctrl-click, the arrow keys, and type-ahead work on it.
    let list_tile = add_tile(&mut engine, panel_id, 1, 7, "List View");
    let mut list = ListView::new(VecModel::new((1..=10_000).map(|n| format!("Item {n:05}"))))
        .at(10, 36)
        .size(TILE_W - 20, 104)
        .selection_mode(SelectionMode::Multiple);

    list.select_range(1..3);
    engine.add_child_widget(list_tile, list);

    // Tab View: three pages of fictional settings.  Click a tab, or press Ctrl+Tab (Ctrl+Shift+Tab
    // goes back) while a tab or a control in a page has the keyboard focus.
    let tabs_tile = add_tile(&mut engine, panel_id, 2, 7, "Tab View");
    let tabs = engine.add_child_widget(
        tabs_tile,
        TabView::new()
            .at(10, 36)
            .size(TILE_W - 20, 104)
            .with_page("General")
            .with_page("Advanced")
            .with_page("About"),
    );
    let pages = engine
        .widgets()
        .get_as::<TabView>(tabs)
        .map(TabView::pages)
        .unwrap_or_default();

    if let [general, advanced, about] = pages[..] {
        engine.add_child_widget(
            general,
            CheckBox::new("Check for updates").at(10, 10).size(200, 20),
        );
        engine.add_child_widget(
            general,
            CheckBox::new("Show hints").at(10, 36).size(200, 20),
        );
        engine.add_child_widget(advanced, Text::new("Cache size").at(10, 10).size(120, 20));
        engine.add_child_widget(advanced, TextField::new().at(10, 36).size(120, 24));
        engine.add_child_widget(
            about,
            Text::new("Sample Settings 1.0").at(10, 10).size(200, 20),
        );
    }

    let quit_id = engine.add_child_widget(
        panel_id,
        button(
            point((WINDOW_W as i32 - 120) / 2, WINDOW_H as i32 - GAP - 24 + 10),
            "Quit",
        ),
    );
    engine
        .widgets()
        .get_mut(quit_id)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE | Behavior::CANCEL_ACTION);
    engine.on_click(quit_id, |_state, ui| ui.shutdown());

    let args: Vec<String> = std::env::args().collect();

    if let Some(path) = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
    {
        return save_screenshot(&mut engine, &settings, path);
    }

    pushrod_sdl2::run(engine, settings, AppState)?;

    Ok(())
}
