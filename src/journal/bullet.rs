//! Bullet journal entries and their plain-text representation.
//!
//! Bullets follow the rapid-logging notation:
//!
//! | Kind      | Marker | Syntax      |
//! |-----------|--------|-------------|
//! | Task      | `•`    | `• text`    |
//! | Completed | `x`    | `x text`    |
//! | Migrated  | `>`    | `> text`    |
//! | Scheduled | `<`    | `< text`    |
//! | Event     | `o`    | `o text`    |
//! | Note      | `-`    | `- text`    |
//!
//! The older Markdown-style checkbox syntax (`- [ ]`, `- [x]`, `- [>]`,
//! `- [<]`) is still accepted when reading, so existing journals keep working.
//!
//! Any line that does not match a known marker is preserved verbatim as
//! [`Bullet::Raw`], which keeps headings and free-form text intact.

/// The rapid-logging state of a bullet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BulletMarker {
    /// An open task.
    Task,
    /// A finished task.
    Completed,
    /// A task moved to a future collection.
    Migrated,
    /// A task scheduled for a future date.
    Scheduled,
    /// An event that happened.
    Event,
    /// A plain note.
    Note,
}

impl BulletMarker {
    /// All markers in display order.
    pub const ALL: [Self; 6] = [
        Self::Task,
        Self::Completed,
        Self::Migrated,
        Self::Scheduled,
        Self::Event,
        Self::Note,
    ];

    /// The symbol used in the journal and on disk.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Task => "•",
            Self::Completed => "x",
            Self::Migrated => ">",
            Self::Scheduled => "<",
            Self::Event => "o",
            Self::Note => "-",
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Task => "Tarea",
            Self::Completed => "Completada",
            Self::Migrated => "Migrada (>)",
            Self::Scheduled => "Programada (<)",
            Self::Event => "Evento",
            Self::Note => "Nota",
        }
    }
}

/// A single bullet journal entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bullet {
    /// A task, either open or completed.
    Task { text: String, completed: bool },
    /// A task moved to a future collection.
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
        Self::with_marker(BulletMarker::Task, text)
    }

    /// Creates a plain note.
    pub fn note(text: impl Into<String>) -> Self {
        Self::with_marker(BulletMarker::Note, text)
    }

    /// Creates an event.
    pub fn event(text: impl Into<String>) -> Self {
        Self::with_marker(BulletMarker::Event, text)
    }

    /// Creates a migrated task.
    pub fn migrated(text: impl Into<String>) -> Self {
        Self::with_marker(BulletMarker::Migrated, text)
    }

    /// Creates a scheduled task.
    pub fn scheduled(text: impl Into<String>) -> Self {
        Self::with_marker(BulletMarker::Scheduled, text)
    }

    /// Creates a bullet from a marker and text.
    pub fn with_marker(marker: BulletMarker, text: impl Into<String>) -> Self {
        let text = text.into();
        match marker {
            BulletMarker::Task => Self::Task {
                text,
                completed: false,
            },
            BulletMarker::Completed => Self::Task {
                text,
                completed: true,
            },
            BulletMarker::Migrated => Self::Migrated(text),
            BulletMarker::Scheduled => Self::Scheduled(text),
            BulletMarker::Event => Self::Event(text),
            BulletMarker::Note => Self::Note(text),
        }
    }

    /// Returns the marker for this bullet, or `None` for raw lines.
    pub fn marker(&self) -> Option<BulletMarker> {
        match self {
            Self::Task {
                completed: false, ..
            } => Some(BulletMarker::Task),
            Self::Task {
                completed: true, ..
            } => Some(BulletMarker::Completed),
            Self::Migrated(_) => Some(BulletMarker::Migrated),
            Self::Scheduled(_) => Some(BulletMarker::Scheduled),
            Self::Event(_) => Some(BulletMarker::Event),
            Self::Note(_) => Some(BulletMarker::Note),
            Self::Raw(_) => None,
        }
    }

    /// Changes the marker while preserving the text. Raw lines are unchanged.
    pub fn set_marker(&mut self, marker: BulletMarker) {
        if matches!(self, Self::Raw(_)) {
            return;
        }
        let text = self.text().to_owned();
        *self = Self::with_marker(marker, text);
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
            return Self::with_marker(BulletMarker::Task, text);
        }
        if let Some(text) = strip_marker(line, "- [x]").or_else(|| strip_marker(line, "- [X]")) {
            return Self::with_marker(BulletMarker::Completed, text);
        }
        if let Some(text) = strip_marker(line, "- [>]") {
            return Self::with_marker(BulletMarker::Migrated, text);
        }
        if let Some(text) = strip_marker(line, "- [<]") {
            return Self::with_marker(BulletMarker::Scheduled, text);
        }
        if let Some(text) = strip_marker(line, "•").or_else(|| strip_marker(line, "·")) {
            return Self::with_marker(BulletMarker::Task, text);
        }
        if let Some(text) = strip_marker(line, "x")
            .or_else(|| strip_marker(line, "X"))
            .or_else(|| strip_marker(line, "×"))
        {
            return Self::with_marker(BulletMarker::Completed, text);
        }
        if let Some(text) = strip_marker(line, ">") {
            return Self::with_marker(BulletMarker::Migrated, text);
        }
        if let Some(text) = strip_marker(line, "<") {
            return Self::with_marker(BulletMarker::Scheduled, text);
        }
        if let Some(text) = strip_marker(line, "o") {
            return Self::with_marker(BulletMarker::Event, text);
        }
        if let Some(text) = strip_marker(line, "-") {
            return Self::with_marker(BulletMarker::Note, text);
        }
        Self::Raw(line.to_owned())
    }

    /// Renders the bullet back to its plain-text line.
    pub fn render(&self) -> String {
        match self {
            Self::Task {
                text,
                completed: true,
            } => format!("x {text}"),
            Self::Task {
                text,
                completed: false,
            } => format!("• {text}"),
            Self::Migrated(text) => format!("> {text}"),
            Self::Scheduled(text) => format!("< {text}"),
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
