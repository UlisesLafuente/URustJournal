//! Journal domain: validated names, sections, pages, bullets and storage.

mod bullet;
mod model;
mod name;
mod page;
mod section;
mod storage;

pub use bullet::{AnnotationColor, Bullet, BulletMarker};
pub use model::Journal;
pub use name::{JournalName, MAX_NAME_LEN};
pub use page::Page;
pub use section::{MAX_PAGE_TITLE_LEN, Section, SectionKind};
pub use storage::JournalStorage;
