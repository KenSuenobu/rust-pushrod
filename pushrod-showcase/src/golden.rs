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

//! Golden-image comparison for the tour: pixels, a tolerant comparison, a diff image, and PNG
//! reading and writing.
//!
//! The tolerance matches the one the widget goldens in `pushrod-sdl2` use, so a page that
//! differs only by antialiasing noise still matches.

use sdl2::image::{LoadSurface, SaveSurface};
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::path::Path;

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

/// Returns whether two pixels differ by more than the tolerance in any channel.
///
/// # Arguments
///
/// * `a` - One pixel's four bytes.
/// * `b` - The other pixel's four bytes.
///
/// # Returns
///
/// `true` when the pixels are visibly different.
fn differs(a: &[u8], b: &[u8]) -> bool {
    a.iter()
        .zip(b.iter())
        .any(|(a, b)| a.abs_diff(*b) > CHANNEL_TOLERANCE)
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
        .filter(|(a, g)| differs(a, g))
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
        if differs(a, g) {
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
///
/// # Returns
///
/// `Ok` once the file is written.
///
/// # Errors
///
/// Returns a message when the directory or the file cannot be written.
pub fn save_png(pixels: &Pixels, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }

    let mut bytes = pixels.rgba.clone();
    let surface = Surface::from_data(
        &mut bytes,
        pixels.w,
        pixels.h,
        pixels.w * 4,
        PixelFormatEnum::ABGR8888,
    )?;

    surface
        .save(path)
        .map_err(|e| format!("{}: {e}", path.display()))
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

/// Writes the images that explain a mismatch: what was drawn, what was expected, and where they
/// differ (the last only when the sizes agree).
///
/// # Arguments
///
/// * `dir` - The directory for the three images.
/// * `name` - The page's file stem.
/// * `actual` - What was rendered.
/// * `golden` - What was expected, or `None` when there is no golden yet.
///
/// # Returns
///
/// `Ok` once the files are written.
///
/// # Errors
///
/// Returns a message when a file cannot be written.
pub fn write_diffs(
    dir: &Path,
    name: &str,
    actual: &Pixels,
    golden: Option<&Pixels>,
) -> Result<(), String> {
    save_png(actual, &dir.join(format!("{name}.actual.png")))?;

    if let Some(golden) = golden {
        save_png(golden, &dir.join(format!("{name}.expected.png")))?;

        if (actual.w, actual.h) == (golden.w, golden.h) {
            save_png(
                &diff_image(actual, golden),
                &dir.join(format!("{name}.diff.png")),
            )?;
        }
    }

    Ok(())
}
