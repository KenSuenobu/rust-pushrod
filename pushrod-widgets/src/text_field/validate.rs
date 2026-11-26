// Text Validation
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

//! # Text validation
//!
//! The rules a [`TextField`](super::TextField) applies to what is typed:
//!
//! * a [`Validator`] judges the whole text as [`Validity::Valid`], [`Validity::Intermediate`]
//!   (not acceptable yet, but could become so with more typing), or [`Validity::Invalid`]
//!   (rejected: the edit that produced it does not happen);
//! * a [`Mask`] is a template such as `"99/99/9999"` (`9` a digit, `A` a letter, `X` any
//!   character, anything else a literal that is filled in for you);
//! * [`IntegerValidator`] and [`NumberValidator`] accept numbers within a range.

use std::fmt::Debug;

/// How acceptable a piece of text is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validity {
    /// The text is acceptable as it stands.
    Valid,
    /// The text is not acceptable yet, but more typing could make it so.  A field allows it
    /// while the user is typing.
    Intermediate,
    /// The text can never become acceptable; a field refuses the edit that made it.
    Invalid,
}

/// Judges text typed into a field.
pub trait Validator: Debug {
    /// Judges the whole text of the field.
    ///
    /// # Arguments
    ///
    /// * `text` - The text as it would be after the edit.
    ///
    /// # Returns
    ///
    /// How acceptable the text is.
    fn validate(&self, text: &str) -> Validity;
}

/// A template for the text of a field.  See the [module documentation](self).
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::text_field::validate::{Mask, Validator, Validity};
///
/// let date = Mask::new("99/99/9999");
///
/// assert_eq!(date.validate("12/25/2026"), Validity::Valid);
/// assert_eq!(date.validate("12/2"), Validity::Intermediate);
/// assert_eq!(date.validate("1a"), Validity::Invalid);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    slots: Vec<char>,
}

/// What the character typed next must be, from the mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Digit,
    Letter,
    Any,
    Literal(char),
}

impl Mask {
    /// Creates a mask from a template: `9` is a digit, `A` a letter, `X` any character; every
    /// other character is a literal.
    ///
    /// # Arguments
    ///
    /// * `template` - The template, such as `"99/99/9999"`.
    ///
    /// # Returns
    ///
    /// The new `Mask`.
    pub fn new(template: &str) -> Self {
        Self {
            slots: template.chars().collect(),
        }
    }

    /// Returns the number of characters in a complete text.
    ///
    /// # Returns
    ///
    /// The length of the template.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Returns whether the template is empty.
    ///
    /// # Returns
    ///
    /// `true` for an empty template, which accepts only empty text.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Returns what position `index` of the text must hold.
    ///
    /// # Arguments
    ///
    /// * `index` - The position, from zero.
    ///
    /// # Returns
    ///
    /// The slot, or `None` past the end of the template.
    fn slot(&self, index: usize) -> Option<Slot> {
        self.slots.get(index).map(|c| match c {
            '9' => Slot::Digit,
            'A' => Slot::Letter,
            'X' => Slot::Any,
            other => Slot::Literal(*other),
        })
    }

    /// Returns whether `c` may go in position `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The position.
    /// * `c` - The character.
    ///
    /// # Returns
    ///
    /// `true` if it fits the slot.
    pub fn accepts(&self, index: usize, c: char) -> bool {
        match self.slot(index) {
            Some(Slot::Digit) => c.is_ascii_digit(),
            Some(Slot::Letter) => c.is_alphabetic(),
            Some(Slot::Any) => !c.is_control(),
            Some(Slot::Literal(l)) => l == c,
            None => false,
        }
    }

    /// Returns the literal that must stand at `index`, if the template has one there.
    ///
    /// # Arguments
    ///
    /// * `index` - The position.
    ///
    /// # Returns
    ///
    /// The literal character, or `None` for a typed position or past the end.
    pub fn literal_at(&self, index: usize) -> Option<char> {
        match self.slot(index) {
            Some(Slot::Literal(c)) => Some(c),
            _ => None,
        }
    }

    /// Types `c` at the end of `text`, filling in the literals that come first.
    ///
    /// Typing the literal itself is accepted too, so `1`, `2`, `/` and `1` type `12/1`.
    ///
    /// # Arguments
    ///
    /// * `text` - The characters so far (a valid prefix of the mask).
    /// * `c` - The character typed.
    ///
    /// # Returns
    ///
    /// The new characters, or `None` if `c` does not fit.
    pub fn append(&self, text: &[char], c: char) -> Option<Vec<char>> {
        let mut out = text.to_vec();

        loop {
            let at = out.len();

            match self.slot(at)? {
                Slot::Literal(l) if l == c => {
                    out.push(l);
                    return Some(out);
                }
                Slot::Literal(l) => out.push(l),
                _ => {
                    return self.accepts(at, c).then(|| {
                        out.push(c);
                        out
                    });
                }
            }
        }
    }

    /// Cuts a text back to where the user can carry on typing: trailing literals are dropped, so
    /// deleting the `5` of `12/5` leaves `12`.
    ///
    /// # Arguments
    ///
    /// * `text` - The characters to trim.
    ///
    /// # Returns
    ///
    /// The trimmed characters.
    pub fn trim(&self, text: &[char]) -> Vec<char> {
        let mut out = text.to_vec();

        while !out.is_empty() && self.literal_at(out.len() - 1).is_some() {
            out.pop();
        }

        out
    }
}

impl Validator for Mask {
    /// Judges text against the template.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// `Valid` if it fills the template, `Intermediate` if it is a start of one, `Invalid` if some
    /// character does not fit.
    fn validate(&self, text: &str) -> Validity {
        let chars: Vec<char> = text.chars().collect();

        if chars.len() > self.slots.len()
            || !chars.iter().enumerate().all(|(i, c)| self.accepts(i, *c))
        {
            Validity::Invalid
        } else if chars.len() == self.slots.len() {
            Validity::Valid
        } else {
            Validity::Intermediate
        }
    }
}

/// Judges a number against a range: inside is valid, a value that more digits could still bring
/// inside is intermediate, anything else is invalid.
fn judge_range(value: f64, min: f64, max: f64) -> Validity {
    if value >= min && value <= max {
        Validity::Valid
    } else if (value > max && value > 0.0) || (value < min && value < 0.0) {
        Validity::Invalid
    } else {
        Validity::Intermediate
    }
}

/// Accepts whole numbers from `min` to `max`.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::text_field::validate::{IntegerValidator, Validator, Validity};
///
/// let age = IntegerValidator::new(0, 120);
///
/// assert_eq!(age.validate("42"), Validity::Valid);
/// assert_eq!(age.validate("421"), Validity::Invalid);
/// assert_eq!(age.validate("x"), Validity::Invalid);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegerValidator {
    min: i64,
    max: i64,
}

impl IntegerValidator {
    /// Creates a validator.  If `min` is greater than `max` the two are swapped.
    ///
    /// # Arguments
    ///
    /// * `min` - The smallest value.
    /// * `max` - The largest value.
    ///
    /// # Returns
    ///
    /// The new `IntegerValidator`.
    pub fn new(min: i64, max: i64) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }
}

impl Validator for IntegerValidator {
    /// Judges an integer.
    ///
    /// # Arguments
    ///
    /// * `text` - The text: an optional `-` (when the range has negatives) and digits.
    ///
    /// # Returns
    ///
    /// The validity; empty text and a lone `-` are `Intermediate`.
    fn validate(&self, text: &str) -> Validity {
        let digits = text.strip_prefix('-').unwrap_or(text);
        let negative = digits.len() != text.len();

        if (negative && self.min >= 0) || !digits.chars().all(|c| c.is_ascii_digit()) {
            return Validity::Invalid;
        }

        if digits.is_empty() {
            return Validity::Intermediate;
        }

        match text.parse::<i64>() {
            Ok(value) => judge_range(value as f64, self.min as f64, self.max as f64),
            Err(_) => Validity::Invalid,
        }
    }
}

/// Accepts decimal numbers from `min` to `max`, with at most `decimals` digits after the point.
///
/// # Examples
///
/// ```rust
/// use pushrod_widgets::text_field::validate::{NumberValidator, Validator, Validity};
///
/// let price = NumberValidator::new(0.0, 999.99, 2);
///
/// assert_eq!(price.validate("19.99"), Validity::Valid);
/// assert_eq!(price.validate("19.999"), Validity::Invalid);
/// assert_eq!(price.validate("19."), Validity::Valid);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberValidator {
    min: f64,
    max: f64,
    decimals: usize,
}

impl NumberValidator {
    /// Creates a validator.  If `min` is greater than `max` the two are swapped; a bound that is
    /// not a finite number counts as `0`.
    ///
    /// # Arguments
    ///
    /// * `min` - The smallest value.
    /// * `max` - The largest value.
    /// * `decimals` - The most digits allowed after the decimal point (`0` allows none).
    ///
    /// # Returns
    ///
    /// The new `NumberValidator`.
    pub fn new(min: f64, max: f64, decimals: usize) -> Self {
        let min = if min.is_finite() { min } else { 0.0 };
        let max = if max.is_finite() { max } else { 0.0 };

        Self {
            min: min.min(max),
            max: min.max(max),
            decimals,
        }
    }
}

impl Validator for NumberValidator {
    /// Judges a decimal number.
    ///
    /// # Arguments
    ///
    /// * `text` - The text: an optional `-` (when the range has negatives), digits, and at most
    ///   one `.`.
    ///
    /// # Returns
    ///
    /// The validity; empty text, `-`, `.` and `-.` are `Intermediate`.
    fn validate(&self, text: &str) -> Validity {
        let body = text.strip_prefix('-').unwrap_or(text);
        let negative = body.len() != text.len();
        let (whole, fraction) = match body.split_once('.') {
            Some((w, f)) => (w, Some(f)),
            None => (body, None),
        };

        let digits_only = |s: &str| s.chars().all(|c| c.is_ascii_digit());

        if (negative && self.min >= 0.0)
            || !digits_only(whole)
            || !fraction.is_none_or(|f| digits_only(f) && f.len() <= self.decimals)
            || (fraction.is_some() && self.decimals == 0)
        {
            return Validity::Invalid;
        }

        if whole.is_empty() && fraction.is_none_or(str::is_empty) {
            return Validity::Intermediate;
        }

        let sign = if negative { -1.0 } else { 1.0 };
        let digits = format!(
            "{}.{}",
            if whole.is_empty() { "0" } else { whole },
            fraction.filter(|f| !f.is_empty()).unwrap_or("0")
        );

        match digits.parse::<f64>() {
            Ok(value) => judge_range(sign * value, self.min, self.max),
            Err(_) => Validity::Invalid,
        }
    }
}
