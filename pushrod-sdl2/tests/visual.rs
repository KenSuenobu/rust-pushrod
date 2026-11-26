//! Golden-image rendering tests (issue #70): every widget, in every visual state, is painted with
//! SDL's software renderer, read back as pixels, and compared with a PNG in `tests/golden/`.
//!
//! * The software renderer is for tests only, because it is deterministic; applications draw on
//!   the GPU.  The tests run on SDL's `dummy` video driver unless `SDL_VIDEODRIVER` is set, so CI
//!   can run them under Xvfb (`xvfb-run`) with `SDL_VIDEODRIVER=x11`.
//! * `PUSHROD_GOLDEN=1 cargo test -p pushrod-sdl2 --test visual` runs the comparison (plain
//!   `cargo test` skips it, because text antialiasing differs between platforms).
//! * `UPDATE_GOLDEN=1 cargo test -p pushrod-sdl2 --test visual` rewrites the goldens.
//! * A mismatch writes `<name>.actual.png`, `<name>.expected.png` and `<name>.diff.png` to
//!   `$PUSHROD_GOLDEN_DIFF_DIR` (default `target/golden-diff`); CI uploads that directory.
//! * The comparison itself is in `support/golden.rs`, shared with the contacts screens.
//!
//! The SDL checks share one test function because SDL's video subsystem is initialized once.

use pushrod::geometry::{Rect, point, size};
use pushrod::paint::Painter;
use pushrod::widget::{VisualState, Widget};
use pushrod_sdl2::ImageCache;
use pushrod_sdl2::Sdl2Painter;
use pushrod_sdl2::font::FontCache;
use pushrod_widgets::prelude::*;
use sdl2::image::InitFlag;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;
use support::golden::{Comparison, Pixels, check_golden, compare, diff_image};

mod support;

/// The background every widget is painted over, so transparent widgets show up in the goldens.
const BACKDROP: Color = Color::RGBA(236, 236, 236, 255);

/// Paints one widget over the backdrop and reads its pixels.
///
/// # Arguments
///
/// * `canvas` - The software canvas, at least as large as the widget.
/// * `fonts` - The shared font cache.
/// * `images` - The shared image cache.
/// * `widget` - The widget to paint.
///
/// # Returns
///
/// The widget's pixels.
fn render(
    canvas: &mut Canvas<Window>,
    fonts: &mut FontCache,
    images: &mut ImageCache,
    widget: &dyn Widget,
) -> Pixels {
    let s = widget.get_size();

    canvas.set_clip_rect(None);
    canvas.set_draw_color(BACKDROP);
    canvas.clear();

    {
        let mut painter = Sdl2Painter::new(canvas, fonts).with_images(images);

        widget.paint(&mut painter as &mut dyn Painter);
    }

    let bytes = canvas
        .read_pixels(
            Some(sdl2::rect::Rect::new(0, 0, s.w, s.h)),
            PixelFormatEnum::ABGR8888,
        )
        .unwrap();

    Pixels {
        w: s.w,
        h: s.h,
        rgba: bytes,
    }
}

/// One widget in one state.
struct Case {
    /// The golden's name.
    name: &'static str,
    /// The widget, already sized and in its state.
    widget: Box<dyn Widget>,
}

/// Builds a case with a widget in a visual state.
///
/// # Arguments
///
/// * `name` - The golden's name.
/// * `widget` - The widget.
/// * `state` - The visual state to put it in.
///
/// # Returns
///
/// The case.
fn case(name: &'static str, mut widget: impl Widget + 'static, state: VisualState) -> Case {
    widget.set_state(state);

    Case {
        name,
        widget: Box::new(widget),
    }
}

/// Every widget in every state that changes how it looks.
///
/// # Returns
///
/// The cases, in the order they are checked.
fn cases() -> Vec<Case> {
    let none = VisualState::empty();
    let sample = format!("{}/examples/assets/sample.png", env!("CARGO_MANIFEST_DIR"));
    let mut field = TextField::new().size(160, 26);

    field.set_text("Hello Pushrod");

    vec![
        case("button-normal", Button::new("OK").size(100, 28), none),
        case(
            "button-hovered",
            Button::new("OK").size(100, 28),
            VisualState::HOVERED,
        ),
        case(
            "button-pressed",
            Button::new("OK").size(100, 28),
            VisualState::PRESSED,
        ),
        case(
            "button-focused",
            Button::new("OK").size(100, 28),
            VisualState::FOCUSED,
        ),
        case(
            "button-disabled",
            Button::new("OK").size(100, 28),
            VisualState::DISABLED,
        ),
        case(
            "button-default",
            Button::new("OK").default_action().size(100, 28),
            none,
        ),
        case(
            "check-box-unchecked",
            CheckBox::new("Remember me").size(140, 24),
            none,
        ),
        case(
            "check-box-checked",
            CheckBox::new("Remember me").checked(true).size(140, 24),
            VisualState::CHECKED,
        ),
        case(
            "check-box-indeterminate",
            CheckBox::new("Remember me").indeterminate().size(140, 24),
            VisualState::INDETERMINATE,
        ),
        case(
            "check-box-disabled",
            CheckBox::new("Remember me").size(140, 24),
            VisualState::DISABLED,
        ),
        case(
            "radio-button-off",
            RadioButton::new("Small").size(120, 24),
            none,
        ),
        case(
            "radio-button-on",
            RadioButton::new("Small").checked(true).size(120, 24),
            VisualState::CHECKED,
        ),
        case(
            "radio-button-disabled",
            RadioButton::new("Small").size(120, 24),
            VisualState::DISABLED,
        ),
        case(
            "slider-horizontal",
            Slider::new(0.0, 100.0).value(40.0).size(160, 28),
            none,
        ),
        case(
            "slider-horizontal-ticks",
            Slider::new(0.0, 100.0)
                .value(40.0)
                .ticks(25.0)
                .show_value(true)
                .size(160, 40),
            none,
        ),
        case(
            "slider-vertical",
            Slider::new(0.0, 100.0).value(70.0).vertical().size(28, 120),
            none,
        ),
        case(
            "slider-disabled",
            Slider::new(0.0, 100.0).value(40.0).size(160, 28),
            VisualState::DISABLED,
        ),
        case(
            "progress-empty",
            ProgressBar::new(100.0).value(0.0).size(160, 18),
            none,
        ),
        case(
            "progress-half",
            ProgressBar::new(100.0).value(50.0).size(160, 18),
            none,
        ),
        case(
            "progress-full",
            ProgressBar::new(100.0).value(100.0).size(160, 18),
            none,
        ),
        case(
            "progress-indeterminate",
            ProgressBar::indeterminate().size(160, 18),
            none,
        ),
        case(
            "scroll-bar-vertical",
            ScrollBar::vertical().size(16, 120),
            none,
        ),
        case(
            "scroll-bar-horizontal",
            ScrollBar::horizontal().size(160, 16),
            none,
        ),
        case("separator-horizontal", Separator::horizontal(140), none),
        case(
            "separator-etched",
            Separator::horizontal(140).etched(),
            none,
        ),
        case("separator-vertical", Separator::vertical(80), none),
        case("frame", Frame::new().size(120, 60), none),
        case("panel", Panel::new().size(120, 60), none),
        case("group-box", GroupBox::new("Options").size(160, 80), none),
        case(
            "boxed-label",
            BoxedLabel::new("Total: 42").size(120, 28),
            none,
        ),
        case("text", Text::new("Hello Pushrod").size(140, 24), none),
        case("glyph-button", GlyphButton::new('+').size(28, 28), none),
        case(
            "glyph-button-pressed",
            GlyphButton::new('+').size(28, 28),
            VisualState::PRESSED,
        ),
        case("text-field", field, none),
        case(
            "text-field-empty",
            TextField::new().placeholder("Name").size(160, 26),
            none,
        ),
        case(
            "text-field-focused",
            TextField::new().placeholder("Name").size(160, 26),
            VisualState::FOCUSED,
        ),
        case("image", ImageWidget::new(sample).size(96, 64), none),
        case(
            "custom-paint",
            CustomPaint::new(point(0, 0), size(80, 40), |p, s, _| {
                p.fill_rect(
                    Rect::new(0, 0, s.w, s.h),
                    pushrod::color::Color::RGB(30, 120, 200),
                );
            }),
            none,
        ),
    ]
}

#[test]
fn every_widget_and_state_matches_its_golden_image() {
    // Text antialiasing differs between platforms and SDL_ttf builds, so the comparison runs
    // where the goldens were made (the `golden` CI job) and wherever it is asked for.
    if std::env::var_os("PUSHROD_GOLDEN").is_none() && std::env::var_os("UPDATE_GOLDEN").is_none() {
        eprintln!("skipped: set PUSHROD_GOLDEN=1 to compare widgets with tests/golden");

        return;
    }

    if std::env::var_os("SDL_VIDEODRIVER").is_none() {
        // SAFETY: set before any other thread in this test binary reads the environment.
        unsafe { std::env::set_var("SDL_VIDEODRIVER", "dummy") };
    }

    let sdl = sdl2::init().unwrap();
    let _image = sdl2::image::init(InitFlag::PNG).unwrap();
    let window = sdl
        .video()
        .unwrap()
        .window("golden", 240, 160)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().software().build().unwrap();
    let mut fonts = FontCache::new();
    let mut images = ImageCache::new();
    let mut failures = Vec::new();
    let cases = cases();

    // Rendering is deterministic: the same widget twice gives the same pixels.
    let first = render(
        &mut canvas,
        &mut fonts,
        &mut images,
        cases[0].widget.as_ref(),
    );
    let again = render(
        &mut canvas,
        &mut fonts,
        &mut images,
        cases[0].widget.as_ref(),
    );

    assert_eq!(first, again);

    for case in &cases {
        let pixels = render(&mut canvas, &mut fonts, &mut images, case.widget.as_ref());

        failures.extend(check_golden(case.name, &pixels));
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Builds a solid image.
///
/// # Arguments
///
/// * `w` - Width.
/// * `h` - Height.
/// * `rgba` - The color of every pixel.
///
/// # Returns
///
/// The image.
fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Pixels {
    Pixels {
        w,
        h,
        rgba: rgba.repeat((w * h) as usize),
    }
}

#[test]
fn identical_images_match() {
    let a = solid(10, 10, [10, 20, 30, 255]);

    assert_eq!(
        compare(&a, &a),
        Comparison {
            bad: 0,
            matches: true
        }
    );
}

#[test]
fn a_difference_within_the_tolerance_matches() {
    let a = solid(10, 10, [10, 20, 30, 255]);
    let b = solid(10, 10, [13, 17, 30, 255]);

    assert!(compare(&a, &b).matches);
}

#[test]
fn a_difference_beyond_the_tolerance_does_not_match() {
    let a = solid(10, 10, [10, 20, 30, 255]);
    let b = solid(10, 10, [10, 20, 40, 255]);
    let result = compare(&a, &b);

    assert_eq!(result.bad, 100);
    assert!(!result.matches);
}

#[test]
fn a_few_bad_pixels_in_a_large_image_are_allowed() {
    let a = solid(100, 100, [0, 0, 0, 255]);
    let mut b = a.clone();

    // 20 of 10,000 pixels is the allowed share; one more is a failure.
    for px in b.rgba.chunks_mut(4).take(20) {
        px[0] = 200;
    }

    assert!(compare(&a, &b).matches);

    b.rgba[80] = 200;

    assert!(!compare(&a, &b).matches);
}

#[test]
fn images_of_different_sizes_never_match() {
    assert!(!compare(&solid(4, 4, [0; 4]), &solid(4, 5, [0; 4])).matches);
}

#[test]
fn the_diff_image_marks_only_the_bad_pixels_red() {
    let a = solid(2, 1, [0, 0, 0, 255]);
    let mut b = a.clone();

    b.rgba[0] = 255;

    let diff = diff_image(&a, &b);

    assert_eq!(&diff.rgba[0..4], &[255, 0, 0, 255]);
    assert_ne!(&diff.rgba[4..8], &[255, 0, 0, 255]);
}
