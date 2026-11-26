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

//! Shared font handles and a built-in default font.
//!
//! A [`Font`] names a registered family and carries a size and style.  It is cheap to clone, and
//! widgets take one instead of a file path.  Fonts are registered once with the engine's
//! [`FontRegistry`] (`engine.fonts().load("Body", path)` or
//! [`load_bytes`](FontRegistry::load_bytes)); a backend resolves a `Font` to its file or bytes
//! when text is first drawn and loads each face once.
//!
//! The core embeds one font, Open Sans Regular (SIL Open Font License 1.1, see
//! `assets/OFL.txt`), under the family name [`BUILT_IN_FAMILY`].  [`Font::default`] uses the
//! registry's default family, so applications need no font files; call
//! [`FontRegistry::set_default`] to change it.
//!
//! ```
//! use pushrod::font::{Font, FontRegistry};
//!
//! let fonts = FontRegistry::new();
//! assert_eq!(Font::default().size(), 14);
//! assert_eq!(fonts.default_font().family(), "Open Sans");
//! ```

use crate::color::FontStyle;
use crate::error::Error;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

/// The family name of the font the core embeds.
pub const BUILT_IN_FAMILY: &str = "Open Sans";

/// The size in points of [`Font::default`].
pub const DEFAULT_SIZE: u16 = 14;

/// The embedded font file (Open Sans Regular, SIL Open Font License 1.1).
const BUILT_IN_BYTES: &[u8] = include_bytes!("../assets/OpenSans-Regular.ttf");

/// A font handle: a family name, a size in points, and a style.
///
/// An empty family means "the registry's default family".  Clones share the family name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font {
    family: Arc<str>,
    size: u16,
    style: FontStyle,
}

impl Font {
    /// Creates a `Font`.
    ///
    /// # Arguments
    ///
    /// * `family` - A family registered with [`FontRegistry::load`] or
    ///   [`FontRegistry::load_bytes`], or `""` for the registry's default family.
    /// * `size` - The size in points.
    /// * `style` - The style (normal, bold, italic, ...).
    ///
    /// # Returns
    ///
    /// The new `Font`.
    pub fn new(family: &str, size: u16, style: FontStyle) -> Self {
        Self {
            family: Arc::from(family),
            size,
            style,
        }
    }

    /// Returns the family name; empty means the registry's default family.
    ///
    /// # Returns
    ///
    /// The family name.
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Returns the size in points.
    ///
    /// # Returns
    ///
    /// The size.
    pub fn size(&self) -> u16 {
        self.size
    }

    /// Returns the style.
    ///
    /// # Returns
    ///
    /// The style flags.
    pub fn style(&self) -> FontStyle {
        self.style
    }

    /// Returns a copy with another size.
    ///
    /// # Arguments
    ///
    /// * `size` - The size in points.
    ///
    /// # Returns
    ///
    /// The new `Font`; the family and style are kept.
    #[must_use]
    pub fn with_size(&self, size: u16) -> Self {
        Self {
            size,
            ..self.clone()
        }
    }

    /// Returns a copy with another style.
    ///
    /// # Arguments
    ///
    /// * `style` - The style flags.
    ///
    /// # Returns
    ///
    /// The new `Font`; the family and size are kept.
    #[must_use]
    pub fn with_style(&self, style: FontStyle) -> Self {
        Self {
            style,
            ..self.clone()
        }
    }
}

impl Default for Font {
    /// The registry's default family at [`DEFAULT_SIZE`] points, in the normal style.
    ///
    /// # Returns
    ///
    /// The default `Font`.
    fn default() -> Self {
        Self::new("", DEFAULT_SIZE, FontStyle::NORMAL)
    }
}

/// Where the bytes of a registered font come from.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FontSource {
    /// A TrueType or OpenType file on disk.
    Path(String),

    /// Font data held in memory; it lives for the rest of the process.
    Bytes(&'static [u8]),
}

/// The registered families and the default family.
struct Families {
    sources: HashMap<String, FontSource>,
    default_family: String,
}

/// The fonts an application has registered, shared between the engine and its backend.
///
/// Cloning gives another handle to the same registry.  It always contains [`BUILT_IN_FAMILY`].
#[derive(Clone)]
pub struct FontRegistry {
    inner: Arc<Mutex<Families>>,
}

impl FontRegistry {
    /// Creates a registry holding only the built-in font, which is also the default.
    ///
    /// # Returns
    ///
    /// The new `FontRegistry`.
    pub fn new() -> Self {
        let mut sources = HashMap::new();
        sources.insert(
            BUILT_IN_FAMILY.to_string(),
            FontSource::Bytes(BUILT_IN_BYTES),
        );

        Self {
            inner: Arc::new(Mutex::new(Families {
                sources,
                default_family: BUILT_IN_FAMILY.to_string(),
            })),
        }
    }

    /// Locks the registry, recovering from a poisoned lock (the data is always consistent).
    ///
    /// # Returns
    ///
    /// The guard.
    fn lock(&self) -> MutexGuard<'_, Families> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Registers a font file under a family name, replacing any family of that name.
    ///
    /// The file is read when text is first drawn with it, not here.
    ///
    /// # Arguments
    ///
    /// * `family` - The name to register; it must not be empty.
    /// * `path` - Path to a TrueType or OpenType file.
    ///
    /// # Returns
    ///
    /// `Ok` once the family is registered.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Font`] if `family` is empty or `path` is not a file.
    pub fn load(&self, family: &str, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let shown = path.display().to_string();

        if family.is_empty() {
            return Err(Error::font(shown, "the family name is empty"));
        }

        if !path.is_file() {
            return Err(Error::font(shown, "no such font file"));
        }

        self.lock()
            .sources
            .insert(family.to_string(), FontSource::Path(shown));
        Ok(())
    }

    /// Registers font data held in memory under a family name, replacing any family of that name.
    ///
    /// The data is kept for the rest of the process, because backends load fonts from it for as
    /// long as the process runs; register each font once.
    ///
    /// # Arguments
    ///
    /// * `family` - The name to register; it must not be empty.
    /// * `bytes` - The contents of a TrueType or OpenType file.
    ///
    /// # Returns
    ///
    /// `Ok` once the family is registered.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Font`] if `family` or `bytes` is empty.
    pub fn load_bytes(&self, family: &str, bytes: Vec<u8>) -> Result<(), Error> {
        if family.is_empty() || bytes.is_empty() {
            return Err(Error::font(family, "the family name or the data is empty"));
        }

        self.lock().sources.insert(
            family.to_string(),
            FontSource::Bytes(Box::leak(bytes.into_boxed_slice())),
        );
        Ok(())
    }

    /// Makes a registered family the default, used by every [`Font`] with an empty family.
    ///
    /// # Arguments
    ///
    /// * `family` - A registered family.
    ///
    /// # Returns
    ///
    /// `Ok` once the default is changed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Font`] if the family is not registered.
    pub fn set_default(&self, family: &str) -> Result<(), Error> {
        let mut families = self.lock();

        if !families.sources.contains_key(family) {
            return Err(Error::font(family, "the family is not registered"));
        }

        families.default_family = family.to_string();
        Ok(())
    }

    /// Returns the default family's name.
    ///
    /// # Returns
    ///
    /// The family name.
    pub fn default_family(&self) -> String {
        self.lock().default_family.clone()
    }

    /// Returns the default font: the default family at [`DEFAULT_SIZE`] points.
    ///
    /// # Returns
    ///
    /// The `Font`, naming the family explicitly.
    pub fn default_font(&self) -> Font {
        Font::new(&self.default_family(), DEFAULT_SIZE, FontStyle::NORMAL)
    }

    /// Returns the registered family names, sorted.
    ///
    /// # Returns
    ///
    /// The names, including [`BUILT_IN_FAMILY`].
    pub fn families(&self) -> Vec<String> {
        let mut names: Vec<String> = self.lock().sources.keys().cloned().collect();
        names.sort();
        names
    }

    /// Returns the family a font resolves to: its own, or the default if it names none.
    ///
    /// # Arguments
    ///
    /// * `font` - The font.
    ///
    /// # Returns
    ///
    /// The family name.
    pub fn resolve_family(&self, font: &Font) -> String {
        if font.family().is_empty() {
            self.default_family()
        } else {
            font.family().to_string()
        }
    }

    /// Returns where a font's data comes from.
    ///
    /// # Arguments
    ///
    /// * `font` - The font.
    ///
    /// # Returns
    ///
    /// The source, or `None` if the font's family is not registered.
    pub fn source(&self, font: &Font) -> Option<FontSource> {
        let family = self.resolve_family(font);
        self.lock().sources.get(&family).cloned()
    }
}

impl Default for FontRegistry {
    /// Creates a registry; the same as [`FontRegistry::new`].
    ///
    /// # Returns
    ///
    /// The new `FontRegistry`.
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FontRegistry {
    /// Lists the registered families and the default.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    /// Formats the value for debugging.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontRegistry")
            .field("families", &self.families())
            .field("default", &self.default_family())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_font_needs_no_files() {
        let fonts = FontRegistry::new();
        let font = Font::default();

        assert_eq!(font.family(), "");
        assert_eq!(font.size(), DEFAULT_SIZE);
        assert_eq!(fonts.resolve_family(&font), BUILT_IN_FAMILY);
        assert!(matches!(fonts.source(&font), Some(FontSource::Bytes(b)) if b.len() > 100_000));
    }

    #[test]
    fn embedded_font_is_a_truetype_file() {
        assert_eq!(&BUILT_IN_BYTES[..4], &[0, 1, 0, 0]);
    }

    #[test]
    fn license_file_is_shipped() {
        let license = include_str!("../assets/OFL.txt");

        assert!(license.contains("SIL OPEN FONT LICENSE Version 1.1"));
        assert!(license.contains("Open Sans Project Authors"));
    }

    #[test]
    fn handles_are_cheap_clones_with_setters() {
        let font = Font::new("Body", 12, FontStyle::NORMAL);
        let bold = font.with_style(FontStyle::BOLD).with_size(16);

        assert_eq!(font.family(), "Body");
        assert_eq!((bold.size(), bold.style()), (16, FontStyle::BOLD));
        assert_eq!(bold.family(), "Body");
        assert!(Arc::ptr_eq(&font.family, &bold.family));
    }

    #[test]
    fn load_registers_a_family_and_reports_a_missing_file() {
        let fonts = FontRegistry::new();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/OpenSans-Regular.ttf");

        fonts.load("Body", path).unwrap();
        assert_eq!(fonts.families(), vec!["Body", BUILT_IN_FAMILY]);
        assert_eq!(
            fonts.source(&Font::new("Body", 10, FontStyle::NORMAL)),
            Some(FontSource::Path(path.to_string()))
        );

        let error = fonts.load("Gone", "no/such/font.ttf").unwrap_err();
        assert!(matches!(error, Error::Font { .. }));
        assert!(fonts.load("", path).is_err());
    }

    #[test]
    fn load_bytes_registers_a_family_and_rejects_empty_data() {
        let fonts = FontRegistry::new();

        fonts.load_bytes("Mem", vec![0, 1, 0, 0]).unwrap();
        assert!(matches!(
            fonts.source(&Font::new("Mem", 10, FontStyle::NORMAL)),
            Some(FontSource::Bytes(b)) if b == [0, 1, 0, 0]
        ));
        assert!(fonts.load_bytes("Empty", Vec::new()).is_err());
        assert!(fonts.load_bytes("", vec![1]).is_err());
    }

    #[test]
    fn the_default_can_change_and_clones_share_it() {
        let fonts = FontRegistry::new();
        let shared = fonts.clone();

        fonts.load_bytes("Body", vec![0, 1, 0, 0]).unwrap();
        assert!(fonts.set_default("Missing").is_err());
        fonts.set_default("Body").unwrap();

        assert_eq!(shared.default_family(), "Body");
        assert_eq!(shared.resolve_family(&Font::default()), "Body");
        assert_eq!(shared.default_font().family(), "Body");
    }

    #[test]
    fn an_unregistered_family_has_no_source() {
        let fonts = FontRegistry::new();

        assert_eq!(
            fonts.source(&Font::new("Nope", 10, FontStyle::NORMAL)),
            None
        );
    }
}
