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

//! The contact book: the application's data, with no user interface in it.
//!
//! Everything here is plain Rust, so it is tested without a window.  All sample data is
//! fictional: names are placeholders, e-mail addresses use `example.com`, and telephone numbers
//! use the fictional `555-01xx` range.

use pushrod_widgets::text_field::validate::{Mask, Validator, Validity};

/// How many contacts the book holds at most.  The list's rows are made once, up front, so it
/// can show this many.
pub const MAX_CONTACTS: usize = 16;

/// The telephone field's mask: `9` is a digit, everything else is filled in for you.
pub const PHONE_MASK: &str = "(999) 999-9999";

/// How a contact prefers to be reached; one radio button each.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// By e-mail.
    #[default]
    Email,
    /// By telephone.
    Phone,
    /// By post.
    Mail,
}

impl Reach {
    /// Every choice, in the order the radio buttons show them.
    pub const ALL: [Reach; 3] = [Reach::Email, Reach::Phone, Reach::Mail];

    /// Returns the radio button's label.
    ///
    /// # Returns
    ///
    /// `"Email"`, `"Phone"`, or `"Mail"`.
    pub fn label(self) -> &'static str {
        match self {
            Reach::Email => "Email",
            Reach::Phone => "Phone",
            Reach::Mail => "Mail",
        }
    }
}

/// One contact card.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Contact {
    /// The full name; required.
    pub name: String,
    /// The e-mail address; empty, or `name@example.com`.
    pub email: String,
    /// The telephone number, as the mask formats it (may be empty).
    pub phone: String,
    /// The city.
    pub city: String,
    /// How the contact prefers to be reached.
    pub reach: Reach,
    /// Shown with a star in the list.
    pub favorite: bool,
    /// Receives the newsletter.
    pub newsletter: bool,
    /// How close a contact is, from 0 (acquaintance) to 5 (family).
    pub closeness: u8,
}

impl Contact {
    /// Creates a contact with a name and an e-mail address; everything else is the default.
    ///
    /// # Arguments
    ///
    /// * `name` - The full name.
    /// * `email` - The e-mail address.
    ///
    /// # Returns
    ///
    /// The contact.
    pub fn new(name: &str, email: &str) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
            ..Self::default()
        }
    }

    /// Returns the label of the contact's row in the list.
    ///
    /// # Returns
    ///
    /// The name, with a star in front for a favorite.
    pub fn list_label(&self) -> String {
        if self.favorite {
            format!("* {}", self.name)
        } else {
            self.name.clone()
        }
    }

    /// Checks the contact before it is saved.
    ///
    /// # Returns
    ///
    /// `Ok` when it can be saved, or the message to show next to the form.
    ///
    /// # Errors
    ///
    /// Returns a message when the name is empty, or the e-mail address or telephone number is
    /// incomplete.
    pub fn check(&self) -> Result<(), &'static str> {
        if RequiredValidator.validate(&self.name) != Validity::Valid {
            return Err("A name is required.");
        }

        if EmailValidator.validate(&self.email) != Validity::Valid {
            return Err("The email address is incomplete.");
        }

        if !self.phone.is_empty() && Mask::new(PHONE_MASK).validate(&self.phone) != Validity::Valid
        {
            return Err("The phone number is incomplete.");
        }

        Ok(())
    }
}

/// The fictional contacts the application starts with.
///
/// # Returns
///
/// Fourteen contacts, sorted by name: more than the list shows at the starting window size, so
/// it scrolls.
pub fn sample_contacts() -> Vec<Contact> {
    let rows = [
        (
            "Alex Example",
            "alex@example.com",
            "(555) 010-0101",
            "Sampleville",
            Reach::Phone,
            false,
            2,
        ),
        (
            "Avery Example",
            "avery@example.net",
            "",
            "Othertown",
            Reach::Mail,
            false,
            1,
        ),
        (
            "Casey Placeholder",
            "casey@example.com",
            "",
            "Anytown",
            Reach::Email,
            false,
            1,
        ),
        (
            "Chris Sample",
            "chris.sample@example.org",
            "(555) 010-0102",
            "Othertown",
            Reach::Mail,
            false,
            0,
        ),
        (
            "Drew Mockup",
            "drew@example.com",
            "(555) 010-0107",
            "Anytown",
            Reach::Phone,
            false,
            2,
        ),
        (
            "Jane Doe",
            "jane.doe@example.com",
            "(555) 010-0100",
            "Anytown",
            Reach::Email,
            true,
            5,
        ),
        (
            "John Doe",
            "john.doe@example.com",
            "(555) 010-0103",
            "Anytown",
            Reach::Phone,
            true,
            5,
        ),
        (
            "Jordan Demo",
            "jordan@example.net",
            "",
            "Sampleville",
            Reach::Email,
            false,
            3,
        ),
        (
            "Morgan Specimen",
            "morgan@example.com",
            "(555) 010-0104",
            "Othertown",
            Reach::Phone,
            false,
            1,
        ),
        (
            "Pat Placeholder",
            "pat@example.com",
            "(555) 010-0105",
            "Anytown",
            Reach::Mail,
            false,
            2,
        ),
        (
            "Quinn Sample",
            "quinn@example.org",
            "",
            "Sampleville",
            Reach::Email,
            false,
            0,
        ),
        (
            "Riley Testcase",
            "riley@example.com",
            "(555) 010-0108",
            "Othertown",
            Reach::Phone,
            false,
            3,
        ),
        (
            "Sam Sample",
            "sam.sample@example.com",
            "",
            "Sampleville",
            Reach::Email,
            false,
            4,
        ),
        (
            "Taylor Anyone",
            "taylor@example.org",
            "(555) 010-0106",
            "Anytown",
            Reach::Email,
            false,
            3,
        ),
    ];

    rows.into_iter()
        .map(
            |(name, email, phone, city, reach, favorite, closeness)| Contact {
                name: name.to_string(),
                email: email.to_string(),
                phone: phone.to_string(),
                city: city.to_string(),
                reach,
                favorite,
                newsletter: favorite,
                closeness,
            },
        )
        .collect()
}

/// The application state: every contact and which one the form shows.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactBook {
    /// The contacts, in list order.
    pub contacts: Vec<Contact>,
    /// The index of the contact the form shows, if any.
    pub selected: Option<usize>,
}

impl Default for ContactBook {
    /// The sample book, with Jane Doe selected.
    fn default() -> Self {
        let contacts = sample_contacts();
        let selected = contacts.iter().position(|c| c.name == "Jane Doe");

        Self { contacts, selected }
    }
}

impl ContactBook {
    /// Returns the contact the form shows.
    ///
    /// # Returns
    ///
    /// The selected contact, or `None` when the book is empty.
    pub fn current(&self) -> Option<&Contact> {
        self.selected.and_then(|i| self.contacts.get(i))
    }

    /// Selects a contact.
    ///
    /// # Arguments
    ///
    /// * `index` - The contact's place in the list.
    ///
    /// # Returns
    ///
    /// `true` if `index` names a contact (it is now selected); `false` leaves the selection.
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.contacts.len() {
            self.selected = Some(index);
            true
        } else {
            false
        }
    }

    /// Returns whether another contact fits.
    ///
    /// # Returns
    ///
    /// `true` while the book holds fewer than [`MAX_CONTACTS`].
    pub fn has_room(&self) -> bool {
        self.contacts.len() < MAX_CONTACTS
    }

    /// Adds a blank contact named `New contact` at the end, and selects it.
    ///
    /// # Returns
    ///
    /// The new contact's index, or `None` when the book is full.
    pub fn add_new(&mut self) -> Option<usize> {
        if !self.has_room() {
            return None;
        }

        self.contacts.push(Contact::new("New contact", ""));

        let index = self.contacts.len() - 1;

        self.selected = Some(index);

        Some(index)
    }

    /// Replaces the selected contact, after checking it.
    ///
    /// # Arguments
    ///
    /// * `contact` - The edited contact.
    ///
    /// # Returns
    ///
    /// `Ok` once saved.
    ///
    /// # Errors
    ///
    /// Returns the check's message (see [`Contact::check`]), or `"No contact is selected."`.
    pub fn save(&mut self, contact: Contact) -> Result<(), &'static str> {
        contact.check()?;

        let slot = self
            .selected
            .and_then(|i| self.contacts.get_mut(i))
            .ok_or("No contact is selected.")?;

        *slot = contact;

        Ok(())
    }

    /// Deletes the selected contact and selects the one that took its place (or the one before
    /// it, when the last was deleted).
    ///
    /// # Returns
    ///
    /// The deleted contact, or `None` when nothing was selected.
    pub fn delete_selected(&mut self) -> Option<Contact> {
        let index = self.selected.filter(|&i| i < self.contacts.len())?;
        let removed = self.contacts.remove(index);

        self.selected = if self.contacts.is_empty() {
            None
        } else {
            Some(index.min(self.contacts.len() - 1))
        };

        Some(removed)
    }
}

/// Accepts any text but calls it `Intermediate` while it is blank, so a required field can be
/// emptied while typing but not saved empty.
#[derive(Debug, Default, Clone, Copy)]
pub struct RequiredValidator;

impl Validator for RequiredValidator {
    /// Judges a required field.
    ///
    /// # Arguments
    ///
    /// * `text` - The field's text.
    ///
    /// # Returns
    ///
    /// `Valid` when it has a character other than white space, `Intermediate` otherwise.
    fn validate(&self, text: &str) -> Validity {
        if text.trim().is_empty() {
            Validity::Intermediate
        } else {
            Validity::Valid
        }
    }
}

/// Judges an e-mail address as it is typed: `name@example.com`, or empty.
///
/// White space and a second `@` are refused outright; text that could still become an address
/// (`jane`, `jane@`, `jane@example`) is `Intermediate`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmailValidator;

impl Validator for EmailValidator {
    /// Judges an e-mail address.
    ///
    /// # Arguments
    ///
    /// * `text` - The field's text.
    ///
    /// # Returns
    ///
    /// `Valid` for an empty field or a complete address, `Invalid` for white space or more than
    /// one `@`, and `Intermediate` for anything else.
    fn validate(&self, text: &str) -> Validity {
        if text.is_empty() {
            return Validity::Valid;
        }

        if text.chars().any(char::is_whitespace) || text.matches('@').count() > 1 {
            return Validity::Invalid;
        }

        let Some((local, domain)) = text.split_once('@') else {
            return Validity::Intermediate;
        };
        let complete = !local.is_empty()
            && domain
                .rsplit_once('.')
                .is_some_and(|(host, tld)| !host.is_empty() && tld.len() >= 2);

        if complete {
            Validity::Valid
        } else {
            Validity::Intermediate
        }
    }
}
