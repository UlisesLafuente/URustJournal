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

/// Optional text color for an annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnnotationColor {
    #[default]
    Black,
    DesaturatedBlue,
    EmeraldGreen,
    RustRed,
}

impl AnnotationColor {
    pub const ALL: [Self; 4] = [
        Self::Black,
        Self::DesaturatedBlue,
        Self::EmeraldGreen,
        Self::RustRed,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Black => "Negro",
            Self::DesaturatedBlue => "Azul desaturado",
            Self::EmeraldGreen => "Verde esmeralda",
            Self::RustRed => "Rojo óxido",
        }
    }

    fn tag(self) -> Option<&'static str> {
        match self {
            Self::Black => None,
            Self::DesaturatedBlue => Some("blue"),
            Self::EmeraldGreen => Some("emerald"),
            Self::RustRed => Some("rust"),
        }
    }

    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "blue" => Some(Self::DesaturatedBlue),
            "emerald" => Some(Self::EmeraldGreen),
            "rust" => Some(Self::RustRed),
            _ => None,
        }
    }
}

/// A single bullet journal entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bullet {
    /// A task, either open or completed.
    Task {
        text: String,
        completed: bool,
        color: AnnotationColor,
    },
    /// A task moved to a future collection.
    Migrated {
        text: String,
        color: AnnotationColor,
    },
    /// A task scheduled for a future date.
    Scheduled {
        text: String,
        color: AnnotationColor,
    },
    /// An event that happened.
    Event {
        text: String,
        color: AnnotationColor,
    },
    /// A plain note.
    Note {
        text: String,
        color: AnnotationColor,
    },
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
        Self::with_marker_and_color(marker, text, AnnotationColor::default())
    }

    /// Creates a bullet from a marker, text and color.
    pub fn with_marker_and_color(
        marker: BulletMarker,
        text: impl Into<String>,
        color: AnnotationColor,
    ) -> Self {
        let text = text.into();
        match marker {
            BulletMarker::Task => Self::Task {
                text,
                completed: false,
                color,
            },
            BulletMarker::Completed => Self::Task {
                text,
                completed: true,
                color,
            },
            BulletMarker::Migrated => Self::Migrated { text, color },
            BulletMarker::Scheduled => Self::Scheduled { text, color },
            BulletMarker::Event => Self::Event { text, color },
            BulletMarker::Note => Self::Note { text, color },
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
            Self::Migrated { .. } => Some(BulletMarker::Migrated),
            Self::Scheduled { .. } => Some(BulletMarker::Scheduled),
            Self::Event { .. } => Some(BulletMarker::Event),
            Self::Note { .. } => Some(BulletMarker::Note),
            Self::Raw(_) => None,
        }
    }

    /// Changes the marker while preserving the text and color. Raw lines are unchanged.
    pub fn set_marker(&mut self, marker: BulletMarker) {
        if matches!(self, Self::Raw(_)) {
            return;
        }
        let text = self.text().to_owned();
        let color = self.color();
        *self = Self::with_marker_and_color(marker, text, color);
    }

    /// Returns the annotation color.
    pub fn color(&self) -> AnnotationColor {
        match self {
            Self::Task { color, .. }
            | Self::Migrated { color, .. }
            | Self::Scheduled { color, .. }
            | Self::Event { color, .. }
            | Self::Note { color, .. } => *color,
            Self::Raw(_) => AnnotationColor::Black,
        }
    }

    /// Sets the annotation color. Raw lines are unchanged.
    pub fn set_color(&mut self, new_color: AnnotationColor) {
        match self {
            Self::Task { color, .. }
            | Self::Migrated { color, .. }
            | Self::Scheduled { color, .. }
            | Self::Event { color, .. }
            | Self::Note { color, .. } => *color = new_color,
            Self::Raw(_) => {}
        }
    }

    /// Sets the text content. Raw lines are unchanged.
    pub fn set_text(&mut self, new_text: String) {
        match self {
            Self::Task { text, .. }
            | Self::Migrated { text, .. }
            | Self::Scheduled { text, .. }
            | Self::Event { text, .. }
            | Self::Note { text, .. } => *text = new_text,
            Self::Raw(_) => {}
        }
    }

    /// Returns the text carried by the bullet.
    pub fn text(&self) -> &str {
        match self {
            Self::Task { text, .. }
            | Self::Migrated { text, .. }
            | Self::Scheduled { text, .. }
            | Self::Event { text, .. }
            | Self::Note { text, .. }
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
            return parse_with_color(BulletMarker::Task, text);
        }
        if let Some(text) = strip_marker(line, "- [x]").or_else(|| strip_marker(line, "- [X]")) {
            return parse_with_color(BulletMarker::Completed, text);
        }
        if let Some(text) = strip_marker(line, "- [>]") {
            return parse_with_color(BulletMarker::Migrated, text);
        }
        if let Some(text) = strip_marker(line, "- [<]") {
            return parse_with_color(BulletMarker::Scheduled, text);
        }
        if let Some(text) = strip_marker(line, "•").or_else(|| strip_marker(line, "·")) {
            return parse_with_color(BulletMarker::Task, text);
        }
        if let Some(text) = strip_marker(line, "x")
            .or_else(|| strip_marker(line, "X"))
            .or_else(|| strip_marker(line, "×"))
        {
            return parse_with_color(BulletMarker::Completed, text);
        }
        if let Some(text) = strip_marker(line, ">") {
            return parse_with_color(BulletMarker::Migrated, text);
        }
        if let Some(text) = strip_marker(line, "<") {
            return parse_with_color(BulletMarker::Scheduled, text);
        }
        if let Some(text) = strip_marker(line, "o") {
            return parse_with_color(BulletMarker::Event, text);
        }
        if let Some(text) = strip_marker(line, "-") {
            return parse_with_color(BulletMarker::Note, text);
        }
        Self::Raw(line.to_owned())
    }

    /// Renders the bullet back to its plain-text line.
    pub fn render(&self) -> String {
        let (marker, text, color) = match self {
            Self::Task {
                text,
                completed: false,
                color,
            } => (BulletMarker::Task, text, *color),
            Self::Task {
                text,
                completed: true,
                color,
            } => (BulletMarker::Completed, text, *color),
            Self::Migrated { text, color } => (BulletMarker::Migrated, text, *color),
            Self::Scheduled { text, color } => (BulletMarker::Scheduled, text, *color),
            Self::Event { text, color } => (BulletMarker::Event, text, *color),
            Self::Note { text, color } => (BulletMarker::Note, text, *color),
            Self::Raw(text) => return text.clone(),
        };
        let prefix = color
            .tag()
            .map_or(String::new(), |tag| format!("{{c:{tag}}} "));
        format!("{} {prefix}{text}", marker.symbol())
    }
}

fn parse_with_color(marker: BulletMarker, rest: &str) -> Bullet {
    if let Some(tagged) = rest.strip_prefix("{c:")
        && let Some(end) = tagged.find("} ")
    {
        let tag = &tagged[..end];
        if let Some(color) = AnnotationColor::from_tag(tag) {
            let text = tagged[end + 2..].to_owned();
            return Bullet::with_marker_and_color(marker, text, color);
        }
    }
    Bullet::with_marker(marker, rest)
}

fn strip_marker<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(marker)?;
    if rest.is_empty() {
        Some(rest)
    } else {
        rest.strip_prefix(' ')
    }
}
