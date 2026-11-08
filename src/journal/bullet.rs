//! Bullet journal entries and their plain-text representation.
//!
//! The on-disk format is line based:
//!
//! | Kind      | Syntax        |
//! |-----------|---------------|
//! | Task      | `- [ ] text`  |
//! | Done task | `- [x] text`  |
//! | Migrated  | `- [>] text`  |
//! | Scheduled | `- [<] text`  |
//! | Event     | `o text`      |
//! | Note      | `- text`      |
//!
//! Any line that does not match a known marker is preserved verbatim as
//! [`Bullet::Raw`], which keeps headings and free-form text intact.

/// A single bullet journal entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bullet {
    /// A task that can be completed.
    Task { text: String, completed: bool },
    /// A task migrated to a future date or collection.
    Migrated(String),
    /// A task scheduled for a future date.
    Scheduled(String),
    /// An event that happened.
    Event(String),
    /// A plain note.
    Note(String),
    /// Any line that could not be parsed, kept exactly as written.
    Raw(String),
}

impl Bullet {
    /// Creates a pending task.
    pub fn task(text: impl Into<String>) -> Self {
        Self::Task {
            text: text.into(),
            completed: false,
        }
    }

    /// Creates a plain note.
    pub fn note(text: impl Into<String>) -> Self {
        Self::Note(text.into())
    }

    /// Creates an event.
    pub fn event(text: impl Into<String>) -> Self {
        Self::Event(text.into())
    }

    /// Creates a migrated task.
    pub fn migrated(text: impl Into<String>) -> Self {
        Self::Migrated(text.into())
    }

    /// Creates a scheduled task.
    pub fn scheduled(text: impl Into<String>) -> Self {
        Self::Scheduled(text.into())
    }

    /// Returns the text carried by the bullet.
    pub fn text(&self) -> &str {
        match self {
            Self::Task { text, .. }
            | Self::Migrated(text)
            | Self::Scheduled(text)
            | Self::Event(text)
            | Self::Note(text)
            | Self::Raw(text) => text,
        }
    }

    /// Flips the completion state of a task, doing nothing for other kinds.
    pub fn toggle(&mut self) {
        if let Self::Task { completed, .. } = self {
            *completed = !*completed;
        }
    }

    /// Parses a single line of a journal file.
    pub fn parse(line: &str) -> Self {
        if let Some(text) = strip_marker(line, "- [ ]") {
            return Self::Task {
                text: text.to_owned(),
                completed: false,
            };
        }
        if let Some(text) = strip_marker(line, "- [x]").or_else(|| strip_marker(line, "- [X]")) {
            return Self::Task {
                text: text.to_owned(),
                completed: true,
            };
        }
        if let Some(text) = strip_marker(line, "- [>]") {
            return Self::Migrated(text.to_owned());
        }
        if let Some(text) = strip_marker(line, "- [<]") {
            return Self::Scheduled(text.to_owned());
        }
        if let Some(text) = strip_marker(line, "o") {
            return Self::Event(text.to_owned());
        }
        if let Some(text) = strip_marker(line, "-") {
            return Self::Note(text.to_owned());
        }
        Self::Raw(line.to_owned())
    }

    /// Renders the bullet back to its plain-text line.
    pub fn render(&self) -> String {
        match self {
            Self::Task {
                text,
                completed: true,
            } => format!("- [x] {text}"),
            Self::Task {
                text,
                completed: false,
            } => format!("- [ ] {text}"),
            Self::Migrated(text) => format!("- [>] {text}"),
            Self::Scheduled(text) => format!("- [<] {text}"),
            Self::Event(text) => format!("o {text}"),
            Self::Note(text) => format!("- {text}"),
            Self::Raw(text) => text.clone(),
        }
    }
}

fn strip_marker<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(marker)?;
    if rest.is_empty() {
        Some(rest)
    } else {
        rest.strip_prefix(' ')
    }
}
