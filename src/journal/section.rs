//! Journal sections and their pages.

use super::Page;
use crate::error::{JournalError, Result};

/// Maximum accepted length of a page title, in characters.
pub const MAX_PAGE_TITLE_LEN: usize = 120;

/// The three collections that make up a journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionKind {
    /// Long-term collection for future events and tasks.
    FutureLog,
    /// Collection for the current month.
    MonthlyLog,
    /// The journal itself, used for day-to-day pages.
    Journal,
}

impl SectionKind {
    /// Sections shown in the UI.
    pub const ALL: [Self; 2] = [Self::FutureLog, Self::MonthlyLog];

    /// Human label used by the UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::FutureLog => "Future Log",
            Self::MonthlyLog => "Monthly Log",
            Self::Journal => "Journal",
        }
    }

    /// Stable identifier used by the on-disk format.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FutureLog => "future_log",
            Self::MonthlyLog => "monthly_log",
            Self::Journal => "journal",
        }
    }

    /// Parses a stable identifier.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "future_log" => Some(Self::FutureLog),
            "monthly_log" => Some(Self::MonthlyLog),
            "journal" => Some(Self::Journal),
            _ => None,
        }
    }
}

/// An ordered collection of pages.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Section {
    pages: Vec<Page>,
}

impl Section {
    /// Creates an empty section.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrows all pages.
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    /// Returns whether the section has no pages.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Returns the number of pages.
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Borrows the page at `index`.
    pub fn page(&self, index: usize) -> Option<&Page> {
        self.pages.get(index)
    }

    /// Mutably borrows the page at `index`.
    pub fn page_mut(&mut self, index: usize) -> Option<&mut Page> {
        self.pages.get_mut(index)
    }

    /// Adds a page with a validated, unique title and returns its index.
    pub fn add_page(&mut self, title: &str) -> Result<usize> {
        let title = normalize_title(title)?;
        if self.pages.iter().any(|page| page.title() == title) {
            return Err(JournalError::DuplicatePage(title));
        }
        self.pages.push(Page::new(title));
        Ok(self.pages.len() - 1)
    }

    /// Removes the page at `index`, if any.
    pub fn remove_page(&mut self, index: usize) -> Option<Page> {
        if index < self.pages.len() {
            Some(self.pages.remove(index))
        } else {
            None
        }
    }

    pub(crate) fn push_page(&mut self, page: Page) {
        self.pages.push(page);
    }
}

fn normalize_title(raw: &str) -> Result<String> {
    let title = raw.trim();
    if title.is_empty() {
        return Err(JournalError::InvalidPage(
            "el título no puede estar vacío".to_owned(),
        ));
    }
    if title.chars().count() > MAX_PAGE_TITLE_LEN {
        return Err(JournalError::InvalidPage(format!(
            "el título supera los {MAX_PAGE_TITLE_LEN} caracteres"
        )));
    }
    if title
        .chars()
        .any(|character| character == '\n' || character == '\r')
    {
        return Err(JournalError::InvalidPage(
            "el título no puede contener saltos de línea".to_owned(),
        ));
    }
    Ok(title.to_owned())
}
