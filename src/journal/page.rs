//! A page inside a journal section.

use super::Bullet;

/// A titled collection of bullets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Page {
    title: String,
    bullets: Vec<Bullet>,
}

impl Page {
    /// Creates an empty page with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            bullets: Vec::new(),
        }
    }

    /// Borrows the page title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrows all bullets on the page.
    pub fn bullets(&self) -> &[Bullet] {
        &self.bullets
    }

    /// Returns whether the page has no bullets.
    pub fn is_empty(&self) -> bool {
        self.bullets.is_empty()
    }

    /// Appends a bullet to the page.
    pub fn push(&mut self, bullet: Bullet) {
        self.bullets.push(bullet);
    }

    /// Mutably borrows the bullet at `index`.
    pub fn bullet_mut(&mut self, index: usize) -> Option<&mut Bullet> {
        self.bullets.get_mut(index)
    }

    /// Removes the bullet at `index`, if any.
    pub fn remove(&mut self, index: usize) -> Option<Bullet> {
        if index < self.bullets.len() {
            Some(self.bullets.remove(index))
        } else {
            None
        }
    }

    /// Moves a bullet from one index to another.
    pub fn move_bullet(&mut self, from: usize, to: usize) -> bool {
        let len = self.bullets.len();
        if from >= len || to >= len || from == to {
            return false;
        }
        let bullet = self.bullets.remove(from);
        self.bullets.insert(to, bullet);
        true
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
