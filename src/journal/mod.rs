//! Journal domain: validated names, bullet model and file storage.

mod bullet;
mod model;
mod name;
mod storage;

pub use bullet::Bullet;
pub use model::Journal;
pub use name::{JournalName, MAX_NAME_LEN};
pub use storage::JournalStorage;
