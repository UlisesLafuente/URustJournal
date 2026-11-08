//! File-system storage for journals.
//!
//! Journals are stored as UTF-8 plain-text files named `<journal>.txt` inside
//! a single directory. By default that directory lives under the platform's
//! user data directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use super::{Journal, JournalName};
use crate::error::{JournalError, Result};

const QUALIFIER: &str = "org";
const ORGANIZATION: &str = "UrustJournal";
const APPLICATION: &str = "UrustJournal";
const JOURNALS_DIRECTORY: &str = "journals";
const FILE_EXTENSION: &str = "txt";

/// Reads and writes journals on disk.
#[derive(Debug, Clone)]
pub struct JournalStorage {
    directory: PathBuf,
}

impl JournalStorage {
    /// Opens the per-user storage directory and creates it if necessary.
    pub fn new_default() -> Result<Self> {
        let project = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| {
            JournalError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "no se pudo determinar el directorio de datos del sistema",
            ))
        })?;
        Self::with_dir(project.data_dir().join(JOURNALS_DIRECTORY))
    }

    /// Opens a storage rooted at `directory`, creating it if necessary.
    pub fn with_dir(directory: impl Into<PathBuf>) -> Result<Self> {
        let storage = Self {
            directory: directory.into(),
        };
        storage.ensure_directory()?;
        Ok(storage)
    }

    /// Returns the directory where journals are stored.
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn ensure_directory(&self) -> Result<()> {
        fs::create_dir_all(&self.directory)?;
        Ok(())
    }

    fn path_for(&self, name: &JournalName) -> PathBuf {
        self.directory
            .join(format!("{}.{FILE_EXTENSION}", name.as_str()))
    }

    /// Returns whether a journal with `name` exists on disk.
    pub fn exists(&self, name: &JournalName) -> bool {
        self.path_for(name).is_file()
    }

    /// Creates a new empty journal, failing if it already exists.
    pub fn create(&self, name: JournalName) -> Result<Journal> {
        self.ensure_directory()?;
        let path = self.path_for(&name);
        if path.exists() {
            return Err(JournalError::AlreadyExists(name.as_str().to_owned()));
        }
        let journal = Journal::new(name);
        fs::write(&path, journal.to_text())?;
        Ok(journal)
    }

    /// Loads a journal from disk.
    pub fn load(&self, name: &JournalName) -> Result<Journal> {
        let path = self.path_for(name);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(JournalError::NotFound(name.as_str().to_owned()));
            }
            Err(error) => return Err(JournalError::Io(error)),
        };
        Ok(Journal::from_text(name.clone(), &text))
    }

    /// Writes the journal to disk.
    pub fn save(&self, journal: &Journal) -> Result<()> {
        self.ensure_directory()?;
        fs::write(self.path_for(journal.name()), journal.to_text())?;
        Ok(())
    }

    /// Deletes a journal from disk.
    pub fn delete(&self, name: &JournalName) -> Result<()> {
        match fs::remove_file(self.path_for(name)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(JournalError::NotFound(name.as_str().to_owned()))
            }
            Err(error) => Err(JournalError::Io(error)),
        }
    }

    /// Renames an existing journal, failing if the target name already exists.
    pub fn rename(&self, old: &JournalName, new: JournalName) -> Result<()> {
        let from = self.path_for(old);
        let to = self.path_for(&new);
        if !from.is_file() {
            return Err(JournalError::NotFound(old.as_str().to_owned()));
        }
        if to.exists() {
            return Err(JournalError::AlreadyExists(new.as_str().to_owned()));
        }
        fs::rename(from, to)?;
        Ok(())
    }

    /// Lists all stored journals, sorted by name.
    pub fn list(&self) -> Result<Vec<JournalName>> {
        self.ensure_directory()?;
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.directory)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some(FILE_EXTENSION) {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if let Ok(name) = JournalName::new(stem) {
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }
}
