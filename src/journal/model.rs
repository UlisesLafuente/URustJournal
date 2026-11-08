//! In-memory journal model and its plain-text serialization.
//!
//! A journal is divided into three fixed sections (future log, monthly log and
//! the journal itself), each holding an ordered list of pages.
//!
//! The on-disk format is line based and human readable:
//!
//! ```text
//! @@section future_log
//! @@page 2027-01
//! o New year trip
//! @@section monthly_log
//! @@section journal
//! @@page 2026-11-08
//! - [ ] Buy bread
//! ```
//!
//! Content lines that begin with `@@` or `\` are escaped with a leading
//! backslash so they can never be confused with section or page markers.

use super::{Bullet, JournalName, Page, Section, SectionKind};
use crate::error::Result;

const SECTION_PREFIX: &str = "@@section ";
const PAGE_PREFIX: &str = "@@page ";
const ESCAPE: char = '\\';
const LEGACY_PAGE_TITLE: &str = "General";

/// A journal loaded in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Journal {
    name: JournalName,
    future_log: Section,
    monthly_log: Section,
    journal: Section,
}

impl Journal {
    /// Creates an empty journal with the three sections.
    pub fn new(name: JournalName) -> Self {
        Self {
            name,
            future_log: Section::new(),
            monthly_log: Section::new(),
            journal: Section::new(),
        }
    }

    /// Borrows the journal name.
    pub fn name(&self) -> &JournalName {
        &self.name
    }

    /// Renames the journal in memory.
    pub fn rename(&mut self, name: JournalName) {
        self.name = name;
    }

    /// Borrows a section.
    pub fn section(&self, kind: SectionKind) -> &Section {
        match kind {
            SectionKind::FutureLog => &self.future_log,
            SectionKind::MonthlyLog => &self.monthly_log,
            SectionKind::Journal => &self.journal,
        }
    }

    /// Mutably borrows a section.
    pub fn section_mut(&mut self, kind: SectionKind) -> &mut Section {
        match kind {
            SectionKind::FutureLog => &mut self.future_log,
            SectionKind::MonthlyLog => &mut self.monthly_log,
            SectionKind::Journal => &mut self.journal,
        }
    }

    /// Adds a page to a section and returns its index.
    pub fn add_page(&mut self, kind: SectionKind, title: &str) -> Result<usize> {
        self.section_mut(kind).add_page(title)
    }

    /// Borrows a page.
    pub fn page(&self, kind: SectionKind, index: usize) -> Option<&Page> {
        self.section(kind).page(index)
    }

    /// Mutably borrows a page.
    pub fn page_mut(&mut self, kind: SectionKind, index: usize) -> Option<&mut Page> {
        self.section_mut(kind).page_mut(index)
    }

    /// Removes a page from a section.
    pub fn remove_page(&mut self, kind: SectionKind, index: usize) -> Option<Page> {
        self.section_mut(kind).remove_page(index)
    }

    /// Returns the total number of pages across all sections.
    pub fn total_pages(&self) -> usize {
        self.future_log.len() + self.monthly_log.len() + self.journal.len()
    }

    /// Serializes the journal back to plain text.
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        for kind in SectionKind::ALL {
            output.push_str(SECTION_PREFIX);
            output.push_str(kind.as_str());
            output.push('\n');
            for page in self.section(kind).pages() {
                output.push_str(PAGE_PREFIX);
                output.push_str(page.title());
                output.push('\n');
                for bullet in page.bullets() {
                    let line = bullet.render();
                    if needs_escape(&line) {
                        output.push(ESCAPE);
                    }
                    output.push_str(&line);
                    output.push('\n');
                }
            }
        }
        output
    }

    /// Builds a journal by parsing plain-text content.
    ///
    /// Files written by the previous single-section format (plain bullets with
    /// no markers) are loaded into a page called `General` in the journal
    /// section.
    pub fn from_text(name: JournalName, text: &str) -> Self {
        let mut journal = Self::new(name);

        let structured = text.lines().any(|line| {
            line.strip_prefix(SECTION_PREFIX).is_some() || line.strip_prefix(PAGE_PREFIX).is_some()
        });
        if !structured {
            if !text.is_empty() {
                let mut page = Page::new(LEGACY_PAGE_TITLE);
                for line in text.lines() {
                    page.push(Bullet::parse(line));
                }
                journal.journal.push_page(page);
            }
            return journal;
        }

        let mut current_kind = SectionKind::Journal;
        let mut current_page: Option<Page> = None;

        for line in text.lines() {
            if let Some(kind) = line
                .strip_prefix(SECTION_PREFIX)
                .and_then(SectionKind::parse)
            {
                flush_page(&mut journal, current_kind, &mut current_page);
                current_kind = kind;
                continue;
            }

            if let Some(title) = line.strip_prefix(PAGE_PREFIX) {
                flush_page(&mut journal, current_kind, &mut current_page);
                current_page = Some(Page::new(title.trim().to_owned()));
                continue;
            }

            let content = unescape(line).unwrap_or(line);
            match current_page.as_mut() {
                Some(page) => page.push(Bullet::parse(content)),
                None if !content.is_empty() => {
                    let mut page = Page::new(LEGACY_PAGE_TITLE);
                    page.push(Bullet::parse(content));
                    current_page = Some(page);
                }
                None => {}
            }
        }

        flush_page(&mut journal, current_kind, &mut current_page);
        journal
    }
}

fn flush_page(journal: &mut Journal, kind: SectionKind, current_page: &mut Option<Page>) {
    if let Some(page) = current_page.take() {
        journal.section_mut(kind).push_page(page);
    }
}

fn needs_escape(line: &str) -> bool {
    line.starts_with("@@") || line.starts_with(ESCAPE)
}

fn unescape(line: &str) -> Option<&str> {
    if line.starts_with("\\\\") || line.starts_with("\\@@") {
        Some(&line[1..])
    } else {
        None
    }
}
