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
use pushrod::engine::Engine;
use pushrod::event::{Event, Propagation};
use pushrod::geometry::{Point, Rect, Size, point, size};
use pushrod::paint::{self, Painter, Truncate, Wrap};
use pushrod::widget::{Behavior, Widget, WidgetId};
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::boxed_label::BoxedLabel;
use pushrod_widgets::button::Button;
use pushrod_widgets::custom_paint::CustomPaint;
use pushrod_widgets::frame::Frame;
use pushrod_widgets::group_box::GroupBox;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::separator::Separator;
use pushrod_widgets::text::{Text, TextJustify};

/// Window size in pixels.
const WINDOW_W: u32 = 640;
const WINDOW_H: u32 = 910;

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
    let button_tile = add_tile(&mut engine, panel_id, 0, 1, "Button");
    engine.add_child_widget(button_tile, button(point(10, 36), "Normal"));
    let mut disabled = button(point(150, 36), "Disabled");
    disabled.set_enabled(false);
    engine.add_child_widget(button_tile, disabled);

    // Text: left, centered, and right justified labels (fictional sample text).
    let text_tile = add_tile(&mut engine, panel_id, 1, 1, "Text");
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
    let truncate_tile = add_tile(&mut engine, panel_id, 0, 2, "Truncate (End, Start, Middle)");
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
    let wrap_tile = add_tile(&mut engine, panel_id, 1, 2, "Wrap (Word, max 4 lines)");
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
        3,
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
    let boxed_tile = add_tile(&mut engine, panel_id, 1, 3, "Boxed Label");
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
    let separator_tile = add_tile(&mut engine, panel_id, 0, 4, "Separator");
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
    let group_tile = add_tile(&mut engine, panel_id, 1, 4, "Group Box");
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
