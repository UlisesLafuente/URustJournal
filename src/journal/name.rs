//! Validated journal names.

use std::fmt;

use crate::error::{JournalError, Result};

/// Maximum accepted length of a journal name, in characters.
pub const MAX_NAME_LEN: usize = 64;

const RESERVED_WINDOWS_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// A journal name that has already been validated.
///
/// Valid names are non-empty, at most [`MAX_NAME_LEN`] characters long and
/// contain only ASCII alphanumerics or underscores.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JournalName(String);

impl JournalName {
    /// Validates `raw` and builds a journal name.
    pub fn new(raw: &str) -> Result<Self> {
        Self::validate(raw)?;
        Ok(Self(raw.to_owned()))
    }

    /// Checks whether `raw` is a valid journal name without allocating.
    pub fn validate(raw: &str) -> Result<()> {
        if raw.is_empty() {
            return Err(JournalError::InvalidName(
                "el nombre no puede estar vacío".to_owned(),
            ));
        }

        if raw.chars().count() > MAX_NAME_LEN {
            return Err(JournalError::InvalidName(format!(
                "el nombre supera los {MAX_NAME_LEN} caracteres"
            )));
        }

        if !raw
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(JournalError::InvalidName(
                "solo se permiten letras, números y guiones bajos".to_owned(),
            ));
        }

        if RESERVED_WINDOWS_NAMES.contains(&raw.to_ascii_uppercase().as_str()) {
            return Err(JournalError::InvalidName(format!(
                "'{raw}' es un nombre reservado por el sistema"
            )));
        }

        Ok(())
    }

    /// Borrows the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JournalName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for JournalName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
