//! Golden-image comparison shared by the rendering tests (`visual.rs`, `contacts.rs`).
//!
//! * `UPDATE_GOLDEN=1` rewrites a golden instead of comparing with it.
//! * A mismatch writes `<name>.actual.png`, `<name>.expected.png` and `<name>.diff.png` to
//!   `$PUSHROD_GOLDEN_DIFF_DIR` (default `target/golden-diff`); CI uploads that directory.

use sdl2::image::{LoadSurface, SaveSurface};
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::path::{Path, PathBuf};

/// The most a color channel may differ from the golden and still count as the same pixel.
pub const CHANNEL_TOLERANCE: u8 = 3;

/// The share of pixels (in parts per 10,000) that may differ by more than the tolerance.
pub const MAX_BAD_PER_10K: usize = 20;

/// A rendered image: tightly packed RGBA bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
    /// `w * h * 4` bytes, in R, G, B, A order.
    pub rgba: Vec<u8>,
}

/// How an image compares with its golden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// Pixels that differ by more than [`CHANNEL_TOLERANCE`] in any channel.
    pub bad: usize,
    /// Whether the images are the same size and within tolerance.
    pub matches: bool,
}

/// Compares two images with a per-channel tolerance and an allowed share of bad pixels.
///
/// # Arguments
///
/// * `actual` - What was rendered.
/// * `golden` - What was expected.
///
/// # Returns
///
/// The number of bad pixels and whether the images match.  Images of different sizes never match.
pub fn compare(actual: &Pixels, golden: &Pixels) -> Comparison {
    if (actual.w, actual.h) != (golden.w, golden.h) {
        return Comparison {
            bad: usize::MAX,
            matches: false,
        };
    }

    let bad = actual
        .rgba
        .chunks(4)
        .zip(golden.rgba.chunks(4))
        .filter(|(a, g)| {
            a.iter()
                .zip(g.iter())
                .any(|(a, g)| a.abs_diff(*g) > CHANNEL_TOLERANCE)
        })
        .count();
    let allowed = actual.rgba.len() / 4 * MAX_BAD_PER_10K / 10_000;

    Comparison {
        bad,
        matches: bad <= allowed,
    }
}

/// Builds an image that highlights where two images differ: red where they do, faded otherwise.
///
/// # Arguments
///
/// * `actual` - What was rendered.
/// * `golden` - What was expected; must be the same size as `actual`.
///
/// # Returns
///
/// The diff image.
pub fn diff_image(actual: &Pixels, golden: &Pixels) -> Pixels {
    let mut rgba = Vec::with_capacity(actual.rgba.len());

    for (a, g) in actual.rgba.chunks(4).zip(golden.rgba.chunks(4)) {
        if a.iter()
            .zip(g.iter())
            .any(|(a, g)| a.abs_diff(*g) > CHANNEL_TOLERANCE)
        {
            rgba.extend_from_slice(&[255, 0, 0, 255]);
        } else {
            rgba.extend_from_slice(&[a[0] / 4 + 190, a[1] / 4 + 190, a[2] / 4 + 190, 255]);
        }
    }

    Pixels {
        w: actual.w,
        h: actual.h,
        rgba,
    }
}

/// Writes an image as a PNG.
///
/// # Arguments
///
/// * `pixels` - The image.
/// * `path` - Where to write it; parent directories are created.
pub fn save_png(pixels: &Pixels, path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let mut bytes = pixels.rgba.clone();
    let surface = Surface::from_data(
        &mut bytes,
        pixels.w,
        pixels.h,
        pixels.w * 4,
        PixelFormatEnum::ABGR8888,
    )
    .unwrap();

    surface.save(path).unwrap();
}

/// Reads a PNG.
///
/// # Arguments
///
/// * `path` - The file.
///
/// # Returns
///
/// The image, or `None` if the file does not exist or cannot be decoded.
pub fn load_png(path: &Path) -> Option<Pixels> {
    let surface = Surface::from_file(path).ok()?;
    let surface = surface.convert_format(PixelFormatEnum::ABGR8888).ok()?;
    let (w, h, pitch) = (surface.width(), surface.height(), surface.pitch() as usize);
    let data = surface.without_lock()?;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);

    for row in 0..h as usize {
        rgba.extend_from_slice(&data[row * pitch..row * pitch + (w * 4) as usize]);
    }

    Some(Pixels { w, h, rgba })
}

/// Where goldens live.
///
/// # Returns
///
/// The `tests/golden` directory of this crate.
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Where diff images are written when a golden does not match.
///
/// # Returns
///
/// `$PUSHROD_GOLDEN_DIFF_DIR`, or `target/golden-diff` in the workspace.
pub fn diff_dir() -> PathBuf {
    std::env::var_os("PUSHROD_GOLDEN_DIFF_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/golden-diff"))
}

/// Checks an image against its golden, or rewrites the golden when `UPDATE_GOLDEN` is set.
///
/// # Arguments
///
/// * `name` - The golden's name, without extension.
/// * `actual` - What was rendered.
///
/// # Returns
///
/// `None` if it matches (or was updated), otherwise a message saying what is wrong.
pub fn check_golden(name: &str, actual: &Pixels) -> Option<String> {
    let path = golden_dir().join(format!("{name}.png"));

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        save_png(actual, &path);

        return None;
    }

    let Some(golden) = load_png(&path) else {
        save_png(actual, &diff_dir().join(format!("{name}.actual.png")));

        return Some(format!(
            "{name}: no golden at {} (run with UPDATE_GOLDEN=1)",
            path.display()
        ));
    };
    let result = compare(actual, &golden);

    if result.matches {
        return None;
    }

    let dir = diff_dir();

    save_png(actual, &dir.join(format!("{name}.actual.png")));
    save_png(&golden, &dir.join(format!("{name}.expected.png")));

    if (actual.w, actual.h) == (golden.w, golden.h) {
        save_png(
            &diff_image(actual, &golden),
            &dir.join(format!("{name}.diff.png")),
        );
    }

    Some(format!(
        "{name}: {} pixels differ from the golden (diffs in {})",
        result.bad,
        dir.display()
    ))
}
