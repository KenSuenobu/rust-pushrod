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

//! Conformance for `docs/specs/paint.md`: the `Painter` contract and the cache policy.

use pushrod::backend::Backend;
use pushrod::backend::headless::HeadlessBackend;
use pushrod::backend::headless::{PaintCall, RecordingPainter};
use pushrod::color::Color;
use pushrod::engine::Engine;
use pushrod::geometry::size;
use pushrod::paint::{CachePolicy, Painter};
use pushrod::root_widget::RootWidget;
use pushrod::widget::Widget;

/// Paint spec, "Cache policy": the default policy is the texture cache, and a backend reports the
/// policy it uses.
#[test]
fn the_default_cache_policy_is_the_texture_cache() {
    assert_eq!(CachePolicy::default(), CachePolicy::TextureCache);
    assert_eq!(
        HeadlessBackend::new(size(10, 10)).cache_policy(),
        CachePolicy::TextureCache
    );
}

/// Paint spec, "Painter": a painter reports its policy and a usable scale factor, which is `1.0`
/// unless the backend draws at a higher density.
#[test]
fn a_painter_reports_its_policy_and_scale() {
    let mut painter = RecordingPainter::default();

    assert_eq!(painter.cache_policy(), CachePolicy::DirectDraw);
    assert_eq!(painter.scale_factor(), 1.0);

    painter.scale = 2.0;
    assert_eq!(painter.scale_factor(), 2.0);

    painter.scale = -3.0;
    assert_eq!(
        painter.scale_factor(),
        1.0,
        "an unusable scale reads as 1.0"
    );
}

/// Paint spec, "Painter": every drawing call a widget makes arrives at the painter, in order.
#[test]
fn a_widget_paint_reaches_the_painter_in_order() {
    let root = RootWidget::new(size(20, 20));
    let mut painter = RecordingPainter::default();

    root.paint(&mut painter);

    assert!(
        matches!(painter.calls.first(), Some(PaintCall::Fill(color)) if *color == Color::WHITE)
    );
}

/// Paint spec, "Texture cache": a frame blits each visible widget's cached texture, and a widget
/// is painted again only after it is invalidated.
#[test]
fn frames_are_blits_of_cached_textures() {
    let mut engine: Engine<()> = Engine::new(size(40, 30), 30);
    let mut backend = HeadlessBackend::new(size(40, 30));
    let child = engine.add_widget(RootWidget::new(size(10, 10)));

    assert!(engine.widgets().draw_loop(&mut backend));
    backend.present();

    let painted: Vec<_> = backend.painted.iter().map(|(id, _)| *id).collect();

    assert_eq!(
        painted,
        vec![0, child],
        "each widget paints once, parents first"
    );
    assert_eq!(backend.draws.len(), 2, "and is blitted once");
    assert_eq!(backend.frames, 1);
}

/// Paint spec, "Texture cache": a window expose blits every cached texture again without
/// repainting; only `repaint_all` (a scale change) paints every widget again.
#[test]
fn an_expose_reblits_and_a_scale_change_repaints() {
    let mut engine: Engine<()> = Engine::new(size(40, 30), 30);
    let mut backend = HeadlessBackend::new(size(40, 30));

    engine.add_widget(RootWidget::new(size(10, 10)));
    engine.widgets().draw_loop(&mut backend);

    let painted = backend.painted.len();
    let drawn = backend.draws.len();

    engine.widgets().invalidate_all();
    assert!(engine.widgets().draw_loop(&mut backend));
    assert_eq!(backend.painted.len(), painted, "an expose repaints nothing");
    assert_eq!(backend.draws.len(), drawn + 2, "but blits every texture");

    engine.widgets().repaint_all();
    assert!(engine.widgets().draw_loop(&mut backend));
    assert_eq!(
        backend.painted.len(),
        painted + 2,
        "a scale change repaints all"
    );
}
