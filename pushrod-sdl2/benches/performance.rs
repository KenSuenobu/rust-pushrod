//! Performance benchmarks (issue #115): event dispatch, frame time, text layout and atlas warm-up,
//! form load time, and texture memory per widget.
//!
//! Run them from the repository root with `cargo bench -p pushrod-sdl2 --bench performance`.  The
//! benchmarks use built-in timing (no extra dependency), run headless (the engine draws through
//! `HeadlessBackend`; the text benchmark uses SDL's dummy video driver and the software
//! renderer, so no display is needed), and print one JSON object of named numbers on the last
//! line of output.  `scripts/check_benchmarks.py` compares that object with the stored baseline,
//! `benches/baseline.json`.  All sample data is fictional.

use std::hint::black_box;
use std::time::Instant;

use pushrod::backend::headless::HeadlessBackend;
use pushrod::color::{Color, FontStyle};
use pushrod::engine::Engine;
use pushrod::font::Font;
use pushrod::geometry::{point, size};
use pushrod::input::{InputEvent, MouseButton};
use pushrod::layout::{FormLayout, Margin};
use pushrod::widget::{Widget, WidgetId};
use pushrod_sdl2::font::FontCache;
use pushrod_widgets::button::Button;
use pushrod_widgets::check_box::CheckBox;
use pushrod_widgets::frame::Frame;
use pushrod_widgets::panel::Panel;
use pushrod_widgets::progress_bar::ProgressBar;
use pushrod_widgets::slider::Slider;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;
use sdl2::pixels::Color as SdlColor;

/// Window size of the gallery-like tree: three columns of tiles.
const WINDOW: (u32, u32) = (950, 1420);

/// Tiles in the gallery-like tree (each holds six widgets).
const TILES: u32 = 21;

/// Rows in the form-load benchmark.
const FORM_ROWS: u32 = 40;

/// Rounds each timing is the best of, to shrug off a noisy neighbor.
const ROUNDS: u32 = 5;

/// Runs `work` `ROUNDS` times and returns the fastest run, in nanoseconds per `iterations`.
///
/// * `iterations` - how many operations one run of `work` performs.
/// * `work` - the code to time.
///
/// Returns the best time for one operation, in nanoseconds.
fn best_ns<F: FnMut()>(iterations: u32, mut work: F) -> f64 {
    let mut best = f64::MAX;

    for _ in 0..ROUNDS {
        let start = Instant::now();

        work();
        best = best.min(start.elapsed().as_nanos() as f64 / f64::from(iterations));
    }

    best
}

/// Builds the gallery-like tree: tiles of a frame, a label, a button, a check box, a slider, a
/// progress bar, and a text field.
///
/// * `engine` - the engine that owns the tree.
///
/// Returns the slider IDs (the widgets the frame benchmark changes) and the widget count.
fn build_gallery(engine: &mut Engine<()>) -> (Vec<WidgetId>, usize) {
    let mut panel = Panel::new().at(0, 0).size(WINDOW.0, WINDOW.1);

    panel.set_color(Color::RGBA(127, 127, 127, 255));

    let panel = engine.add_widget(panel);
    let mut sliders = Vec::new();
    let mut count = 1;

    for tile in 0..TILES {
        let x = 20 + (tile % 3) as i32 * 310;
        let y = 20 + (tile / 3) as i32 * 170;
        let frame = engine.add_child_widget(panel, Frame::new().at(x, y).size(290, 150));

        engine.add_child_widget(frame, Text::new("Sample tile").at(10, 6).size(270, 18));
        engine.add_child_widget(frame, Button::new("Press me").at(10, 30).size(120, 24));
        engine.add_child_widget(
            frame,
            CheckBox::new("Remember me").at(140, 30).size(140, 24),
        );
        sliders
            .push(engine.add_child_widget(frame, Slider::new(0.0, 100.0).at(10, 64).size(270, 20)));
        engine.add_child_widget(
            frame,
            ProgressBar::new(100.0).at(10, 94).size(270, 16).value(40.0),
        );
        engine.add_child_widget(
            frame,
            TextField::new()
                .at(10, 118)
                .size(270, 24)
                .with_text("Jane Doe"),
        );
        count += 7;
    }

    (sliders, count)
}

/// Draws one frame and forgets what the backend recorded, so the record does not grow.
///
/// * `engine` - the engine that owns the tree.
/// * `backend` - the headless backend.
///
/// Returns whether a frame was drawn.
fn frame(engine: &mut Engine<()>, backend: &mut HeadlessBackend) -> bool {
    let drawn = engine.widgets().draw_loop(backend);

    backend.painted.clear();
    backend.draws.clear();
    backend.draw_sources.clear();
    drawn
}

/// Times dispatching pointer events across the gallery tree.
///
/// Returns nanoseconds per event.
fn event_dispatch() -> f64 {
    let mut engine: Engine<()> = Engine::new(size(WINDOW.0, WINDOW.1), 60);
    let mut backend = HeadlessBackend::new(size(WINDOW.0, WINDOW.1));

    build_gallery(&mut engine);
    frame(&mut engine, &mut backend);

    let events = 20_000;

    best_ns(events, || {
        for i in 0..events as i32 {
            let position = point(20 + (i * 7) % 900, 20 + (i * 13) % 1300);

            engine.dispatch(InputEvent::MouseMove { position }, &mut ());
            black_box(&engine);
        }
    })
}

/// Times a frame under interaction (one slider changes and the pointer moves) and a full redraw.
///
/// Returns microseconds per interactive frame, microseconds per full redraw, and the widget count.
fn frame_times() -> (f64, f64, usize) {
    let mut engine: Engine<()> = Engine::new(size(WINDOW.0, WINDOW.1), 60);
    let mut backend = HeadlessBackend::new(size(WINDOW.0, WINDOW.1));
    let (sliders, count) = build_gallery(&mut engine);

    frame(&mut engine, &mut backend);

    let frames = 2_000;
    let interactive = best_ns(frames, || {
        for i in 0..frames as usize {
            let slider = sliders[i % sliders.len()];

            if let Some(widget) = engine.widgets().get_as_mut::<Slider>(slider) {
                widget.set_value((i % 100) as f64);
            }

            engine.dispatch(
                InputEvent::MouseDown {
                    position: point(30, 80),
                    button: MouseButton::Left,
                    clicks: 1,
                },
                &mut (),
            );
            black_box(frame(&mut engine, &mut backend));
        }
    }) / 1_000.0;
    let full_frames = 200;
    let full = best_ns(full_frames, || {
        for _ in 0..full_frames {
            engine.widgets().invalidate_all();
            assert!(
                frame(&mut engine, &mut backend),
                "a full redraw draws a frame"
            );
        }
    }) / 1_000.0;

    (interactive, full, count)
}

/// Measures texture memory: the pixels of every widget texture drawn in one full frame.
///
/// Returns bytes per widget at four bytes per pixel.
fn texture_bytes_per_widget() -> f64 {
    let mut engine: Engine<()> = Engine::new(size(WINDOW.0, WINDOW.1), 60);
    let mut backend = HeadlessBackend::new(size(WINDOW.0, WINDOW.1));
    let (_, count) = build_gallery(&mut engine);

    engine.widgets().invalidate_all();
    engine.widgets().draw_loop(&mut backend);

    let bytes: u64 = backend
        .draws
        .iter()
        .map(|(_, rect)| u64::from(rect.w) * u64::from(rect.h) * 4)
        .sum();

    bytes as f64 / count as f64
}

/// Times building a form with `FORM_ROWS` label and field rows, and drawing its first frame.
///
/// Returns microseconds from an empty engine to the first drawn frame.
fn form_load() -> f64 {
    best_ns(1, || {
        let mut engine: Engine<()> = Engine::new(size(500, 1400), 60);
        let mut backend = HeadlessBackend::new(size(500, 1400));
        let sheet = engine.add_widget(Panel::new().at(0, 0).size(500, 1400));
        let mut layout = FormLayout::new().spacing(6).margins(Margin::all(6));

        for row in 0..FORM_ROWS {
            let label = engine.add_child_widget(sheet, Text::new(format!("Field {row}:")));
            let field = engine.add_child_widget(
                sheet,
                TextField::new().size(100, 24).with_text("Sample value"),
            );

            layout.add_row(label, field);
        }

        engine
            .widgets()
            .get_mut(sheet)
            .unwrap()
            .set_layout(Box::new(layout));
        black_box(frame(&mut engine, &mut backend));
    }) / 1_000.0
}

/// Times text: the first layout (font load and glyph rasterization into the atlas) and a repeat
/// layout of the same text, on SDL's dummy video driver with the software renderer.
///
/// Returns microseconds for the warm-up and nanoseconds per cached layout.
fn text_layout() -> (f64, f64) {
    // SAFETY: set before SDL starts and before any other thread reads the environment.
    unsafe { std::env::set_var("SDL_VIDEODRIVER", "dummy") };

    let sdl = sdl2::init().expect("SDL2 starts");
    let window = sdl
        .video()
        .expect("video starts")
        .window("benchmark", 240, 120)
        .build()
        .expect("a window opens");
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .expect("a canvas builds");
    let mut fonts = FontCache::new();
    let font = Font::new("", 16, FontStyle::NORMAL);
    let text = "The quick brown fox jumps over the lazy dog 0123456789";

    canvas.set_draw_color(SdlColor::WHITE);
    canvas.clear();

    let start = Instant::now();

    black_box(
        fonts
            .layout(&canvas, &font, text, 240)
            .expect("text lays out"),
    );

    let warm = start.elapsed().as_nanos() as f64 / 1_000.0;
    let repeats = 2_000;
    let cached = best_ns(repeats, || {
        for _ in 0..repeats {
            black_box(
                fonts
                    .layout(&canvas, &font, text, 240)
                    .expect("text lays out"),
            );
        }
    });

    (warm, cached)
}

/// Runs every benchmark and prints a human-readable table, then the JSON object on the last line.
fn main() {
    let dispatch = event_dispatch();
    let (interactive, full, widgets) = frame_times();
    let (atlas_warmup, layout_cached) = text_layout();
    let form = form_load();
    let texture = texture_bytes_per_widget();
    let numbers = [
        ("event_dispatch_ns", dispatch),
        ("frame_interactive_us", interactive),
        ("frame_full_us", full),
        ("text_atlas_warmup_us", atlas_warmup),
        ("text_layout_cached_ns", layout_cached),
        ("form_load_us", form),
        ("texture_bytes_per_widget", texture),
        ("gallery_widgets", widgets as f64),
    ];

    for (name, value) in numbers {
        println!("{name:>26}  {value:>14.1}");
    }

    let fields: Vec<String> = numbers
        .iter()
        .map(|(name, value)| format!("\"{name}\": {value:.1}"))
        .collect();

    println!("{{{}}}", fields.join(", "));
}
