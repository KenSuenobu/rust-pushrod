//! Core tests that need no widget library: the engine creates its own root widget.

use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::root_widget::RootWidget;
use pushrod::widget::{ROOT_WIDGET_ID, Widget};

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

/// Regression test: creating and dropping engines on many threads at once must not crash.
///
/// SDL2_ttf's initialization and shutdown are not thread-safe; engines used to initialize it in
/// `Engine::new` and shut it down when dropped, which crashed tests running in parallel
/// ("double free or corruption").  SDL2_ttf is now initialized once per process, only when text
/// is drawn.
#[test]
fn engines_can_be_created_and_dropped_on_many_threads() {
    let threads: Vec<_> = (0..16)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..50 {
                    let engine: Engine<()> = Engine::new(size(64, 64), 30);
                    drop(engine);
                }
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn widget_cache_uses_the_backend_cache_policy() {
    use pushrod::cache::WidgetCache;
    use pushrod::paint::CachePolicy;

    let mut cache = WidgetCache::new();
    assert_eq!(cache.cache_policy(), CachePolicy::TextureCache);

    cache.set_cache_policy(CachePolicy::DirectDraw);
    assert_eq!(cache.cache_policy(), CachePolicy::DirectDraw);
}
