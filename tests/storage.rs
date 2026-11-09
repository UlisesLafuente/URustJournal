use std::fs;

use tempfile::TempDir;
use ujournal::JournalError;
use ujournal::journal::{Bullet, JournalName, JournalStorage, SectionKind};

fn storage() -> (TempDir, JournalStorage) {
    let temp = tempfile::tempdir().expect("temp dir");
    let storage =
        JournalStorage::with_dir(temp.path().join("journals")).expect("storage directory");
    (temp, storage)
}

fn name(raw: &str) -> JournalName {
    JournalName::new(raw).expect("valid name")
}

#[test]
fn creates_directory_and_journal_file() {
    let (temp, storage) = storage();
    let dir = temp.path().join("journals");
    assert!(dir.is_dir());

    let journal = storage.create(name("first")).expect("create journal");
    assert_eq!(journal.name(), &name("first"));
    assert!(dir.join("first.txt").is_file());
}

#[test]
fn create_does_not_overwrite_existing_journal() {
    let (_temp, storage) = storage();
    let mut journal = storage.create(name("keep")).expect("create journal");
    let index = journal.add_page(SectionKind::Journal, "Day").unwrap();
    journal
        .page_mut(SectionKind::Journal, index)
        .unwrap()
        .push(Bullet::note("contenido importante"));
    storage.save(&journal).expect("save journal");

    let error = storage.create(name("keep")).unwrap_err();
    assert!(matches!(error, JournalError::AlreadyExists(_)));

    let loaded = storage.load(&name("keep")).expect("load journal");
    assert_eq!(
        loaded.page(SectionKind::Journal, index).unwrap().bullets(),
        &[Bullet::note("contenido importante")]
    );
}

#[test]
fn loads_saved_content() {
    let (_temp, storage) = storage();
    let mut journal = storage.create(name("saved")).expect("create journal");
    let index = journal.add_page(SectionKind::Journal, "Day").unwrap();
    let page = journal.page_mut(SectionKind::Journal, index).unwrap();
    page.push(Bullet::task("comprar pan"));
    page.push(Bullet::event("reunión"));
    storage.save(&journal).expect("save journal");

    let loaded = storage.load(&name("saved")).expect("load journal");
    let page = loaded.page(SectionKind::Journal, index).unwrap();
    assert_eq!(page.bullets().len(), 2);
    assert_eq!(page.bullets()[0], Bullet::task("comprar pan"));
}

#[test]
fn persists_pages_across_all_sections() {
    let (_temp, storage) = storage();
    let mut journal = storage.create(name("pages")).expect("create journal");

    for (kind, title) in [
        (SectionKind::FutureLog, "2027-01"),
        (SectionKind::MonthlyLog, "2026-11"),
        (SectionKind::Journal, "2026-11-08"),
    ] {
        let index = journal.add_page(kind, title).unwrap();
        journal
            .page_mut(kind, index)
            .unwrap()
            .push(Bullet::note(title));
    }
    storage.save(&journal).expect("save journal");

    let loaded = storage.load(&name("pages")).expect("load journal");
    assert_eq!(loaded, journal);
    assert_eq!(loaded.total_pages(), 3);
}

#[test]
fn loading_missing_journal_reports_not_found() {
    let (_temp, storage) = storage();
    let error = storage.load(&name("missing")).unwrap_err();
    assert!(matches!(error, JournalError::NotFound(_)));
}

#[test]
fn lists_journals_sorted_by_name() {
    let (_temp, storage) = storage();
    for raw in ["zeta", "alpha", "middle"] {
        storage.create(name(raw)).expect("create journal");
    }

    let names: Vec<String> = storage
        .list()
        .expect("list journals")
        .into_iter()
        .map(|name| name.as_str().to_owned())
        .collect();
    assert_eq!(names, ["alpha", "middle", "zeta"]);
}

#[test]
fn list_ignores_non_journal_files() {
    let (temp, storage) = storage();
    storage.create(name("real")).expect("create journal");
    fs::write(temp.path().join("journals").join("notes.md"), "ignore me")
        .expect("write foreign file");

    let names = storage.list().expect("list journals");
    assert_eq!(names, [name("real")]);
}

#[test]
fn deletes_journal() {
    let (_temp, storage) = storage();
    storage.create(name("gone")).expect("create journal");
    assert!(storage.exists(&name("gone")));

    storage.delete(&name("gone")).expect("delete journal");
    assert!(!storage.exists(&name("gone")));
    assert!(matches!(
        storage.delete(&name("gone")).unwrap_err(),
        JournalError::NotFound(_)
    ));
}

#[test]
fn renames_journal() {
    let (_temp, storage) = storage();
    storage.create(name("before")).expect("create journal");

    storage
        .rename(&name("before"), name("after"))
        .expect("rename journal");
    assert!(!storage.exists(&name("before")));
    assert!(storage.exists(&name("after")));
}

#[test]
fn rename_fails_when_target_exists() {
    let (_temp, storage) = storage();
    storage.create(name("one")).expect("create journal");
    storage.create(name("two")).expect("create journal");

    let error = storage.rename(&name("one"), name("two")).unwrap_err();
    assert!(matches!(error, JournalError::AlreadyExists(_)));
    assert!(storage.exists(&name("one")));
    assert!(storage.exists(&name("two")));
}

#[test]
fn rename_missing_journal_reports_not_found() {
    let (_temp, storage) = storage();
    let error = storage.rename(&name("missing"), name("other")).unwrap_err();
    assert!(matches!(error, JournalError::NotFound(_)));
}
