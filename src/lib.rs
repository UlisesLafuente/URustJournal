//! UrustJournal: a cross-platform bullet journal written in Rust.

pub mod app;
pub mod error;
pub mod i18n;
pub mod journal;
pub mod theme;

pub use error::{JournalError, Result};
pub use i18n::Lang;
