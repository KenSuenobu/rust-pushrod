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

//! A long settings form inside a `Container`: more rows than fit, shown through a viewport.
//!
//! - **Scroll** with the wheel, the scroll bar, or (after clicking the form's background)
//!   Page Up, Page Down, Home, End, and the arrow keys.
//! - **Tab** through the fields: the focused field is scrolled into view.
//! - **Top / Bottom** pan from code with `scroll_to`.
//! - **Bigger / Smaller** resizes the container: more or less of the form shows, and the form
//!   itself keeps its layout (its fields follow the viewport's width).
//! - **Move** moves the container: the form inside does not shift.
//! - The status line shows each `ViewportChanged` event.
//!
//! Run it with `cargo run -p pushrod-sdl2 --example container`.  Pass
//! `--screenshot <file.png>` to draw one frame, save it, and exit; add `scrolled`, `resized`,
//! or `moved` after the file name to capture that state.  All sample data is fictional.

use pushrod::color::Color;
use pushrod::composite::TypedParts;
use pushrod::engine::Engine;
use pushrod::geometry::{Point, Size, point, size};
use pushrod::layout::{FormLayout, Margin};
use pushrod::widget::{Widget, WidgetId};
use pushrod_sdl2::{Sdl2Backend, WindowSettings};
use pushrod_widgets::button::Button;
use pushrod_widgets::container::{Container, ContentSize};
use pushrod_widgets::frame::Frame;
use pushrod_widgets::text::Text;
use pushrod_widgets::text_field::TextField;

/// The window's size, in pixels.
const WINDOW: Size = Size { w: 560, h: 400 };

/// Where the container starts, and where **Move** puts it.
const HOME: Point = Point { x: 20, y: 60 };
const MOVED: Point = Point { x: 180, y: 90 };

/// The container's two sizes, in pixels.
const SMALL: Size = Size { w: 340, h: 200 };
const BIG: Size = Size { w: 360, h: 290 };

/// The form's background, which its labels share.
const FORM_COLOR: Color = Color::RGBA(225, 225, 225, 255);

/// The settings rows: a label and the field's starting text.
const ROWS: [(&str, &str); 14] = [
    ("Display name:", "Jane Doe"),
    ("Email:", "jane.doe@example.com"),
    ("Reply-to:", "jane.doe@example.com"),
    ("Organization:", "Example Company"),
    ("City:", "Anytown"),
    ("Incoming server:", "mail.example.com"),
    ("Incoming port:", "993"),
    ("Outgoing server:", "smtp.example.com"),
    ("Outgoing port:", "587"),
    ("Check every (minutes):", "15"),
    ("Signature:", "Sent from Anytown"),
    ("Download folder:", "/home/user/Downloads"),
    ("Archive folder:", "/home/user/Archive"),
    ("Language:", "English"),
];

/// Application state: the container's size and place.
#[derive(Default)]
struct AppState {
    big: bool,
    moved: bool,
}

/// Adds the container and its form to the window.
///
/// # Arguments
///
/// * `engine` - The engine that owns the widget tree.
///
/// # Returns
///
/// The container's ID.
fn add_form(engine: &mut Engine<AppState>) -> WidgetId {
    // The content is as wide as the viewport and as tall as the form's layout asks, so the
    // form scrolls up and down only.  The size given here is not used in that mode.
    let container = engine.add_widget(
        Container::new(HOME, SMALL, size(0, 0))
            .content_mode(ContentSize::FollowWidth { fit_height: true }),
    );
    let content = engine
        .widgets()
        .get_as::<Container>(container)
        .map_or(container, |c| c.typed_parts().content);
    let mut form = FormLayout::new()
        .spacing(8)
        .column_spacing(8)
        .margins(Margin::all(12));

    for (label, text) in ROWS {
        let mut label = Text::new(label);

        // Labels share the form's background, so they read as text drawn on the form.
        label.set_color(FORM_COLOR);

        let label = engine.add_child_widget(content, label);
        let field =
            engine.add_child_widget(content, TextField::new().size(140, 24).with_text(text));

        form.add_row(label, field);
    }

    if let Some(widget) = engine.widgets().get_mut(content) {
        widget.set_color(FORM_COLOR);
        widget.set_layout(Box::new(form));
    }

    container
}

/// Draws the example into a fresh window in one of its states, and saves the frame as an image.
///
/// # Arguments
///
/// * `engine` - The engine holding the example.
/// * `settings` - How to create the window to draw into.
/// * `path` - Where to save the image; the format follows the extension (`.png`).
/// * `variant` - `"scrolled"`, `"resized"`, or `"moved"`; anything else is the starting state.
/// * `container` - The container's ID.
/// * `outline` - The ID of the outline behind the container.
///
/// # Returns
///
/// Nothing on success.
///
/// # Errors
///
/// Returns the backend's error if the window cannot be created or the image cannot be saved.
fn save_screenshot(
    engine: &mut Engine<AppState>,
    settings: &WindowSettings,
    path: &str,
    variant: Option<&str>,
    container: WidgetId,
    outline: WidgetId,
) -> Result<(), pushrod::error::Error> {
    let mut backend = Sdl2Backend::new(settings)?;

    // The first frame lays the form out, so the content has its size before it is panned.
    engine.widgets().draw_loop(&mut backend);

    let (origin, bounds) = match variant {
        Some("resized") => (HOME, BIG),
        Some("moved") => (MOVED, SMALL),
        _ => (HOME, SMALL),
    };

    if let Some(c) = engine.widgets().get_as_mut::<Container>(container) {
        c.set_origin(origin);
        c.set_size(bounds);

        if variant == Some("scrolled") {
            c.scroll_to(point(0, 150));
        }
    }

    if let Some(frame) = engine.widgets().get_as_mut::<Frame>(outline) {
        frame.set_origin(point(origin.x - 1, origin.y - 1));
        frame.set_size(size(bounds.w + 2, bounds.h + 2));
    }

    engine.widgets().draw_loop(&mut backend);
    backend.save_screenshot(path)
}

pub fn main() -> Result<(), pushrod::error::Error> {
    let settings = WindowSettings::new("pushrod: container", WINDOW);
    let mut engine: Engine<AppState> = Engine::new(WINDOW, 30);
    let top = engine.add_widget(Button::new("Top").at(20, 20).size(80, 24));
    let bottom = engine.add_widget(Button::new("Bottom").at(110, 20).size(80, 24));
    let resize = engine.add_widget(Button::new("Bigger").at(200, 20).size(80, 24));
    let shift = engine.add_widget(Button::new("Move").at(290, 20).size(80, 24));
    let status = engine.add_widget(
        Text::new("ViewportChanged events appear here.")
            .at(20, 372)
            .size(520, 20),
    );

    // A one-pixel outline behind the container shows where its frame is.
    let outline = engine.add_widget(
        Frame::new()
            .at(HOME.x - 1, HOME.y - 1)
            .size(SMALL.w + 2, SMALL.h + 2),
    );
    let container = add_form(&mut engine);

    engine.on_click(top, move |_state, ui| {
        if let Some(c) = ui.get_mut::<Container>(container) {
            c.scroll_to(point(0, 0));
        }
    });

    engine.on_click(bottom, move |_state, ui| {
        if let Some(c) = ui.get_mut::<Container>(container) {
            c.scroll_to(point(0, i32::MAX));
        }
    });

    engine.on_click(resize, move |state, ui| {
        state.big = !state.big;

        let bounds = if state.big { BIG } else { SMALL };

        if let Some(c) = ui.get_mut::<Container>(container) {
            c.set_size(bounds);
        }

        if let Some(frame) = ui.get_mut::<Frame>(outline) {
            frame.set_size(size(bounds.w + 2, bounds.h + 2));
        }

        if let Some(button) = ui.get_mut::<Button>(resize) {
            button.set_text(if state.big { "Smaller" } else { "Bigger" }.to_string());
        }
    });

    engine.on_click(shift, move |state, ui| {
        state.moved = !state.moved;

        let origin = if state.moved { MOVED } else { HOME };

        if let Some(c) = ui.get_mut::<Container>(container) {
            c.set_origin(origin);
        }

        if let Some(frame) = ui.get_mut::<Frame>(outline) {
            frame.set_origin(point(origin.x - 1, origin.y - 1));
        }

        if let Some(button) = ui.get_mut::<Button>(shift) {
            button.set_text(if state.moved { "Move back" } else { "Move" }.to_string());
        }
    });

    engine.on_viewport_change(container, move |_state, ui, offset| {
        let visible = ui.get::<Container>(container).map(|c| c.viewport());

        if let (Some(visible), Some(text)) = (visible, ui.get_mut::<Text>(status)) {
            text.set_text(format!(
                "ViewportChanged {{ offset: ({}, {}) }}, showing {} by {}",
                offset.x, offset.y, visible.w, visible.h
            ));
        }
    });

    let args: Vec<String> = std::env::args().collect();

    if let Some(index) = args.iter().position(|a| a == "--screenshot")
        && let Some(path) = args.get(index + 1)
    {
        let variant = args.get(index + 2).map(String::as_str);

        return save_screenshot(&mut engine, &settings, path, variant, container, outline);
    }

    pushrod_sdl2::run(engine, settings, AppState::default())?;

    Ok(())
}
