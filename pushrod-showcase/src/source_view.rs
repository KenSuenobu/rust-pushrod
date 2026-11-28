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

//! The source view: the code of the selected page's `build` function, in a scrolling, selectable
//! view with a **Copy** button.
//!
//! The text comes from `build.rs`, which reads each page's own file when the showcase is
//! compiled, so the code shown is the code that built the page.  The `Source` toggle in the
//! shell swaps the page for the view; **Copy** (or Ctrl/Command+C while the view has the
//! keyboard focus) puts the whole snippet on the clipboard.  Both emit an `Activated` event.
//!
//! The documentation site's widget pages include the same snippets: they are the text of the
//! page files under `pushrod-showcase/src/pages/`.

use pushrod::cache::WidgetCache;
use pushrod::color::Color;
use pushrod::event::{Event, EventMask, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{Key, Modifiers, MouseButton};
use pushrod::paint::{self, Painter, TextJustify, TextStyle};
use pushrod::widget::{Behavior, Widget, WidgetId};
use pushrod_widgets::custom_paint::CustomPaint;

include!(concat!(env!("OUT_DIR"), "/page_sources.rs"));

/// The height of one line of code, in pixels.
pub const LINE_H: u32 = 18;
/// The space around the code, in pixels.
const PAD: i32 = 8;
/// The width of the **Copy** button, in pixels.
pub const COPY_W: u32 = 64;
/// The most columns a line is measured at when sizing the view.
const MAX_COLS: usize = 140;
/// Pixels per column used to size the view; code is drawn with its indentation kept.
const COL_W: u32 = 8;

/// Returns the text of a page file's `build` function.
///
/// # Arguments
///
/// * `file` - The page file's name, such as `"button.rs"`.
///
/// # Returns
///
/// The code; an empty string if the file has no `build` function.
pub fn build_source(file: &str) -> &'static str {
    PAGE_SOURCES
        .iter()
        .find(|(name, _)| *name == file)
        .map_or("", |(_, code)| code)
}

/// The data a code view shares between its painter and its event handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    /// The snippet.
    pub text: String,
    /// Whether the whole snippet is selected (Ctrl/Command+A).
    pub selected: bool,
}

/// Returns the size a code view needs for a snippet.
///
/// # Arguments
///
/// * `text` - The snippet.
///
/// # Returns
///
/// The width and height in pixels.
pub fn view_size(text: &str) -> (u32, u32) {
    let cols = text.lines().map(|l| l.chars().count()).max().unwrap_or(0);

    (
        cols.min(MAX_COLS) as u32 * COL_W + 2 * PAD as u32,
        text.lines().count() as u32 * LINE_H + 2 * PAD as u32,
    )
}

/// Draws a snippet: one line per row, the indentation kept, the selection highlighted.
fn paint_code(p: &mut dyn Painter, extent: pushrod::geometry::Size, code: &Code) {
    paint::fill(p, Color::WHITE);

    let style = TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left);
    let indent_w = p.text_width(&style, "0").max(1);

    for (row, line) in code.text.lines().enumerate() {
        let body = line.trim_start_matches(' ');
        let indent = (line.len() - body.len()) as u32;
        let x = PAD + (indent * indent_w) as i32;
        let y = PAD + (row as u32 * LINE_H) as i32;
        let width = extent.w.saturating_sub(x as u32);

        if code.selected && !body.is_empty() {
            let w = p.text_width(&style, body);

            p.fill_rect(Rect::new(x, y, w, LINE_H), Color::RGB(180, 208, 255));
        }

        if !body.is_empty() {
            p.text(Rect::new(x, y, width, LINE_H), &style, Color::BLACK, body);
        }
    }
}

/// Creates a code view.
///
/// Click it to give it the keyboard; **Ctrl/Command+A** selects the snippet and
/// **Ctrl/Command+C** copies it.  Copying emits `Activated`.
///
/// # Arguments
///
/// * `text` - The snippet.
///
/// # Returns
///
/// The widget, sized for the snippet.
pub fn code_view(text: &str) -> CustomPaint<Code> {
    let (w, h) = view_size(text);
    let code = Code {
        text: text.to_string(),
        selected: false,
    };
    let mut view = CustomPaint::with_data(point(0, 0), size(w, h), code, paint_code).on_event(
        |code, event, ctx| match event {
            Event::PointerPressed { .. } => {
                code.selected = false;
                Propagation::Continue
            }
            Event::KeyPressed {
                id,
                key: Key::Char(c),
                modifiers,
                ..
            } if modifiers.intersects(Modifiers::CTRL | Modifiers::GUI) => {
                match c.to_ascii_lowercase() {
                    'a' => code.selected = true,
                    'c' => {
                        ctx.set_clipboard_text(code.text.clone());
                        ctx.emit(Event::Activated { id: *id });
                    }
                    _ => return Propagation::Continue,
                }

                Propagation::Stop
            }
            _ => Propagation::Continue,
        },
    );

    view.set_behavior(Behavior::FOCUSABLE);
    view
}

/// The data of the **Copy** button.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CopyState {
    /// The snippet a click places on the clipboard.
    pub text: String,
    /// Whether the last click copied it, so the label reads *Copied*.
    pub copied: bool,
}

/// Paints the **Copy** button.
fn paint_copy(p: &mut dyn Painter, extent: pushrod::geometry::Size, state: &CopyState) {
    let fill = if state.copied {
        Color::RGB(200, 235, 205)
    } else {
        Color::RGB(230, 230, 230)
    };

    paint::fill(p, fill);
    paint::border(p, extent, Color::GRAY, 1);

    let style = TextStyle::new(Font::default(), Color::BLACK, TextJustify::Center);
    let label = if state.copied { "Copied" } else { "Copy" };

    p.text(
        Rect::new(0, 0, extent.w, extent.h),
        &style,
        Color::BLACK,
        label,
    );
}

/// Creates the **Copy** button.  Clicking it places its snippet on the clipboard and emits
/// `Activated`; the label reads *Copied* until the pointer next presses it.  Change the snippet
/// with [`CustomPaint::set_data`].
///
/// # Returns
///
/// The button, with an empty snippet.
pub fn copy_button() -> CustomPaint<CopyState> {
    CustomPaint::with_data(
        point(0, 0),
        size(COPY_W, 24),
        CopyState::default(),
        paint_copy,
    )
    .on_event(|state, event, ctx| match event {
        Event::PointerPressed {
            button: MouseButton::Left,
            ..
        } => {
            state.copied = false;
            Propagation::Stop
        }
        Event::PointerReleased {
            id,
            button: MouseButton::Left,
            ..
        } => {
            ctx.set_clipboard_text(state.text.clone());
            ctx.emit(Event::Activated { id: *id });
            state.copied = true;
            Propagation::Stop
        }
        _ => Propagation::Continue,
    })
    .with_event_mask(EventMask::POINTER_BUTTONS)
}

/// Adds a code view for `text` under `parent`.
///
/// # Arguments
///
/// * `cache` - The widget tree.
/// * `parent` - The widget to add the view to.
/// * `text` - The snippet.
///
/// # Returns
///
/// The view's ID.
pub fn add_code_view(cache: &mut WidgetCache, parent: WidgetId, text: &str) -> WidgetId {
    cache.add_child(parent, Box::new(code_view(text)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_sources_hold_the_build_function_of_each_page() {
        let code = build_source("button.rs");

        assert!(code.starts_with("fn build(&self, ui: &mut ShowcaseUi<'_>) -> WidgetId {"));
        assert!(code.ends_with('}'));
        assert!(code.contains("Button::new(\"Normal\")"));
        assert_eq!(build_source("nope.rs"), "");
    }

    #[test]
    fn a_view_is_sized_for_its_longest_line_and_line_count() {
        assert_eq!(view_size(""), (16, 16));
        assert_eq!(view_size("ab\nabcd\n"), (4 * COL_W + 16, 2 * LINE_H + 16));
    }
}
