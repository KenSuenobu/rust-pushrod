// Color
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

//! # Color
//!
//! Pushrod's own color and font style types.  `Widget`s and the [`Painter`](crate::paint::Painter)
//! use these instead of `SDL2` types, so a widget library never compiles against a specific
//! `SDL2` version.  A backend converts them at its boundary.

/// A color with red, green, blue, and alpha channels, each `0..=255`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel (`255` is opaque).
    pub a: u8,
}

#[allow(non_snake_case)]
impl Color {
    /// Opaque black.
    pub const BLACK: Color = Color::RGB(0, 0, 0);
    /// Opaque white.
    pub const WHITE: Color = Color::RGB(255, 255, 255);
    /// Opaque gray.
    pub const GRAY: Color = Color::RGB(128, 128, 128);
    /// Opaque red.
    pub const RED: Color = Color::RGB(255, 0, 0);
    /// Opaque green.
    pub const GREEN: Color = Color::RGB(0, 255, 0);
    /// Opaque blue.
    pub const BLUE: Color = Color::RGB(0, 0, 255);
    /// Opaque yellow.
    pub const YELLOW: Color = Color::RGB(255, 255, 0);
    /// Opaque cyan.
    pub const CYAN: Color = Color::RGB(0, 255, 255);
    /// Opaque magenta.
    pub const MAGENTA: Color = Color::RGB(255, 0, 255);

    /// Creates an opaque color.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel.
    /// * `g` - Green channel.
    /// * `b` - Blue channel.
    ///
    /// # Returns
    ///
    /// The new `Color`, with alpha `255`.
    pub const fn RGB(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates a color with an alpha channel.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel.
    /// * `g` - Green channel.
    /// * `b` - Blue channel.
    /// * `a` - Alpha channel (`255` is opaque).
    ///
    /// # Returns
    ///
    /// The new `Color`.
    pub const fn RGBA(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Font style flags, which can be combined with `|`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FontStyle(u8);

impl FontStyle {
    /// Plain text.
    pub const NORMAL: FontStyle = FontStyle(0);
    /// Bold text.
    pub const BOLD: FontStyle = FontStyle(1);
    /// Italic text.
    pub const ITALIC: FontStyle = FontStyle(2);
    /// Underlined text.
    pub const UNDERLINE: FontStyle = FontStyle(4);
    /// Struck-through text.
    pub const STRIKETHROUGH: FontStyle = FontStyle(8);

    /// Returns the raw flag bits (bold `1`, italic `2`, underline `4`, strikethrough `8`).
    ///
    /// # Returns
    ///
    /// The bits.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Returns whether every flag in `other` is set.
    ///
    /// # Arguments
    ///
    /// * `other` - The flags to look for.
    ///
    /// # Returns
    ///
    /// `true` if all of `other`'s flags are set in this style.
    pub const fn contains(self, other: FontStyle) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for FontStyle {
    type Output = FontStyle;

    /// Combines two styles.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The style to add.
    ///
    /// # Returns
    ///
    /// A style with the flags of both.
    fn bitor(self, rhs: FontStyle) -> FontStyle {
        FontStyle(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_is_opaque() {
        assert_eq!(Color::RGB(1, 2, 3), Color::RGBA(1, 2, 3, 255));
    }

    #[test]
    fn named_colors() {
        assert_eq!(Color::BLACK, Color::RGBA(0, 0, 0, 255));
        assert_eq!(Color::WHITE.r, 255);
    }

    #[test]
    fn font_styles_combine() {
        let style = FontStyle::BOLD | FontStyle::ITALIC;
        assert!(style.contains(FontStyle::BOLD));
        assert!(style.contains(FontStyle::ITALIC));
        assert!(!style.contains(FontStyle::UNDERLINE));
        assert_eq!(style.bits(), 3);
        assert!(FontStyle::NORMAL.contains(FontStyle::NORMAL));
    }
}
