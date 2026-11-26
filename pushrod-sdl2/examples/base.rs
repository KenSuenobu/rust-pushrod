use pushrod::appearance::StateStyle;
use pushrod::color::{Color, FontStyle};
use pushrod::engine::Engine;
use pushrod::event::{EngineEvent, EngineEventKind, Event, EventMask, EventTarget, MaskScope};
use pushrod::font::Font;
use pushrod::geometry::{point, size};
use pushrod::widget::{Behavior, VisualState, Widget};
use pushrod_sdl2::WindowSettings;
use pushrod_widgets::base_widget::BaseWidget;
use pushrod_widgets::box_widget::BoxWidget;
use pushrod_widgets::button_widget::ButtonWidget;
use pushrod_widgets::text_widget::{TextJustify, TextWidget};

/// A custom event posted by the background thread below: how far along the "download" is.
struct DownloadProgress {
    percent: u8,
}

/// Application state, passed mutably to every callback.
#[derive(Default)]
struct AppState {
    clicks: u32,
    hover_enabled: bool,
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let mut engine: Engine<AppState> = Engine::new(size(800, 600), 30);

    // A gray panel.  Every other widget is a child of the panel, so their origins are relative
    // to the panel's upper left-hand corner (20, 20).
    let mut panel = BaseWidget::new(point(20, 20), size(760, 560));
    panel.set_color(Color::RGBA(127, 127, 127, 255));
    let panel_id = engine.add_widget(panel);

    let mut welcome = TextWidget::new(
        point(20, 10),
        size(180, 18),
        Font::default(),
        Color::WHITE,
        TextJustify::Left,
        String::from("Welcome to Pushrod!"),
    );
    welcome.set_color(Color::RGBA(127, 127, 127, 255));
    engine.add_child_widget(panel_id, welcome);

    let mut box_widget1 = BoxWidget::new(point(20, 40), size(100, 100), Color::BLUE, 3);
    box_widget1.set_color(Color::CYAN);
    engine.add_child_widget(panel_id, box_widget1);

    let mut box_widget2 = BoxWidget::new(point(160, 40), size(100, 100), Color::GREEN, 5);
    box_widget2.set_color(Color::GRAY);
    engine.add_child_widget(panel_id, box_widget2);

    let mut box_widget3 = BoxWidget::new(point(300, 40), size(100, 100), Color::RED, 10);
    box_widget3.set_color(Color::MAGENTA);
    engine.add_child_widget(panel_id, box_widget3);

    let button_id = engine.add_child_widget(
        panel_id,
        ButtonWidget::new(
            point(20, 160),
            size(120, 20),
            Font::default(),
            Color::BLACK,
            TextJustify::Center,
            String::from("Click me"),
            2,
        ),
    );

    let mut status = TextWidget::new(
        point(160, 161),
        size(300, 18),
        Font::default(),
        Color::WHITE,
        TextJustify::Left,
        String::from("Not clicked yet."),
    );
    status.set_color(Color::RGBA(127, 127, 127, 255));
    let status_id = engine.add_child_widget(panel_id, status);

    // A disabled button is drawn dimmed and receives no input.
    let mut disabled = ButtonWidget::new(
        point(20, 240),
        size(120, 20),
        Font::default(),
        Color::BLACK,
        TextJustify::Center,
        String::from("Disabled"),
        2,
    );
    disabled.set_enabled(false);
    engine.add_child_widget(panel_id, disabled);

    // Per-state appearance: the button turns blue while the pointer is over it, and dark blue
    // while it is pressed.  Everything else about its look stays at the defaults.
    let mut styled = ButtonWidget::new(
        point(20, 320),
        size(120, 20),
        Font::default(),
        Color::BLACK,
        TextJustify::Center,
        String::from("Hover me"),
        2,
    );
    styled.set_style(
        VisualState::HOVERED,
        StateStyle {
            background: Some(Color::BLUE),
            foreground: Some(Color::WHITE),
            ..StateStyle::default()
        },
    );
    styled.set_style(
        VisualState::PRESSED,
        StateStyle {
            background: Some(Color::RGB(0, 0, 128)),
            foreground: Some(Color::WHITE),
            ..StateStyle::default()
        },
    );
    engine.add_child_widget(panel_id, styled);

    let quit_id = engine.add_child_widget(
        panel_id,
        ButtonWidget::new(
            point(20, 200),
            size(120, 20),
            Font::default(),
            Color::BLACK,
            TextJustify::Center,
            String::from("Quit"),
            2,
        ),
    );

    // Engine events: a status line that reports what Pushrod does internally.
    let mut engine_status = TextWidget::new(
        point(160, 201),
        size(560, 18),
        Font::default(),
        Color::WHITE,
        TextJustify::Left,
        String::from("Engine events appear here."),
    );
    engine_status.set_color(Color::RGBA(127, 127, 127, 255));
    let engine_status_id = engine.add_child_widget(panel_id, engine_status);

    // Fonts: `Font::default()` above needs no files.  A registered family is used by name; this
    // one is registered from bytes, and it becomes the default for widgets created after it.
    engine
        .fonts()
        .load_bytes(
            "Body",
            include_bytes!("../../assets/OpenSans-Regular.ttf").to_vec(),
        )
        .unwrap();

    let mut font_status = TextWidget::new(
        point(160, 281),
        size(560, 24),
        Font::new("Body", 18, FontStyle::BOLD),
        Color::WHITE,
        TextJustify::Left,
        String::from("Font \"Body\" registered with engine.fonts().load_bytes"),
    );
    font_status.set_color(Color::RGBA(127, 127, 127, 255));
    engine.add_child_widget(panel_id, font_status);

    // Toggling the run-loop hover mask is an internal action, so it emits `EventMaskChanged`.
    let hover_id = engine.add_child_widget(
        panel_id,
        ButtonWidget::new(
            point(20, 280),
            size(120, 20),
            Font::default(),
            Color::BLACK,
            TextJustify::Center,
            String::from("Toggle hover"),
            2,
        ),
    );

    // Custom events: a background thread posts progress, and the UI thread shows it.
    let mut task_status = TextWidget::new(
        point(160, 241),
        size(560, 18),
        Font::default(),
        Color::WHITE,
        TextJustify::Left,
        String::from("Background task: starting..."),
    );
    task_status.set_color(Color::RGBA(127, 127, 127, 255));
    let task_status_id = engine.add_child_widget(panel_id, task_status);

    engine.on_custom::<DownloadProgress, _>(move |_state, ui, _source, progress| {
        ui.get_mut::<TextWidget>(task_status_id)
            .unwrap()
            .set_text(format!("Background task: {}% done", progress.percent));
    });

    let sender = engine.sender();

    std::thread::spawn(move || {
        for percent in (0..=100).step_by(10) {
            sender.post(EventTarget::ApplicationOnly, DownloadProgress { percent });
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    // Behavior flags are stored on the widget; Return activation is added with focus support.
    engine
        .widgets()
        .get_mut(quit_id)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE | Behavior::CANCEL_ACTION);

    // Visual Basic-style event procedures: each callback gets the application state, and a
    // `Ui` handle to find and change any widget by its ID.
    engine.on_click(button_id, move |state, ui| {
        state.clicks += 1;

        ui.get_mut::<TextWidget>(status_id)
            .unwrap()
            .set_text(format!("Clicked {} time(s).", state.clicks));
    });

    engine.on_click(quit_id, |_state, ui| ui.shutdown());

    // Per-kind callbacks: pointer-down on the button shows a message before the click completes.
    engine.on_press(button_id, move |_state, ui| {
        ui.get_mut::<TextWidget>(status_id)
            .unwrap()
            .set_text(String::from("Button pressed..."));
    });

    engine.on_click(hover_id, |state, ui| {
        state.hover_enabled = !state.hover_enabled;
        ui.set_event_mask(if state.hover_enabled {
            EventMask::ALL
        } else {
            EventMask::ALL & !EventMask::HOVER
        });
    });

    // The status line asks for window events, so focus and resize broadcasts reach it: click
    // outside the window and back, or resize the window, to see `EventBroadcast`.
    engine
        .widgets()
        .set_event_mask(engine_status_id, EventMask::WINDOW);

    // Mask changes and broadcasts; the text of the status line shows the latest one.
    engine.on_engine_event_filtered(
        &[
            EngineEventKind::EventMaskChanged,
            EngineEventKind::EventBroadcast,
        ],
        move |_state, ui, event| {
            let text = match event {
                EngineEvent::EventMaskChanged { scope, new, .. } => {
                    let who = match scope {
                        MaskScope::RunLoop => String::from("window"),
                        MaskScope::Widget(id) => format!("widget {}", id),
                    };

                    format!("{} mask is now {:?}", who, new)
                }
                EngineEvent::EventBroadcast { kind, recipients } => {
                    format!("{:?} broadcast to {} widget(s)", kind, recipients)
                }
                _ => return,
            };

            ui.get_mut::<TextWidget>(engine_status_id)
                .unwrap()
                .set_text(text);
        },
    );

    // Receives every event generated by every widget.
    engine.on_event(|_state, _ui, event| match event {
        Event::Clicked { id, count, .. } => {
            eprintln!("[example] Widget {} clicked ({} click(s))", id, count);
        }
        Event::Engine(engine_event) => eprintln!("[example] Engine event: {:?}", engine_event),
        _ => {}
    });

    let state = pushrod_sdl2::run(
        engine,
        WindowSettings::new("pushrod", size(800, 600)),
        AppState {
            hover_enabled: true,
            ..AppState::default()
        },
    )?;

    eprintln!("[example] Exiting after {} click(s).", state.clicks);

    Ok(())
}
