//! The widget gallery: one window showing every merged widget, each on a labeled tile.
//!
//! Run it from the repository root with `cargo run -p pushrod-sdl2 --example gallery`.
//! Click **Quit** (or close the window) to exit.  Pass `--screenshot <file.png>` to draw one frame,
//! save it, and exit without waiting for input; this is how the documentation screenshot is made.
//!
//! Every time a widget issue merges, add its tile here: call [`add_tile`] for the label and
//! frame, then add the widget (and its states) inside the tile's content area.  All sample
//! data in this gallery is fictional.

use pushrod::color::{Color, FontStyle};
use pushrod::engine::Engine;
use pushrod::geometry::{Point, point, size};
use pushrod::paint::{Truncate, Wrap};
use pushrod::widget::{Behavior, Widget, WidgetId};
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::base_widget::BaseWidget;
use pushrod_widgets::box_widget::BoxWidget;
use pushrod_widgets::button_widget::ButtonWidget;
use pushrod_widgets::text_widget::{TextJustify, TextWidget};

const FONT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/OpenSans-Regular.ttf"
);

/// Window size in pixels.
const WINDOW_W: u32 = 640;
const WINDOW_H: u32 = 570;

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
    let mut label = TextWidget::new(
        origin,
        size(width, 18),
        String::from(FONT),
        FontStyle::NORMAL,
        14,
        Color::WHITE,
        TextJustify::Left,
        String::from(text),
    );
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
    let mut frame = BoxWidget::new(origin, size(TILE_W, TILE_H), Color::WHITE, 1);
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
fn button(origin: Point, text: &str) -> ButtonWidget {
    ButtonWidget::new(
        origin,
        size(120, 24),
        String::from(FONT),
        FontStyle::NORMAL,
        14,
        Color::BLACK,
        TextJustify::Center,
        String::from(text),
        2,
    )
}

/// Draws one frame of the widget tree into a fresh window and saves it as an image.
///
/// * `engine` - the engine holding the fully built gallery.
/// * `settings` - how to create the window to draw into.
/// * `path` - where to save the image; the format follows the extension (`.png`).
fn save_screenshot(engine: &mut Engine<AppState>, settings: &WindowSettings, path: &str) {
    let mut backend = Sdl2Backend::new(settings).expect("SDL2 must open the window");

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);
    backend.save_screenshot(path).unwrap();
}

pub fn main() {
    let settings = WindowSettings::new("pushrod gallery", size(WINDOW_W, WINDOW_H));
    let mut engine: Engine<AppState> = Engine::new(size(WINDOW_W, WINDOW_H), 30);

    let mut panel = BaseWidget::new(point(0, 0), size(WINDOW_W, WINDOW_H));
    panel.set_color(Color::RGBA(80, 80, 80, 255));
    let panel_id = engine.add_widget(panel);

    // Base: a plain filled rectangle.
    let base_tile = add_tile(&mut engine, panel_id, 0, 0, "BaseWidget");
    let mut base = BaseWidget::new(point(10, 36), size(120, 60));
    base.set_color(Color::RGBA(30, 90, 160, 255));
    engine.add_child_widget(base_tile, base);

    // Box: a fill with a border, at three border widths.
    let box_tile = add_tile(&mut engine, panel_id, 1, 0, "BoxWidget");
    for (i, width) in [1u8, 3, 6].iter().enumerate() {
        let mut sample = BoxWidget::new(
            point(10 + i as i32 * 90, 36),
            size(80, 60),
            Color::BLUE,
            *width,
        );
        sample.set_color(Color::CYAN);
        engine.add_child_widget(box_tile, sample);
    }

    // Button: normal and disabled.
    let button_tile = add_tile(&mut engine, panel_id, 0, 1, "ButtonWidget");
    engine.add_child_widget(button_tile, button(point(10, 36), "Normal"));
    let mut disabled = button(point(150, 36), "Disabled");
    disabled.set_enabled(false);
    engine.add_child_widget(button_tile, disabled);

    // Text: left, centered, and right justified labels (fictional sample text).
    let text_tile = add_tile(&mut engine, panel_id, 1, 1, "TextWidget");
    for (i, (justify, text)) in [
        (TextJustify::Left, "Jane Doe"),
        (TextJustify::Center, "Anytown"),
        (TextJustify::Right, "jane.doe@example.com"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut sample = TextWidget::new(
            point(10, 36 + i as i32 * 26),
            size(TILE_W - 20, 18),
            String::from(FONT),
            FontStyle::NORMAL,
            14,
            Color::WHITE,
            justify,
            String::from(text),
        );
        sample.set_color(Color::RGBA(30, 90, 160, 255));
        engine.add_child_widget(text_tile, sample);
    }

    // Truncation: the same long (fictional) file name cut at the end, start and middle.
    let truncate_tile = add_tile(&mut engine, panel_id, 0, 2, "Truncate (End, Start, Middle)");
    for (i, mode) in [Truncate::End, Truncate::Start, Truncate::Middle]
        .into_iter()
        .enumerate()
    {
        let mut sample = TextWidget::new(
            point(10, 36 + i as i32 * 26),
            size(TILE_W - 20, 18),
            String::from(FONT),
            FontStyle::NORMAL,
            14,
            Color::WHITE,
            TextJustify::Left,
            String::from("Quarterly sales report for the western region.txt"),
        );
        sample.set_color(Color::RGBA(30, 90, 160, 255));
        sample.set_truncate(mode);
        engine.add_child_widget(truncate_tile, sample);
    }

    // Wrapped text: a (fictional) paragraph that reflows, limited to four lines.
    let wrap_tile = add_tile(&mut engine, panel_id, 1, 2, "Wrap (Word, max 4 lines)");
    let mut paragraph = TextWidget::new(
        point(10, 32),
        size(TILE_W - 20, 110),
        String::from(FONT),
        FontStyle::NORMAL,
        14,
        Color::WHITE,
        TextJustify::Left,
        String::from(
            "Your changes to the quarterly report have not been saved.\nClose the document anyway and discard them?",
        ),
    );
    paragraph.set_color(Color::RGBA(30, 90, 160, 255));
    paragraph.set_wrap(Wrap::Word);
    paragraph.set_max_lines(Some(4));
    engine.add_child_widget(wrap_tile, paragraph);

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
        save_screenshot(&mut engine, &settings, path);
        return;
    }

    pushrod_sdl2::run(engine, settings, AppState);
}
