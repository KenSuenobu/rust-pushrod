// Pushrod Widgets
// Container tests
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

//! Tests for the `Container`: a scrollable viewport onto content with its own size and 0,0.

use pushrod::backend::headless::HeadlessBackend;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::geometry::{Point, Rect, Size, point, rect, size};
use pushrod::input::{InputEvent, Key, Modifiers, MouseButton};
use pushrod::layout::Column;
use pushrod::widget::{Behavior, Widget, WidgetId};
use pushrod_widgets::container::{Container, ContainerParts, ContentSize, ScrollBars};
use pushrod_widgets::panel::Panel;
use pushrod_widgets::scroll_bar::ScrollBar;

/// The offsets reported by `ViewportChanged`, in order.
type Log = Vec<Point>;

/// A window with one container at (10, 10), logging its `ViewportChanged` events.
struct Fixture {
    engine: Engine<Log>,
    backend: HeadlessBackend,
    log: Log,
    id: WidgetId,
    parts: ContainerParts,
}

impl Fixture {
    /// Builds the window around `container` and draws the first frame.
    fn new(container: Container) -> Self {
        let mut engine: Engine<Log> = Engine::new(size(400, 400), 60);
        let id = engine.add_widget(container);
        let parts = engine
            .widgets()
            .get_as::<Container>(id)
            .unwrap()
            .typed_parts();

        engine.on_viewport_change(id, |log, _ui, offset| log.push(offset));

        let mut fixture = Self {
            engine,
            backend: HeadlessBackend::new(size(400, 400)),
            log: Log::new(),
            id,
            parts,
        };

        fixture.frame();
        fixture
    }

    /// A 200 by 100 container at (10, 10) over 300 by 600 of content.
    fn standard() -> Self {
        Self::new(Container::new(
            point(10, 10),
            size(200, 100),
            size(300, 600),
        ))
    }

    /// Draws a frame and delivers the events it queued, forgetting earlier paints and draws.
    fn frame(&mut self) {
        self.backend.painted.clear();
        self.backend.draws.clear();
        self.backend.draw_sources.clear();
        self.engine.widgets().draw_loop(&mut self.backend);
        self.engine.run_idle(&mut self.log);
    }

    /// Sends one input event.
    fn send(&mut self, event: InputEvent) {
        self.engine.dispatch(event, &mut self.log);
    }

    /// Adds a 40 by 20 child to the content at a content position.
    fn child(&mut self, x: i32, y: i32) -> WidgetId {
        self.engine
            .add_child_widget(self.parts.content, Panel::new().at(x, y).size(40, 20))
    }

    /// The container.
    fn container(&mut self) -> &mut Container {
        self.engine
            .widgets()
            .get_as_mut::<Container>(self.id)
            .unwrap()
    }

    /// Whether a `Widget` is hidden.
    fn hidden(&mut self, id: WidgetId) -> bool {
        self.engine.widgets().get(id).unwrap().is_hidden()
    }

    /// A `Widget`'s upper left-hand corner in the window.
    fn at(&mut self, id: WidgetId) -> Point {
        self.engine.widgets().absolute_origin(id)
    }
}

fn motion(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseMove {
        position: point(x, y),
    }
}

fn press(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseDown {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn release(x: i32, y: i32) -> InputEvent {
    InputEvent::MouseUp {
        position: point(x, y),
        button: MouseButton::Left,
        clicks: 1,
    }
}

fn wheel(x: i32, y: i32) -> InputEvent {
    InputEvent::Wheel { delta: point(x, y) }
}

fn key(key: Key) -> InputEvent {
    InputEvent::KeyDown {
        key,
        modifiers: Modifiers::empty(),
        repeat: false,
    }
}

#[test]
fn it_has_four_parts_and_children_live_on_the_content() {
    let mut f = Fixture::standard();
    let child = f.child(30, 40);

    assert_eq!(f.engine.widgets().parts(f.id).len(), 4);
    assert_eq!(f.container().content(), f.parts.content);
    assert_eq!(
        f.engine.widgets().get(child).unwrap().get_parent(),
        f.parts.content
    );
    // Content 0,0 is the container's upper left-hand corner until it scrolls.
    assert_eq!(f.at(child), point(40, 50));
    assert_eq!(f.container().content_size(), size(300, 600));
}

#[test]
fn panning_moves_children_on_screen_but_not_within_the_content() {
    let mut f = Fixture::standard();
    let child = f.child(30, 140);

    f.container().scroll_to(point(20, 100));
    f.frame();

    assert_eq!(f.at(child), point(10 + 30 - 20, 10 + 140 - 100));
    assert_eq!(
        f.engine.widgets().get(child).unwrap().get_origin(),
        point(30, 140)
    );
}

#[test]
fn resizing_changes_only_the_visible_area() {
    let mut f = Fixture::standard();
    let child = f.child(30, 40);

    // Both bars are shown, each taking 16 pixels.
    assert_eq!(f.container().viewport(), rect(0, 0, 184, 84));

    f.container().set_size(size(250, 300));
    f.frame();

    assert_eq!(f.container().viewport(), rect(0, 0, 234, 284));
    assert_eq!(f.container().content_size(), size(300, 600));
    assert_eq!(
        f.engine.widgets().get(child).unwrap().get_origin(),
        point(30, 40)
    );
    assert_eq!(
        f.engine.widgets().get(f.parts.content).unwrap().get_size(),
        size(234, 284)
    );
}

#[test]
fn moving_the_container_does_not_shift_the_content() {
    let mut f = Fixture::standard();
    let child = f.child(30, 40);

    f.container().scroll_to(point(0, 20));
    f.container().set_origin(point(100, 150));
    f.frame();

    assert_eq!(f.container().scroll_offset(), point(0, 20));
    assert_eq!(
        f.engine.widgets().get(child).unwrap().get_origin(),
        point(30, 40)
    );
    assert_eq!(f.at(child), point(130, 170));
    // Moving alone is not a viewport change; the pan was reported once.
    assert_eq!(f.log, vec![point(0, 20)]);
}

#[test]
fn scroll_bars_appear_only_when_needed() {
    // Content fits: no bars, no corner.
    let mut fits = Fixture::new(Container::new(
        point(10, 10),
        size(200, 100),
        size(200, 100),
    ));

    assert!(fits.hidden(fits.parts.vertical));
    assert!(fits.hidden(fits.parts.horizontal));
    assert!(fits.hidden(fits.parts.corner));
    assert_eq!(fits.container().viewport(), rect(0, 0, 200, 100));

    // Taller only: the vertical bar, which leaves the content's 150 pixels of width visible.
    let mut tall = Fixture::new(Container::new(
        point(10, 10),
        size(200, 100),
        size(150, 400),
    ));

    assert!(!tall.hidden(tall.parts.vertical));
    assert!(tall.hidden(tall.parts.horizontal));
    assert!(tall.hidden(tall.parts.corner));
    assert_eq!(tall.container().viewport(), rect(0, 0, 184, 100));

    // Taller, and exactly as wide as the frame: the vertical bar takes 16 pixels of width, so
    // the horizontal bar is needed too.
    let mut both = Fixture::new(Container::new(
        point(10, 10),
        size(200, 100),
        size(200, 400),
    ));

    assert!(!both.hidden(both.parts.vertical));
    assert!(!both.hidden(both.parts.horizontal));
    assert!(!both.hidden(both.parts.corner));
    assert_eq!(both.container().viewport(), rect(0, 0, 184, 84));
}

#[test]
fn scroll_bars_can_be_always_or_never_shown() {
    let mut always = Fixture::new(
        Container::new(point(10, 10), size(200, 100), size(50, 50)).scroll_bars(ScrollBars::Always),
    );

    assert!(!always.hidden(always.parts.vertical));
    assert!(!always.hidden(always.parts.horizontal));
    assert_eq!(always.container().viewport(), rect(0, 0, 184, 84));

    let mut never = Fixture::new(
        Container::new(point(10, 10), size(200, 100), size(300, 600))
            .scroll_bars(ScrollBars::Never),
    );

    assert!(never.hidden(never.parts.vertical));
    assert!(never.hidden(never.parts.horizontal));
    assert_eq!(never.container().get_scroll_bars(), ScrollBars::Never);

    // Without bars it still pans.
    never.container().scroll_by(40, 50);
    assert_eq!(never.container().viewport(), rect(40, 50, 200, 100));

    // Switching the policy later shows the bars.
    never.container().set_scroll_bars(ScrollBars::Auto);
    never.frame();
    assert!(!never.hidden(never.parts.vertical));
}

#[test]
fn the_scroll_bars_sit_on_the_right_and_bottom_edges() {
    let mut f = Fixture::standard();
    let cache = f.engine.widgets();
    let place = |id: WidgetId| {
        let w = cache.get(id).unwrap();

        (w.get_origin(), w.get_size())
    };

    assert_eq!(place(f.parts.content), (point(0, 0), size(184, 84)));
    assert_eq!(place(f.parts.vertical), (point(184, 0), size(16, 84)));
    assert_eq!(place(f.parts.horizontal), (point(0, 84), size(184, 16)));
    assert_eq!(place(f.parts.corner), (point(184, 84), size(16, 16)));
}

#[test]
fn the_offset_is_clamped_to_the_content() {
    let mut f = Fixture::standard();

    // The viewport is 184 by 84 over 300 by 600: the offset runs to (116, 516).
    f.container().scroll_to(point(5000, 5000));
    assert_eq!(f.container().scroll_offset(), point(116, 516));

    f.container().scroll_to(point(-5, -5));
    assert_eq!(f.container().scroll_offset(), point(0, 0));

    f.container().scroll_by(10, 30);
    f.container().scroll_by(5, -10);
    assert_eq!(f.container().scroll_offset(), point(15, 20));

    f.container().scroll_by(i32::MAX, i32::MIN);
    assert_eq!(f.container().scroll_offset(), point(116, 0));
}

#[test]
fn shrinking_the_content_pulls_the_offset_back() {
    let mut f = Fixture::standard();

    f.container().scroll_to(point(0, 516));
    f.frame();
    f.log.clear();

    f.container().set_content_size(size(300, 200));
    f.frame();

    assert_eq!(f.container().scroll_offset(), point(0, 116));
    assert_eq!(f.log, vec![point(0, 116)]);
}

#[test]
fn ensure_visible_pans_as_little_as_possible() {
    let mut f = Fixture::standard();

    // Already visible: nothing happens.
    f.container().ensure_visible(rect(10, 10, 40, 20));
    assert_eq!(f.container().scroll_offset(), point(0, 0));

    // Below and to the right: its far edges come to the viewport's far edges.
    f.container().ensure_visible(rect(200, 300, 40, 20));
    assert_eq!(f.container().scroll_offset(), point(240 - 184, 320 - 84));

    // Above and to the left: its near edges come to the viewport's near edges.
    f.container().ensure_visible(rect(20, 30, 40, 20));
    assert_eq!(f.container().scroll_offset(), point(20, 30));

    // Larger than the viewport: shown from its upper left-hand corner.
    f.container().ensure_visible(rect(50, 100, 250, 400));
    assert_eq!(f.container().scroll_offset(), point(50, 100));
}

#[test]
fn scrolling_from_code_emits_viewport_changed_once() {
    let mut f = Fixture::standard();

    assert!(f.log.is_empty());

    f.container().scroll_to(point(0, 40));
    f.container().scroll_to(point(0, 60));
    f.frame();
    assert_eq!(f.log, vec![point(0, 60)]);

    // The same offset again changes nothing.
    f.container().scroll_to(point(0, 60));
    f.frame();
    assert_eq!(f.log, vec![point(0, 60)]);
}

#[test]
fn resizing_emits_viewport_changed() {
    let mut f = Fixture::standard();

    f.container().set_size(size(220, 120));
    f.frame();

    assert_eq!(f.log, vec![point(0, 0)]);
}

#[test]
fn the_wheel_pans_over_the_container_and_its_children() {
    let mut f = Fixture::standard();
    let child = f.child(20, 20);

    f.frame();

    // Over a child: the scroll bubbles up to the container.  One notch is three line steps.
    f.send(motion(40, 40));
    assert_eq!(f.engine.widgets().widget_at(40, 40), child);
    f.send(wheel(0, -1));
    assert_eq!(f.container().scroll_offset(), point(0, 60));
    assert_eq!(f.log, vec![point(0, 60)]);

    // Away from the user scrolls back up; sideways scrolls sideways.
    f.send(wheel(1, 1));
    assert_eq!(f.container().scroll_offset(), point(60, 0));
    assert_eq!(f.log, vec![point(0, 60), point(60, 0)]);

    // At the top, scrolling up changes nothing and reports nothing.
    f.send(wheel(0, 1));
    assert_eq!(f.log.len(), 2);

    // Drawing afterward does not report the same pan again.
    f.frame();
    assert_eq!(f.log.len(), 2);
}

#[test]
fn a_wheel_at_the_end_passes_to_the_container_around_it() {
    let mut f = Fixture::standard();
    let inner = f.engine.add_child_widget(
        f.parts.content,
        Container::new(point(0, 0), size(100, 60), size(80, 100)),
    );

    f.frame();
    f.send(motion(30, 30));

    // The inner container scrolls to its end (40) first.
    f.send(wheel(0, -1));
    assert_eq!(
        f.engine
            .widgets()
            .get_as::<Container>(inner)
            .unwrap()
            .scroll_offset(),
        point(0, 40)
    );
    assert_eq!(f.container().scroll_offset(), point(0, 0));

    // Then the outer one takes over.
    f.send(wheel(0, -1));
    assert_eq!(f.container().scroll_offset(), point(0, 60));
}

#[test]
fn keys_pan_while_the_container_has_the_focus() {
    let mut f = Fixture::standard();

    // Without the focus the keys do nothing.
    f.send(key(Key::PageDown));
    assert_eq!(f.container().scroll_offset(), point(0, 0));

    assert!(f.engine.widgets().set_focus(f.id));

    f.send(key(Key::PageDown));
    assert_eq!(f.container().scroll_offset(), point(0, 84));
    f.send(key(Key::Down));
    assert_eq!(f.container().scroll_offset(), point(0, 104));
    f.send(key(Key::Up));
    f.send(key(Key::Right));
    assert_eq!(f.container().scroll_offset(), point(20, 84));
    f.send(key(Key::Left));
    f.send(key(Key::PageUp));
    assert_eq!(f.container().scroll_offset(), point(0, 0));
    f.send(key(Key::End));
    assert_eq!(f.container().scroll_offset(), point(0, 516));
    f.send(key(Key::Home));
    assert_eq!(f.container().scroll_offset(), point(0, 0));

    assert_eq!(f.log.len(), 8);
}

#[test]
fn a_press_on_the_content_focuses_the_container() {
    let mut f = Fixture::standard();

    f.send(motion(50, 50));
    f.send(press(50, 50));
    f.send(release(50, 50));

    assert_eq!(f.engine.widgets().focused(), Some(f.id));
}

#[test]
fn keys_for_a_focused_child_do_not_pan() {
    let mut f = Fixture::standard();
    let mut field = Panel::new().at(10, 10).size(40, 20);

    field.set_behavior(Behavior::FOCUSABLE);

    let field = f.engine.add_child_widget(f.parts.content, field);

    f.frame();
    assert!(f.engine.widgets().set_focus(field));
    f.send(key(Key::PageDown));

    assert_eq!(f.container().scroll_offset(), point(0, 0));
}

#[test]
fn a_scroll_bar_arrow_pans_by_a_line() {
    let mut f = Fixture::standard();
    let bar = f
        .engine
        .widgets()
        .get_as::<ScrollBar>(f.parts.vertical)
        .unwrap()
        .typed_parts();
    let arrow = f.at(bar.down);

    f.send(motion(arrow.x + 4, arrow.y + 4));
    f.send(press(arrow.x + 4, arrow.y + 4));
    f.send(release(arrow.x + 4, arrow.y + 4));
    f.frame();

    assert_eq!(f.container().scroll_offset(), point(0, 20));
    assert_eq!(f.log, vec![point(0, 20)]);
}

#[test]
fn dragging_a_thumb_pans_and_the_thumb_follows_code() {
    let mut f = Fixture::standard();

    // From code: the bar's value follows the offset.
    f.container().scroll_to(point(0, 258));
    f.frame();

    let value = f
        .engine
        .widgets()
        .get_as::<ScrollBar>(f.parts.vertical)
        .unwrap()
        .get_value();

    assert!((value - 0.5).abs() < 1e-9);

    // From the user: the offset follows the bar.  Drag the thumb far past the end.
    let bar = f
        .engine
        .widgets()
        .get_as::<ScrollBar>(f.parts.vertical)
        .unwrap()
        .typed_parts();
    let thumb = f.at(bar.thumb);

    f.send(motion(thumb.x + 3, thumb.y + 3));
    f.send(press(thumb.x + 3, thumb.y + 3));
    f.send(motion(thumb.x + 3, thumb.y + 300));
    f.send(release(thumb.x + 3, thumb.y + 300));
    f.frame();

    assert_eq!(f.container().scroll_offset(), point(0, 516));
    assert_eq!(f.log.last(), Some(&point(0, 516)));
}

#[test]
fn the_thumb_shows_how_much_content_is_visible() {
    let mut f = Fixture::standard();
    let bar = f
        .engine
        .widgets()
        .get_as::<ScrollBar>(f.parts.vertical)
        .unwrap();

    // 84 of 600 pixels are visible; an arrow moves 20 of the 516 that do not fit.
    assert!((bar.get_page_size() - 84.0 / 600.0).abs() < 1e-9);
    assert!((bar.get_step() - 20.0 / 516.0).abs() < 1e-9);

    f.container().set_line_step(43);
    f.frame();

    let bar = f
        .engine
        .widgets()
        .get_as::<ScrollBar>(f.parts.vertical)
        .unwrap();

    assert!((bar.get_step() - 43.0 / 516.0).abs() < 1e-9);
    assert_eq!(f.container().get_line_step(), 43);
}

#[test]
fn children_are_clipped_to_and_hit_through_the_viewport() {
    let mut f = Fixture::standard();
    let near = f.child(10, 70);
    let far = f.child(10, 300);

    f.frame();

    // The near child starts at window y 80 and the viewport ends at 94: 14 rows show.
    assert_eq!(
        f.engine.widgets().visible_rect(near),
        Some(Rect::new(20, 80, 40, 14))
    );
    assert_eq!(f.engine.widgets().visible_rect(far), None);
    assert_eq!(f.engine.widgets().widget_at(25, 85), near);
    // Its cropped rows lie under the horizontal scroll bar.
    assert_ne!(f.engine.widgets().widget_at(25, 97), near);

    // After panning, the far child is the one under the same window point.
    f.container().scroll_to(point(0, 230));
    f.frame();

    assert_eq!(f.engine.widgets().visible_rect(near), None);
    assert_eq!(f.engine.widgets().widget_at(25, 85), far);
}

#[test]
fn panning_repaints_nothing_and_draws_only_visible_children() {
    let mut f = Fixture::standard();
    let near = f.child(10, 10);
    let far = f.child(10, 300);

    f.frame();
    f.container().scroll_to(point(0, 280));
    f.frame();

    // No child was repainted: the frame only copied textures.
    assert!(
        f.backend
            .painted
            .iter()
            .all(|(id, _)| *id != near && *id != far)
    );

    let drawn: Vec<WidgetId> = f.backend.draws.iter().map(|(id, _)| *id).collect();

    assert!(drawn.contains(&far));
    assert!(!drawn.contains(&near));
    // The far child is drawn where the offset puts it.
    assert!(f.backend.draws.contains(&(far, Rect::new(20, 30, 40, 20))));
}

#[test]
fn a_cropped_child_is_drawn_from_the_part_of_its_texture_that_shows() {
    let mut f = Fixture::standard();
    let child = f.child(10, 10);

    f.container().scroll_to(point(0, 20));
    f.frame();

    // Its top 10 rows are above the viewport.
    assert!(
        f.backend
            .draws
            .contains(&(child, Rect::new(20, 10, 40, 10)))
    );
    assert!(
        f.backend
            .draw_sources
            .contains(&(child, Rect::new(0, 10, 40, 10)))
    );
}

#[test]
fn content_far_larger_than_a_texture_works() {
    let mut f = Fixture::new(Container::new(
        point(10, 10),
        size(200, 100),
        size(1_000_000, 2_000_000),
    ));
    let child = f.child(900_000, 1_900_000);

    f.container()
        .ensure_visible(rect(900_000, 1_900_000, 40, 20));
    f.frame();

    assert!(f.engine.widgets().visible_rect(child).is_some());

    // No `Widget` is anywhere near the content's size: the content part is only the viewport.
    let cache = f.engine.widgets();

    for id in 0..cache.len() as WidgetId {
        let Size { w, h } = cache.get(id).unwrap().get_size();

        assert!(w <= 400 && h <= 400, "widget {id} is {w} by {h}");
    }
}

#[test]
fn fit_children_grows_with_the_children() {
    let mut f = Fixture::new(
        Container::new(point(10, 10), size(200, 100), size(0, 0))
            .content_mode(ContentSize::FitChildren),
    );

    assert_eq!(f.container().content_size(), size(0, 0));
    assert!(f.hidden(f.parts.vertical));

    let child = f.child(60, 280);

    f.frame();
    assert_eq!(f.container().content_size(), size(100, 300));
    assert!(!f.hidden(f.parts.vertical));

    // Moving a child changes the extent.
    f.engine
        .widgets()
        .get_mut(child)
        .unwrap()
        .set_origin(point(260, 30));
    f.frame();
    assert_eq!(f.container().content_size(), size(300, 50));
    assert!(f.hidden(f.parts.vertical));
    assert!(!f.hidden(f.parts.horizontal));

    // A hidden child takes no room.
    f.engine.widgets().get_mut(child).unwrap().set_hidden(true);
    f.frame();
    assert_eq!(f.container().content_size(), size(0, 0));
    assert_eq!(f.container().get_content_mode(), ContentSize::FitChildren);
}

#[test]
fn follow_width_keeps_the_content_as_wide_as_the_viewport() {
    let mut f = Fixture::new(
        Container::new(point(10, 10), size(200, 100), size(999, 400))
            .content_mode(ContentSize::FollowWidth { fit_height: false }),
    );

    assert_eq!(f.container().content_size(), size(184, 400));
    assert!(f.hidden(f.parts.horizontal));

    f.container().set_size(size(300, 100));
    f.frame();
    assert_eq!(f.container().content_size(), size(284, 400));

    // Tall enough to show everything: the vertical bar goes, and its width comes back.
    f.container().set_size(size(300, 400));
    f.frame();
    assert_eq!(f.container().content_size(), size(300, 400));
    assert!(f.hidden(f.parts.vertical));
}

#[test]
fn follow_width_can_fit_the_height_of_the_children() {
    let mut f = Fixture::new(
        Container::new(point(10, 10), size(200, 100), size(0, 0))
            .content_mode(ContentSize::FollowWidth { fit_height: true }),
    );

    f.child(0, 230);
    f.frame();

    assert_eq!(f.container().content_size(), size(184, 250));
}

#[test]
fn a_layout_manager_arranges_the_content() {
    let mut f = Fixture::new(
        Container::new(point(10, 10), size(200, 100), size(0, 0))
            .content_mode(ContentSize::FollowWidth { fit_height: true }),
    );
    let rows: Vec<WidgetId> = (0..10).map(|_| f.child(0, 0)).collect();

    f.engine
        .widgets()
        .get_mut(f.parts.content)
        .unwrap()
        .set_layout(Box::new(Column::new().spacing(10)));
    f.frame();

    // Ten rows of 20 pixels with nine gaps of 10: the content is as tall as the column asks.
    assert_eq!(f.container().content_size(), size(184, 290));
    assert!(!f.hidden(f.parts.vertical));
    assert_eq!(
        f.engine.widgets().get(rows[9]).unwrap().get_origin(),
        point(0, 270)
    );

    // The last row is reachable.  The content never scrolls sideways, so there is no
    // horizontal bar and the viewport is the full 100 pixels tall.
    f.container().scroll_to(point(0, 9999));
    f.frame();
    assert_eq!(f.container().scroll_offset(), point(0, 190));
    assert!(f.engine.widgets().visible_rect(rows[9]).is_some());
}

#[test]
fn a_focused_child_is_scrolled_into_view() {
    let mut f = Fixture::standard();
    let mut field = Panel::new().at(10, 400).size(40, 20);

    field.set_behavior(Behavior::FOCUSABLE);

    let field = f.engine.add_child_widget(f.parts.content, field);

    f.frame();
    assert_eq!(f.engine.widgets().visible_rect(field), None);

    assert!(f.engine.widgets().set_focus(field));
    f.frame();

    // Its bottom edge comes to the bottom of the viewport.
    assert_eq!(f.container().scroll_offset(), point(0, 420 - 84));
    assert!(f.engine.widgets().visible_rect(field).is_some());
    assert_eq!(f.log, vec![point(0, 336)]);

    // Panning away afterward is not undone while the focus stays put.
    f.container().scroll_to(point(0, 0));
    f.frame();
    assert_eq!(f.container().scroll_offset(), point(0, 0));
}

#[test]
fn focus_on_a_scroll_bar_part_does_not_pan() {
    let mut f = Fixture::standard();

    f.container().scroll_to(point(0, 100));
    f.frame();

    let arrow = f
        .engine
        .widgets()
        .get_as::<ScrollBar>(f.parts.vertical)
        .unwrap()
        .typed_parts()
        .up;

    f.engine
        .widgets()
        .get_mut(arrow)
        .unwrap()
        .set_behavior(Behavior::FOCUSABLE);
    assert!(f.engine.widgets().set_focus(arrow));
    f.frame();

    assert_eq!(f.container().scroll_offset(), point(0, 100));
}

#[test]
fn the_parts_before_the_container_is_added_are_its_own_id() {
    let container = Container::new(point(0, 0), size(100, 100), size(100, 100));
    let parts = container.typed_parts();

    assert_eq!(parts.content, container.get_id());
    assert_eq!(parts.corner, container.get_id());
}
