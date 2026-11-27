// TextField Widget
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

//! # TextField
//!
//! A single line of editable text, with a caret, a selection, the clipboard, an optional input
//! mask or validator, and accessory buttons inside the edit area.
//!
//! * **Typing and editing:** typed text goes in at the caret, replacing the selection.
//!   Backspace and Delete remove a character (or the selection; with Alt or Ctrl, a word).
//! * **Caret and selection:** Left, Right, Home and End move the caret (Alt or Ctrl moves by
//!   word); with Shift they extend the selection.  Clicking places the caret, dragging selects,
//!   a double click selects a word, a triple click selects all.  The caret blinks while the
//!   field has the focus.
//! * **Input methods (IME):** the composition text is drawn underlined at the caret and stays
//!   out of [`TextField::text`] (and sends no `TextChanged`) until it is committed; a password
//!   field never shows it.  The caret is reported through
//!   [`Widget::text_input_rect`], so the platform's
//!   candidate window follows it.  See the guide *Text Input and IME*.
//! * **Clipboard:** Ctrl or Command with `A` (select all), `C` (copy), `X` (cut) and `V`
//!   (paste).  A password field neither copies nor cuts.
//! * **Restricting input:** a [`Mask`] (`"99/99/9999"`) or a [`Validator`] refuses any edit that
//!   would make the text [`Validity::Invalid`]; the edit simply does not happen.
//!   [`IntegerField`] and [`NumberField`] build fields that accept numbers within a range.
//!   `set_text` is for the application and skips these rules.
//! * **Accessories:** a glyph button at the leading and/or trailing edge, *inside* the edit
//!   area; the text never runs underneath one.  See [`Accessory`].
//! * **Label:** `with_label("Name:")` draws a label to the left of the edit area.
//!
//! It emits `TextChanged { id }` after every edit (not after `set_text`), and
//! `EditingFinished { id }` on Return and when the field loses the focus.  Return is not
//! consumed, so a default button still sees it.  A clickable accessory emits
//! `AccessoryClicked { id, which }`.

use pushrod::color::Color;
use pushrod::event::{AccessorySide, Event, EventCtx, EventMask, Propagation};
use pushrod::font::Font;
use pushrod::geometry::{Rect, point, size};
use pushrod::input::{Key, Modifiers, MouseButton};
use pushrod::paint::{self, Painter, TextJustify, TextStyle, Truncate};
use pushrod::widget::{Behavior, Widget, WidgetCore};
use std::cell::{Cell, RefCell};
use std::time::Duration;

pub mod glyphs;
pub mod validate;

use validate::{IntegerValidator, Mask, NumberValidator, Validator, Validity};

/// The space between the border and the text or an accessory, in pixels.
const PADDING: u32 = 3;

/// The space between the label and the edit area, in pixels.
const LABEL_GAP: u32 = 6;

/// How long the caret stays on, then off, in milliseconds.
const BLINK_MS: u64 = 500;

/// The character that stands in for each character of a password.
const MASK_CHAR: char = '*';

/// A button drawn inside the edit area of a [`TextField`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accessory {
    /// A cross, shown only while the field has text.  Clicking it clears the text and emits
    /// `TextChanged`.
    Clear,
    /// A magnifier.  Clicking it emits `AccessoryClicked`; ignore that event to leave it
    /// decorative.
    Search,
    /// An eye.  Clicking it shows or hides a password field's characters.
    RevealPassword,
    /// A button showing this character.  Clicking it emits `AccessoryClicked`.
    Custom(char),
}

/// Where the parts of a field are, in the field's own coordinates.
#[derive(Debug, Clone, Copy, Default)]
struct Layout {
    edit: Rect,
    leading: Rect,
    trailing: Rect,
    text: Rect,
}

/// What the last paint measured, for placing the caret from a pointer position.
#[derive(Debug, Default)]
struct Metrics {
    label_width: u32,
    widths: Vec<u32>,
    line_height: u32,
    /// The caret as drawn at the last paint, in the field's own coordinates.
    caret: Option<Rect>,
}

/// The input method's composition (pre-edit) text: shown at the caret, underlined, but not part of
/// the field's text until it is committed.
#[derive(Debug, Clone, PartialEq)]
struct Preedit {
    /// The composition text.
    chars: Vec<char>,
    /// The input method's cursor inside the composition, in characters.
    cursor: usize,
    /// How many characters the input method has selected from the cursor (zero for none).
    selected: usize,
}

/// A single-line text entry.  See the [module documentation](self) for its behavior.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::text_field::TextField;
///
/// let name = TextField::new().at(20, 20).size(240, 24).with_label("Name:").placeholder("Your name");
///
/// assert_eq!(name.text(), "");
/// ```
#[derive(Debug)]
pub struct TextField {
    core: WidgetCore,
    style: TextStyle,
    chars: Vec<char>,
    caret: usize,
    anchor: usize,
    max_length: Option<usize>,
    placeholder: String,
    label: Option<String>,
    password: bool,
    revealed: bool,
    mask: Option<Mask>,
    validator: Option<Box<dyn Validator>>,
    leading: Option<Accessory>,
    trailing: Option<Accessory>,
    focused: bool,
    caret_on: bool,
    blink: Duration,
    dragging: bool,
    scroll: Cell<i32>,
    metrics: RefCell<Metrics>,
    preedit: Option<Preedit>,
}

impl Widget for TextField {
    /// Returns the field's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the field's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Returns `true`: the engine asks the system for text input while the field has the focus.
    ///
    /// # Returns
    ///
    /// `true`.
    fn accepts_text_input(&self) -> bool {
        true
    }

    /// Draws the label, the edit area with its one-pixel border, the accessories, and the text
    /// (or the placeholder) clipped to the room between the accessories.  The selection is drawn
    /// inverted and the caret as a one-pixel line.
    ///
    /// Colors set with [`Widget::set_style`] override the background (`background`), the border
    /// and accessories (`border_color`), and the text (`foreground`).
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the field.
    fn paint(&self, p: &mut dyn Painter) {
        let bounds = self.get_size();
        let style = self.style();
        let ink = style.border_color.unwrap_or(Color::BLACK);
        let text_color = style.foreground.unwrap_or(self.style.font_color);
        let font = style.font.as_ref().unwrap_or(&self.style);

        paint::fill(p, self.get_color());

        // The label.
        let label_width = match &self.label {
            Some(label) => p.text_width(font, label) + LABEL_GAP,
            None => 0,
        }
        .min(bounds.w);

        if let Some(label) = &self.label {
            paint::text(
                p,
                Rect::new(0, 0, label_width.saturating_sub(LABEL_GAP), bounds.h),
                font,
                text_color,
                label,
                Truncate::End,
            );
        }

        let layout = self.layout(label_width);

        p.fill_rect(layout.edit, style.background.unwrap_or(Color::WHITE));
        p.rect(layout.edit, ink);

        // Measure the shown text once; the caret, the selection and the mouse all use it.
        let mut shown = self.shown();
        let preedit = self.active_preedit().cloned();
        let (insert_at, _) = self.selection();

        // The composition is drawn inline at the selection start, ahead of the text after it.
        if let Some(preedit) = &preedit {
            shown.splice(insert_at..insert_at, preedit.chars.iter().copied());
        }

        let widths: Vec<u32> = (0..=shown.len())
            .map(|n| {
                let prefix: String = shown[..n].iter().collect();

                p.text_width(font, &prefix)
            })
            .collect();
        let line_height = p.line_height(font);
        let total = widths.last().copied().unwrap_or(0) as i32;
        let room = layout.text.w as i32;

        // Keep the caret in view.
        let caret_index = match &preedit {
            Some(preedit) => insert_at + preedit.cursor.min(preedit.chars.len()),
            None => self.caret.min(shown.len()),
        };
        let caret_x = widths[caret_index] as i32;
        let mut scroll = self.scroll.get();

        if caret_x - scroll > room {
            scroll = caret_x - room;
        }

        if caret_x < scroll {
            scroll = caret_x;
        }

        scroll = scroll.clamp(0, (total - room).max(0));
        self.scroll.set(scroll);

        // The text, clipped so it never runs under an accessory.
        let origin = layout.text.x - scroll;
        let line_y = layout.edit.y + (layout.edit.h as i32 - line_height as i32) / 2;

        p.clip(Some(layout.text));

        if shown.is_empty() {
            if !self.placeholder.is_empty() {
                paint::text(
                    p,
                    layout.text,
                    font,
                    Color::GRAY,
                    &self.placeholder,
                    Truncate::End,
                );
            }
        } else {
            let line = Rect::new(
                origin,
                layout.edit.y + 1,
                total as u32 + 2,
                layout.edit.h - 2,
            );

            paint::text(
                p,
                line,
                font,
                text_color,
                &shown.iter().collect::<String>(),
                Truncate::None,
            );

            let (from, to) = self.selection();

            if let Some(preedit) = &preedit {
                // Underline the composition; the input method's selection gets a thicker line.
                let end = insert_at + preedit.chars.len();
                let y = line_y + line_height as i32 - 1;

                p.line(
                    point(origin + widths[insert_at] as i32, y),
                    point(origin + widths[end] as i32, y),
                    ink,
                );

                if preedit.selected > 0 {
                    let first = insert_at + preedit.cursor.min(preedit.chars.len());
                    let last = (first + preedit.selected).min(end);

                    p.fill_rect(
                        Rect::new(
                            origin + widths[first] as i32,
                            y - 1,
                            widths[last] - widths[first],
                            2,
                        ),
                        ink,
                    );
                }
            } else if from != to {
                let left = origin + widths[from] as i32;
                let width = widths[to] - widths[from];
                let chosen: String = shown[from..to].iter().collect();

                p.fill_rect(Rect::new(left, line_y, width, line_height), ink);
                paint::text(
                    p,
                    Rect::new(left, layout.edit.y + 1, width + 2, layout.edit.h - 2),
                    font,
                    Color::WHITE,
                    &chosen,
                    Truncate::None,
                );
            }
        }

        let caret_rect = Rect::new(origin + caret_x, line_y, 1, line_height);

        if self.focused && self.caret_on && (self.anchor == self.caret || preedit.is_some()) {
            p.fill_rect(caret_rect, ink);
        }

        p.clip(None);

        // The accessories.
        for (accessory, area) in [
            (self.leading, layout.leading),
            (self.trailing, layout.trailing),
        ] {
            if let Some(accessory) = accessory
                && self.is_shown(accessory)
            {
                self.paint_accessory(p, accessory, area, ink, font);
            }
        }

        let mut metrics = self.metrics.borrow_mut();

        metrics.label_width = label_width;
        metrics.widths = widths;
        metrics.line_height = line_height;
        metrics.caret = Some(caret_rect);
    }

    /// Returns where the caret was drawn at the last paint, so the platform's input-method
    /// candidate window can sit next to it.  With a composition showing, the caret is at the
    /// input method's cursor inside it.
    ///
    /// # Returns
    ///
    /// The caret rectangle in the field's own coordinates, or `None` before the first paint.
    fn text_input_rect(&self) -> Option<Rect> {
        self.metrics.borrow().caret
    }

    /// Edits the text for typed text and keys, places the caret and selects with the mouse, blinks
    /// the caret on animation ticks, and finishes editing on Return and when the focus leaves.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the field.
    /// * `ctx` - The context used to emit events, read and write the clipboard, ask for the next
    ///   animation tick, and turn window coordinates into the field's own.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for text, the editing keys and the mouse presses it handles;
    /// [`Propagation::Continue`] for Return, Tab, Escape and anything else, so a default button or
    /// the focus order still sees them.
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        match event {
            Event::TextComposing { text, cursor, .. } => {
                self.compose(text, *cursor);
                Propagation::Stop
            }
            Event::TextEntered { text, .. } => {
                self.preedit = None;
                self.type_text(text, ctx);
                Propagation::Stop
            }
            Event::KeyPressed { key, modifiers, .. } => self.on_key(*key, *modifiers, ctx),
            Event::PointerPressed {
                button: MouseButton::Left,
                position,
                count,
                ..
            } => {
                let local = point(position.x - ctx.origin().x, position.y - ctx.origin().y);

                self.on_press(local.x, local.y, *count, ctx);
                Propagation::Stop
            }
            Event::PointerMoved { position, .. } if self.dragging => {
                let index = self.index_at(position.x - ctx.origin().x);

                self.move_caret(index, true);
                Propagation::Stop
            }
            Event::PointerReleased {
                button: MouseButton::Left,
                ..
            } if self.dragging => {
                self.dragging = false;
                Propagation::Stop
            }
            Event::FocusGained { .. } => {
                self.focused = true;
                self.restart_blink();
                ctx.request_animation();
                Propagation::Continue
            }
            Event::FocusLost { .. } => {
                self.focused = false;
                self.dragging = false;
                self.preedit = None;
                self.anchor = self.caret;
                self.set_invalidated(true);
                ctx.emit(Event::EditingFinished { id: self.get_id() });
                Propagation::Continue
            }
            Event::Tick { elapsed } => {
                if self.focused {
                    self.blink += *elapsed;

                    if self.blink >= Duration::from_millis(BLINK_MS) {
                        self.blink = Duration::ZERO;
                        self.caret_on = !self.caret_on;
                        self.set_invalidated(true);
                    }

                    ctx.request_animation();
                }

                Propagation::Continue
            }
            _ => Propagation::Continue,
        }
    }
}

impl TextField {
    /// Creates an empty field.
    ///
    /// It is 200 by 24 pixels at the upper left-hand corner of its parent, can take keyboard
    /// focus, has no label, placeholder, mask, validator, accessories or length limit, and has a
    /// transparent background until set with `set_color` (the edit area itself is white).
    ///
    /// # Returns
    ///
    /// The new `TextField`.
    pub fn new() -> Self {
        let mut core = WidgetCore::new(point(0, 0), size(200, 24));

        core.set_event_mask(
            EventMask::POINTER_BUTTONS
                | EventMask::POINTER_MOTION
                | EventMask::KEYBOARD
                | EventMask::TEXT
                | EventMask::FOCUS
                | EventMask::TIMER,
        );

        let mut field = Self {
            core,
            style: TextStyle::new(Font::default(), Color::BLACK, TextJustify::Left),
            chars: Vec::new(),
            caret: 0,
            anchor: 0,
            max_length: None,
            placeholder: String::new(),
            label: None,
            password: false,
            revealed: false,
            mask: None,
            validator: None,
            leading: None,
            trailing: None,
            focused: false,
            caret_on: true,
            blink: Duration::ZERO,
            dragging: false,
            scroll: Cell::new(0),
            metrics: RefCell::new(Metrics::default()),
            preedit: None,
        };

        field.set_behavior(Behavior::FOCUSABLE);
        field
    }

    /// Creates the search preset: a magnifier at the leading edge, a clear button at the
    /// trailing edge, and the placeholder `Search`.  Return emits `EditingFinished`, as in every
    /// field.
    ///
    /// # Returns
    ///
    /// The new `TextField`.
    pub fn search() -> Self {
        Self::new()
            .with_leading(Accessory::Search)
            .with_trailing(Accessory::Clear)
            .placeholder("Search")
    }

    /// Sets the upper left-hand corner, relative to the parent.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels.
    /// * `y` - Vertical position in pixels.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.set_origin(point(x, y));
        self
    }

    /// Sets the width and height.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels.
    /// * `height` - Height in pixels.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Draws a label to the left of the edit area.
    ///
    /// # Arguments
    ///
    /// * `label` - The label, such as `"Name:"`.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the gray text shown while the field is empty.
    ///
    /// # Arguments
    ///
    /// * `text` - The placeholder.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Sets the starting text, skipping the mask, validator and length limit.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn with_text(mut self, text: &str) -> Self {
        self.set_text(text);
        self
    }

    /// Limits the text to `limit` characters.  Ignored with a mask, whose length is its own.
    ///
    /// # Arguments
    ///
    /// * `limit` - The most characters allowed.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn max_length(mut self, limit: usize) -> Self {
        self.max_length = Some(limit);
        self
    }

    /// Shows each character as `*`, and refuses to copy or cut.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Restricts the text to a mask such as `"99/99/9999"` (`9` digit, `A` letter, `X` any, other
    /// characters are literals that fill in as you type).  Within a mask, only the end of the
    /// text can be deleted.  Replaces any validator.
    ///
    /// # Arguments
    ///
    /// * `template` - The mask template.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn with_mask(mut self, template: &str) -> Self {
        self.mask = Some(Mask::new(template));
        self.validator = None;
        self
    }

    /// Restricts the text with a validator.  Replaces any mask.
    ///
    /// # Arguments
    ///
    /// * `validator` - The validator; edits it judges `Invalid` do not happen.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn with_validator(mut self, validator: impl Validator + 'static) -> Self {
        self.validator = Some(Box::new(validator));
        self.mask = None;
        self
    }

    /// Puts an accessory at the leading (left) edge, inside the edit area.
    ///
    /// # Arguments
    ///
    /// * `accessory` - The accessory.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn with_leading(mut self, accessory: Accessory) -> Self {
        self.leading = Some(accessory);
        self
    }

    /// Puts an accessory at the trailing (right) edge, inside the edit area.
    ///
    /// # Arguments
    ///
    /// * `accessory` - The accessory.
    ///
    /// # Returns
    ///
    /// The field, for chaining.
    #[must_use]
    pub fn with_trailing(mut self, accessory: Accessory) -> Self {
        self.trailing = Some(accessory);
        self
    }

    /// Returns the text.
    ///
    /// # Returns
    ///
    /// What the user has typed (never the `*` of a password).
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    /// Replaces the text, skipping the mask, validator and (beyond truncating to it) the length
    /// limit, and emitting nothing.  The caret goes to the end and the selection is cleared.
    ///
    /// # Arguments
    ///
    /// * `text` - The new text; line breaks and other control characters are dropped.
    pub fn set_text(&mut self, text: &str) {
        self.chars = text.chars().filter(|c| !c.is_control()).collect();

        if let Some(limit) = self.max_length {
            self.chars.truncate(limit);
        }

        self.caret = self.chars.len();
        self.anchor = self.caret;
        self.restart_blink();
    }

    /// Returns the selected text.
    ///
    /// # Returns
    ///
    /// The selection, empty if there is none.
    pub fn selected_text(&self) -> String {
        let (from, to) = self.selection();

        self.chars[from..to].iter().collect()
    }

    /// Returns where the caret is.
    ///
    /// # Returns
    ///
    /// The caret position, in characters from the start of the text.
    pub fn caret(&self) -> usize {
        self.caret
    }

    /// Judges the current text with the mask or validator.
    ///
    /// # Returns
    ///
    /// The validity; `Valid` when there is neither a mask nor a validator.
    pub fn validity(&self) -> Validity {
        let text = self.text();

        match (&self.mask, &self.validator) {
            (Some(mask), _) => mask.validate(&text),
            (None, Some(validator)) => validator.validate(&text),
            _ => Validity::Valid,
        }
    }

    /// Returns the text as a whole number.
    ///
    /// # Returns
    ///
    /// The number, or `None` if the text is not one.
    pub fn value_i64(&self) -> Option<i64> {
        self.text().parse().ok()
    }

    /// Returns the text as a decimal number.
    ///
    /// # Returns
    ///
    /// The number, or `None` if the text is not one.
    pub fn value_f64(&self) -> Option<f64> {
        self.text().parse().ok()
    }

    /// Returns whether a password field is showing its characters.
    ///
    /// # Returns
    ///
    /// `true` after the reveal accessory was clicked.
    pub fn is_revealed(&self) -> bool {
        self.revealed
    }

    /// Returns the selection, ordered.
    fn selection(&self) -> (usize, usize) {
        (self.anchor.min(self.caret), self.anchor.max(self.caret))
    }

    /// Returns the characters as they are drawn: the text, or `*` for each when masked.
    fn shown(&self) -> Vec<char> {
        if self.password && !self.revealed {
            vec![MASK_CHAR; self.chars.len()]
        } else {
            self.chars.clone()
        }
    }

    /// Returns the composition to show: none for a password field, which never displays what the
    /// input method is composing.
    fn active_preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref().filter(|_| !self.password)
    }

    /// Records the input method's composition text, or clears it when `text` is empty.  The
    /// composition stays out of `chars`, so no change is reported until it is committed.
    ///
    /// # Arguments
    ///
    /// * `text` - The composition text so far; empty when the composition ended.
    /// * `cursor` - The input method's cursor or selection as `(start, length)` in characters.
    fn compose(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        let chars: Vec<char> = text.chars().collect();
        let (cursor, selected) = cursor.unwrap_or((chars.len(), 0));
        let preedit = (!chars.is_empty()).then(|| Preedit {
            cursor: cursor.min(chars.len()),
            selected,
            chars,
        });

        if preedit != self.preedit {
            self.preedit = preedit;
            self.restart_blink();
            self.set_invalidated(true);
        }
    }

    /// Returns whether an accessory is drawn (and clickable) right now.
    fn is_shown(&self, accessory: Accessory) -> bool {
        accessory != Accessory::Clear || !self.chars.is_empty()
    }

    /// Works out where the parts are for a label of the given width.
    fn layout(&self, label_width: u32) -> Layout {
        let bounds = self.get_size();
        let edit = Rect::new(
            label_width as i32,
            0,
            bounds.w.saturating_sub(label_width),
            bounds.h,
        );
        let side = edit.h.saturating_sub(2);
        let leading = match self.leading {
            Some(_) => Rect::new(edit.x + 1, edit.y + 1, side, side),
            None => Rect::new(edit.x + 1, edit.y + 1, 0, 0),
        };
        let trailing = match self.trailing {
            Some(_) => Rect::new(
                edit.x + edit.w as i32 - 1 - side as i32,
                edit.y + 1,
                side,
                side,
            ),
            None => Rect::new(edit.x + edit.w as i32 - 1, edit.y + 1, 0, 0),
        };
        let left = leading.x + leading.w as i32 + PADDING as i32;
        let right = trailing.x - PADDING as i32;

        Layout {
            edit,
            leading,
            trailing,
            text: Rect::new(left, edit.y + 1, (right - left).max(0) as u32, side),
        }
    }

    /// Returns the layout from what the last paint measured.
    fn current_layout(&self) -> Layout {
        self.layout(self.metrics.borrow().label_width)
    }

    /// Returns the character position nearest an x coordinate in the field's own coordinates.
    fn index_at(&self, x: i32) -> usize {
        let layout = self.current_layout();
        let relative = x - layout.text.x + self.scroll.get();
        let metrics = self.metrics.borrow();
        let count = self.chars.len();

        if metrics.widths.len() != count + 1 {
            return count;
        }

        (0..=count)
            .min_by_key(|i| (metrics.widths[*i] as i32 - relative).abs())
            .unwrap_or(count)
    }

    /// Draws an accessory inside its square.
    fn paint_accessory(
        &self,
        p: &mut dyn Painter,
        accessory: Accessory,
        area: Rect,
        ink: Color,
        font: &TextStyle,
    ) {
        let soft = self.style().border_color.unwrap_or(glyphs::SOFT_INK);

        match accessory {
            Accessory::Clear => glyphs::clear(p, area),
            Accessory::Search => glyphs::search(p, area, soft),
            Accessory::RevealPassword => glyphs::eye(p, area, soft, self.revealed),
            Accessory::Custom(glyph) => {
                paint::text(
                    p,
                    area,
                    &TextStyle::new(font.font.clone(), ink, TextJustify::Center),
                    ink,
                    &glyph.to_string(),
                    Truncate::None,
                );
            }
        }
    }

    /// Makes the caret visible and starts its blink over, and repaints.
    fn restart_blink(&mut self) {
        self.caret_on = true;
        self.blink = Duration::ZERO;
        self.set_invalidated(true);
    }

    /// Moves the caret, extending the selection when `extend` is set.
    fn move_caret(&mut self, index: usize, extend: bool) {
        self.caret = index.min(self.chars.len());

        if !extend {
            self.anchor = self.caret;
        }

        self.restart_blink();
    }

    /// Returns the start of the word before `from`.
    fn word_before(&self, from: usize) -> usize {
        let mut i = from;

        while i > 0 && !self.chars[i - 1].is_alphanumeric() {
            i -= 1;
        }

        while i > 0 && self.chars[i - 1].is_alphanumeric() {
            i -= 1;
        }

        i
    }

    /// Returns the end of the word after `from`.
    fn word_after(&self, from: usize) -> usize {
        let mut i = from;

        while i < self.chars.len() && !self.chars[i].is_alphanumeric() {
            i += 1;
        }

        while i < self.chars.len() && self.chars[i].is_alphanumeric() {
            i += 1;
        }

        i
    }

    /// Judges a candidate text; `false` means the edit must be refused.
    fn acceptable(&self, candidate: &[char]) -> bool {
        let text: String = candidate.iter().collect();

        match (&self.mask, &self.validator) {
            (Some(mask), _) => mask.validate(&text) != Validity::Invalid,
            (None, Some(validator)) => validator.validate(&text) != Validity::Invalid,
            _ => self.max_length.is_none_or(|limit| candidate.len() <= limit),
        }
    }

    /// Replaces the text and caret if the edit is acceptable, and says so with `TextChanged`.
    ///
    /// # Returns
    ///
    /// `true` if the edit happened.
    fn commit(&mut self, text: Vec<char>, caret: usize, ctx: &mut EventCtx) -> bool {
        if !self.acceptable(&text) {
            return false;
        }

        let changed = text != self.chars;

        self.chars = text;
        self.caret = caret.min(self.chars.len());
        self.anchor = self.caret;
        self.restart_blink();

        if changed {
            ctx.emit(Event::TextChanged { id: self.get_id() });
        }

        true
    }

    /// Returns the text with `from..to` removed, as a mask allows it: only a cut that reaches
    /// the end of the text.
    fn without(&self, from: usize, to: usize) -> Option<Vec<char>> {
        if from == to {
            return Some(self.chars.clone());
        }

        if let Some(mask) = &self.mask {
            return (to == self.chars.len()).then(|| mask.trim(&self.chars[..from]));
        }

        let mut text = self.chars.clone();

        text.drain(from..to);
        Some(text)
    }

    /// Removes `from..to` and puts the caret at `from`.
    fn delete_range(&mut self, from: usize, to: usize, ctx: &mut EventCtx) {
        if from == to {
            return;
        }

        if let Some(text) = self.without(from, to) {
            let caret = from.min(text.len());

            self.commit(text, caret, ctx);
        }
    }

    /// Types one character at the caret, replacing the selection, if the rules allow it.
    fn type_char(&mut self, c: char, ctx: &mut EventCtx) {
        let (from, to) = self.selection();
        let Some(base) = self.without(from, to) else {
            return;
        };
        let at = from.min(base.len());

        let (text, caret) = if let Some(mask) = &self.mask {
            if at == base.len() {
                match mask.append(&base, c) {
                    Some(text) => (text.clone(), text.len()),
                    None => return,
                }
            } else {
                // Inside the text a typed character overwrites, hopping over literals.
                let mut text = base.clone();
                let mut index = at;

                while mask.literal_at(index).is_some_and(|l| l != c) {
                    index += 1;
                }

                if index >= text.len() || !mask.accepts(index, c) {
                    return;
                }

                text[index] = c;
                (text, index + 1)
            }
        } else {
            let mut text = base;

            text.insert(at, c);
            (text, at + 1)
        };

        self.commit(text, caret, ctx);
    }

    /// Types text (typed or pasted) one character at a time.
    fn type_text(&mut self, text: &str, ctx: &mut EventCtx) {
        for c in text.chars().filter(|c| !c.is_control()) {
            self.type_char(c, ctx);
        }
    }

    /// Handles a left-button press at (`x`, `y`) in the field's own coordinates.
    fn on_press(&mut self, x: i32, y: i32, count: u8, ctx: &mut EventCtx) {
        let layout = self.current_layout();
        let inside = |r: Rect| x >= r.x && x < r.x + r.w as i32 && y >= r.y && y < r.y + r.h as i32;

        for (accessory, area, side) in [
            (self.leading, layout.leading, AccessorySide::Leading),
            (self.trailing, layout.trailing, AccessorySide::Trailing),
        ] {
            if let Some(accessory) = accessory
                && inside(area)
            {
                if self.is_shown(accessory) {
                    self.click_accessory(accessory, side, ctx);
                }

                return;
            }
        }

        let index = self.index_at(x);

        match count {
            0 | 1 => {
                self.dragging = true;
                self.move_caret(index, false);
            }
            2 => {
                let at = index.min(self.chars.len());

                self.anchor = self.word_before(at.max(1).min(self.chars.len())).min(at);
                self.caret = self.word_after(self.anchor).max(at);
                self.restart_blink();
            }
            _ => {
                self.anchor = 0;
                self.caret = self.chars.len();
                self.restart_blink();
            }
        }
    }

    /// Does what an accessory does when clicked.
    fn click_accessory(&mut self, accessory: Accessory, side: AccessorySide, ctx: &mut EventCtx) {
        match accessory {
            Accessory::Clear => {
                self.chars.clear();
                self.caret = 0;
                self.anchor = 0;
                self.restart_blink();
                ctx.emit(Event::TextChanged { id: self.get_id() });
            }
            Accessory::RevealPassword => {
                self.revealed = !self.revealed;
                self.set_invalidated(true);
            }
            Accessory::Search | Accessory::Custom(_) => {
                ctx.emit(Event::AccessoryClicked {
                    id: self.get_id(),
                    which: side,
                });
            }
        }
    }

    /// Handles a key press.
    fn on_key(&mut self, key: Key, modifiers: Modifiers, ctx: &mut EventCtx) -> Propagation {
        let shift = modifiers.contains(Modifiers::SHIFT);
        let command = modifiers.intersects(Modifiers::CTRL | Modifiers::GUI);
        let word = modifiers.intersects(Modifiers::ALT | Modifiers::CTRL);
        let (from, to) = self.selection();
        let has_selection = from != to;

        match key {
            Key::Char(c) if command => {
                self.shortcut(c, ctx);
                Propagation::Stop
            }
            Key::Char(_) => Propagation::Stop,
            Key::Left => {
                let target = if has_selection && !shift {
                    from
                } else if word {
                    self.word_before(self.caret)
                } else {
                    self.caret.saturating_sub(1)
                };

                self.move_caret(target, shift);
                Propagation::Stop
            }
            Key::Right => {
                let target = if has_selection && !shift {
                    to
                } else if word {
                    self.word_after(self.caret)
                } else {
                    self.caret + 1
                };

                self.move_caret(target, shift);
                Propagation::Stop
            }
            Key::Home => {
                self.move_caret(0, shift);
                Propagation::Stop
            }
            Key::End => {
                self.move_caret(self.chars.len(), shift);
                Propagation::Stop
            }
            Key::Backspace => {
                if has_selection {
                    self.delete_range(from, to, ctx);
                } else if word {
                    self.delete_range(self.word_before(self.caret), self.caret, ctx);
                } else if self.caret > 0 {
                    self.delete_range(self.caret - 1, self.caret, ctx);
                }

                Propagation::Stop
            }
            Key::Delete => {
                if has_selection {
                    self.delete_range(from, to, ctx);
                } else if word {
                    self.delete_range(self.caret, self.word_after(self.caret), ctx);
                } else if self.caret < self.chars.len() {
                    self.delete_range(self.caret, self.caret + 1, ctx);
                }

                Propagation::Stop
            }
            Key::Return | Key::KpEnter => {
                ctx.emit(Event::EditingFinished { id: self.get_id() });
                Propagation::Continue
            }
            _ => Propagation::Continue,
        }
    }

    /// Runs a clipboard or select-all shortcut: `a`, `c`, `x` or `v`.
    fn shortcut(&mut self, c: char, ctx: &mut EventCtx) {
        let (from, to) = self.selection();
        let copyable = !self.password && from != to;

        match c.to_ascii_lowercase() {
            'a' => {
                self.anchor = 0;
                self.caret = self.chars.len();
                self.restart_blink();
            }
            'c' if copyable => ctx.set_clipboard_text(self.selected_text()),
            'x' if copyable => {
                ctx.set_clipboard_text(self.selected_text());
                self.delete_range(from, to, ctx);
            }
            'v' => {
                if let Some(text) = ctx.clipboard_text().map(str::to_string) {
                    self.type_text(&text, ctx);
                }
            }
            _ => {}
        }
    }
}

impl Default for TextField {
    /// Creates an empty field; see [`TextField::new`].
    ///
    /// # Returns
    ///
    /// The new `TextField`.
    fn default() -> Self {
        Self::new()
    }
}

/// A preset: a field that takes whole numbers from `min` to `max`.
#[derive(Debug, Clone, Copy)]
pub struct IntegerField;

impl IntegerField {
    /// Creates a field that accepts only whole numbers in a range (a `-` only when the range has
    /// negatives).  Read the number with [`TextField::value_i64`].
    ///
    /// # Arguments
    ///
    /// * `min` - The smallest value.
    /// * `max` - The largest value.
    ///
    /// # Returns
    ///
    /// The new `TextField`.
    pub fn range(min: i64, max: i64) -> TextField {
        TextField::new().with_validator(IntegerValidator::new(min, max))
    }
}

/// A preset: a field that takes decimal numbers from `min` to `max`.
#[derive(Debug, Clone, Copy)]
pub struct NumberField;

impl NumberField {
    /// Creates a field that accepts only decimal numbers in a range, with at most `decimals`
    /// digits after the point.  Read the number with [`TextField::value_f64`].
    ///
    /// # Arguments
    ///
    /// * `min` - The smallest value.
    /// * `max` - The largest value.
    /// * `decimals` - The most digits after the decimal point.
    ///
    /// # Returns
    ///
    /// The new `TextField`.
    pub fn range(min: f64, max: f64, decimals: usize) -> TextField {
        TextField::new().with_validator(NumberValidator::new(min, max, decimals))
    }
}
