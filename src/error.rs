//! Domain error type shared by the whole application.

use std::fmt;
use std::io;

/// Errors that can occur while managing journals.
#[derive(Debug)]
pub enum JournalError {
    /// The requested journal name does not satisfy the validation rules.
    InvalidName(String),
    /// A journal with the same name already exists.
    AlreadyExists(String),
    /// The requested journal does not exist.
    NotFound(String),
    /// An underlying filesystem operation failed.
    Io(io::Error),
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(reason) => write!(f, "Nombre de journal inválido: {reason}"),
            Self::AlreadyExists(name) => write!(f, "Ya existe un journal llamado '{name}'"),
            Self::NotFound(name) => write!(f, "No se encontró el journal '{name}'"),
            Self::Io(error) => write!(f, "Error de entrada/salida: {error}"),
        }
    }
}

impl std::error::Error for JournalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for JournalError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Convenience result type used across the crate.
pub type Result<T> = std::result::Result<T, JournalError>;
