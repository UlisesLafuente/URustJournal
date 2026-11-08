//! In-memory journal model.

use super::{Bullet, JournalName};

/// A journal loaded in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Journal {
    name: JournalName,
    bullets: Vec<Bullet>,
}

impl Journal {
    /// Creates an empty journal.
    pub fn new(name: JournalName) -> Self {
        Self {
            name,
            bullets: Vec::new(),
        }
    }

    /// Builds a journal by parsing plain-text content.
    pub fn from_text(name: JournalName, text: &str) -> Self {
        Self {
            name,
            bullets: text.lines().map(Bullet::parse).collect(),
        }
    }

    /// Serializes the journal back to plain text.
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        for bullet in &self.bullets {
            output.push_str(&bullet.render());
            output.push('\n');
        }
        output
    }

    /// Borrows the journal name.
    pub fn name(&self) -> &JournalName {
        &self.name
    }

    /// Renames the journal in memory.
    pub fn rename(&mut self, name: JournalName) {
        self.name = name;
    }

    /// Borrows all bullets.
    pub fn bullets(&self) -> &[Bullet] {
        &self.bullets
    }

    /// Returns whether the journal has no bullets.
    pub fn is_empty(&self) -> bool {
        self.bullets.is_empty()
    }

    /// Appends a bullet.
    pub fn push(&mut self, bullet: Bullet) {
        self.bullets.push(bullet);
    }

    /// Removes the bullet at `index`, if any.
    pub fn remove(&mut self, index: usize) -> Option<Bullet> {
        if index < self.bullets.len() {
            Some(self.bullets.remove(index))
        } else {
            None
        }
    }

    /// Toggles the task at `index`. Returns whether the index existed.
    pub fn toggle(&mut self, index: usize) -> bool {
        match self.bullets.get_mut(index) {
            Some(bullet) => {
                bullet.toggle();
                true
            }
            None => false,
        }
    }
}
