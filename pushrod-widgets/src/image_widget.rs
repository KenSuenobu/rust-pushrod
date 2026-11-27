// Image Widget
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

//! # ImageWidget
//!
//! Shows a bitmap (PNG, BMP, or anything `SDL2_image` reads) from a file.  The backend loads each
//! file once into a shared texture cache keyed by path; the widget only chooses where the pixels
//! go, using a [`FitMode`]: [`Center`](FitMode::Center), [`Stretch`](FitMode::Stretch),
//! [`Fit`](FitMode::Fit) (keeping the aspect ratio), or [`Tile`](FitMode::Tile).  A file that
//! cannot be loaded draws a crossed box instead of failing.

use pushrod::color::Color;
use pushrod::controllers::{ClickController, ControllerEvent};
use pushrod::error::Result;
use pushrod::event::{Event, EventCtx, Propagation};
use pushrod::geometry::{Rect, Size, point, size};
use pushrod::paint::{self, Painter};
use pushrod::properties::{Property, core_properties, expect_text, set_core_property, track};
use pushrod::registry::PropertyValue;
use pushrod::widget::{Widget, WidgetCore};

/// The most tiles one paint draws, so a tiny image in a huge widget cannot stall painting.
pub const MAX_TILES: u32 = 4096;

/// How an image is placed in the widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum FitMode {
    /// Draw at its own size, centered; the parts that do not fit are cut off.
    #[default]
    Center,
    /// Scale to the whole widget, ignoring the aspect ratio.
    Stretch,
    /// Scale to the largest size that fits, keeping the aspect ratio, centered.
    Fit,
    /// Repeat at its own size from the upper left-hand corner; edge tiles are cut off.
    Tile,
}

/// One copy of the image: the part of it to draw (`None` for all) and where it goes.
pub type Placement = (Option<Rect>, Rect);

/// Works out where the image is drawn.  This is the layout the widget paints, separate from any
/// drawing so it can be tested on its own.
///
/// # Arguments
///
/// * `mode` - How the image is placed.
/// * `image` - The image's size in pixels.
/// * `area` - The widget's size in pixels.
///
/// # Returns
///
/// The copies to draw, in order; empty if the image or the area has no pixels.  [`FitMode::Tile`]
/// returns at most [`MAX_TILES`] copies.
pub fn layout(mode: FitMode, image: Size, area: Size) -> Vec<Placement> {
    if image.w == 0 || image.h == 0 || area.w == 0 || area.h == 0 {
        return Vec::new();
    }

    match mode {
        FitMode::Stretch => vec![(None, Rect::new(0, 0, area.w, area.h))],
        FitMode::Center => {
            let (sx, dx, w) = centered(image.w, area.w);
            let (sy, dy, h) = centered(image.h, area.h);
            let whole = (sx, sy, w, h) == (0, 0, image.w, image.h);
            let source = (!whole).then(|| Rect::new(sx as i32, sy as i32, w, h));

            vec![(source, Rect::new(dx as i32, dy as i32, w, h))]
        }
        FitMode::Fit => {
            // Compare image.w / area.w against image.h / area.h without floating point.
            let (w, h) = if u64::from(area.w) * u64::from(image.h)
                <= u64::from(area.h) * u64::from(image.w)
            {
                let h = u64::from(image.h) * u64::from(area.w) / u64::from(image.w);

                (area.w, (h as u32).max(1))
            } else {
                let w = u64::from(image.w) * u64::from(area.h) / u64::from(image.h);

                ((w as u32).max(1), area.h)
            };

            vec![(
                None,
                Rect::new(((area.w - w) / 2) as i32, ((area.h - h) / 2) as i32, w, h),
            )]
        }
        FitMode::Tile => {
            let mut tiles = Vec::new();
            let mut y = 0;

            'rows: while y < area.h {
                let mut x = 0;

                while x < area.w {
                    if tiles.len() as u32 >= MAX_TILES {
                        break 'rows;
                    }

                    let (w, h) = ((area.w - x).min(image.w), (area.h - y).min(image.h));
                    let source = (w < image.w || h < image.h).then(|| Rect::new(0, 0, w, h));

                    tiles.push((source, Rect::new(x as i32, y as i32, w, h)));
                    x += image.w;
                }

                y += image.h;
            }

            tiles
        }
    }
}

/// Centers one axis.
///
/// # Arguments
///
/// * `image` - The image's length on this axis.
/// * `area` - The widget's length on this axis.
///
/// # Returns
///
/// The start in the image, the start in the widget, and the length drawn.
fn centered(image: u32, area: u32) -> (u32, u32, u32) {
    if image <= area {
        (0, (area - image) / 2, image)
    } else {
        ((image - area) / 2, 0, area)
    }
}

/// A bitmap loaded from a file and drawn with a [`FitMode`].
///
/// It is display only unless made [`clickable`](ImageWidget::clickable), in which case a click
/// emits `Clicked { id, button, count }`.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::image_widget::{FitMode, ImageWidget};
///
/// let logo = ImageWidget::new("assets/logo.png")
///     .at(20, 20)
///     .size(160, 120)
///     .fit(FitMode::Fit);
/// ```
#[derive(Debug)]
pub struct ImageWidget {
    core: WidgetCore,
    path: String,
    fit: FitMode,
    clickable: bool,
    error_color: Color,
}

impl Widget for ImageWidget {
    /// Returns the image widget's shared state.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core(&self) -> &WidgetCore {
        &self.core
    }

    /// Returns the image widget's shared state for modification.
    ///
    /// # Returns
    ///
    /// The `WidgetCore`.
    fn core_mut(&mut self) -> &mut WidgetCore {
        &mut self.core
    }

    /// Lists the widget's properties: the core properties, then its own.
    ///
    /// # Returns
    ///
    /// The properties.
    fn properties(&self) -> Vec<Property> {
        let mut list = core_properties(self);

        list.push(Property::text("path", self.path()));

        list
    }

    /// Sets a property by name; the widget's own, or else a core property.
    ///
    /// # Arguments
    ///
    /// * `name` - The property name.
    /// * `value` - The new value.
    ///
    /// # Returns
    ///
    /// `true` if the value changed, `false` if the widget already had it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Property`](pushrod::error::Error::Property) if there is no such
    /// property, it is read only, or the value is the wrong kind or out of range.
    fn set_property(&mut self, name: &str, value: &PropertyValue) -> Result<bool> {
        match name {
            "path" => {
                let t = expect_text(name, value)?.to_string();

                track(self, |w| w.set_path(t))
            }
            _ => set_core_property(self, name, value),
        }
    }

    /// Fills the background, then draws the image per the fit mode, or a crossed box if the file
    /// cannot be loaded.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    fn paint(&self, p: &mut dyn Painter) {
        let area = self.get_size();
        let style = self.style();

        paint::fill(p, style.background.unwrap_or(self.get_color()));

        if area.w == 0 || area.h == 0 {
            return;
        }

        let drawn = p.image_size(&self.path).ok().and_then(|image| {
            let parts = layout(self.fit, image, area);

            (!parts.is_empty()).then_some(parts)
        });
        let loaded = drawn.is_some_and(|parts| {
            parts
                .into_iter()
                .all(|(source, bounds)| p.image_part(&self.path, source, bounds).is_ok())
        });

        if !loaded {
            self.paint_missing(p, area, style.border_color.unwrap_or(self.error_color));
        }
    }

    /// Feeds the click controller (when clickable) and emits `Clicked` on a click.
    ///
    /// # Arguments
    ///
    /// * `event` - The event sent to the widget.
    /// * `ctx` - The context used to emit `Clicked`.
    ///
    /// # Returns
    ///
    /// [`Propagation::Stop`] for a click this widget handled; otherwise [`Propagation::Continue`].
    fn handle_event(&mut self, event: &Event, ctx: &mut EventCtx) -> Propagation {
        self.feed_controllers(event, ctx)
    }

    /// Emits `Clicked { id, button, count }` when the click controller reports a click.
    ///
    /// # Arguments
    ///
    /// * `event` - What the controller reported.
    /// * `ctx` - The context used to emit `Clicked`.
    ///
    /// # Returns
    ///
    /// Always [`Propagation::Continue`].
    fn on_controller_event(&mut self, event: &ControllerEvent, ctx: &mut EventCtx) -> Propagation {
        if let ControllerEvent::Clicked { button, count } = event {
            ctx.emit(Event::Clicked {
                id: self.get_id(),
                button: *button,
                count: *count,
            });
        }

        Propagation::Continue
    }
}

impl ImageWidget {
    /// Creates an image widget, 100 by 100 pixels at the upper left-hand corner of its parent,
    /// that centers the image ([`FitMode::Center`]) and is display only.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file.  It is loaded when first painted, not here.
    ///
    /// # Returns
    ///
    /// The new `ImageWidget`.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            core: WidgetCore::new(point(0, 0), size(100, 100)),
            path: path.into(),
            fit: FitMode::Center,
            clickable: false,
            error_color: Color::BLACK,
        }
    }

    /// Draws the missing-image marker: a box with both diagonals.
    ///
    /// # Arguments
    ///
    /// * `p` - The painter, targeting the widget.
    /// * `area` - The widget's size.
    /// * `color` - The marker color.
    fn paint_missing(&self, p: &mut dyn Painter, area: Size, color: Color) {
        let (last_x, last_y) = (area.w as i32 - 1, area.h as i32 - 1);

        paint::border(p, area, color, 1);
        p.line(point(0, 0), point(last_x, last_y), color);
        p.line(point(0, last_y), point(last_x, 0), color);
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
    /// The widget, for chaining.
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
    /// The widget, for chaining.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.set_size(size(width, height));
        self
    }

    /// Sets how the image is placed.
    ///
    /// # Arguments
    ///
    /// * `fit` - The fit mode.
    ///
    /// # Returns
    ///
    /// The widget, for chaining.
    #[must_use]
    pub fn fit(mut self, fit: FitMode) -> Self {
        self.fit = fit;
        self
    }

    /// Makes the widget emit `Clicked { id, button, count }` when clicked.
    ///
    /// # Returns
    ///
    /// The widget, for chaining.
    #[must_use]
    pub fn clickable(mut self) -> Self {
        self.set_clickable();
        self
    }

    /// Sets the color of the crossed box drawn for a missing file.
    ///
    /// # Arguments
    ///
    /// * `color` - The marker color.
    ///
    /// # Returns
    ///
    /// The widget, for chaining.
    #[must_use]
    pub fn missing_color(mut self, color: Color) -> Self {
        self.error_color = color;
        self
    }

    /// Adds the click controller, once.  Later calls do nothing.
    fn set_clickable(&mut self) {
        if !self.clickable {
            self.clickable = true;
            self.core.add_controller(ClickController::new());
        }
    }

    /// Returns the image file path.
    ///
    /// # Returns
    ///
    /// The path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Changes the image file and invalidates the widget.  The new file is loaded on the next
    /// paint (and cached by the backend); a path that does not exist shows the crossed box.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file.
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
        self.set_invalidated(true);
    }

    /// Returns the fit mode.
    ///
    /// # Returns
    ///
    /// The current [`FitMode`].
    pub fn fit_mode(&self) -> FitMode {
        self.fit
    }

    /// Changes the fit mode and invalidates the widget.
    ///
    /// # Arguments
    ///
    /// * `fit` - The new fit mode.
    pub fn set_fit(&mut self, fit: FitMode) {
        self.fit = fit;
        self.set_invalidated(true);
    }

    /// Returns whether the widget emits `Clicked`.
    ///
    /// # Returns
    ///
    /// `true` if clickable.
    pub fn is_clickable(&self) -> bool {
        self.clickable
    }
}
