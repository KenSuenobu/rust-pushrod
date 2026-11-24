//! Core tests that need no widget library: the engine creates its own root widget.

use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{Widget, ROOT_WIDGET_ID};
use sdl2::pixels::Color;

#[test]
fn engine_creates_a_white_root_widget_without_any_widget_library() {
    let mut engine: Engine<()> = Engine::new(size(320, 200), 30);
    let root = engine
        .widgets()
        .get_as::<RootWidget>(ROOT_WIDGET_ID)
        .unwrap();

    assert_eq!(root.get_color(), Color::RGBA(255, 255, 255, 255));
    assert_eq!(root.get_size().w, 320);
}

#[test]
fn set_background_recolors_the_root_widget() {
    let mut engine: Engine<()> = Engine::new(size(320, 200), 30);
    engine.set_background(Color::RGBA(200, 200, 200, 255));

    let root = engine
        .widgets()
        .get_as::<RootWidget>(ROOT_WIDGET_ID)
        .unwrap();
    assert_eq!(root.get_color(), Color::RGBA(200, 200, 200, 255));
}
