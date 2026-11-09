use urust_journal::JournalError;
use urust_journal::journal::{Bullet, Journal, JournalName, SectionKind};

fn name(raw: &str) -> JournalName {
    JournalName::new(raw).expect("valid name")
}

#[test]
fn new_journal_has_three_empty_sections() {
    let journal = Journal::new(name("empty"));

    for kind in SectionKind::ALL {
        assert!(journal.section(kind).is_empty(), "section {kind:?}");
    }
    assert_eq!(journal.total_pages(), 0);
}

#[test]
fn sections_are_independent() {
    let mut journal = Journal::new(name("sections"));
    journal.add_page(SectionKind::FutureLog, "2027-01").unwrap();
    journal
        .add_page(SectionKind::MonthlyLog, "2026-11")
        .unwrap();
    journal
        .add_page(SectionKind::Journal, "2026-11-08")
        .unwrap();

    assert_eq!(journal.section(SectionKind::FutureLog).len(), 1);
    assert_eq!(journal.section(SectionKind::MonthlyLog).len(), 1);
    assert_eq!(journal.section(SectionKind::Journal).len(), 1);
    assert_eq!(
        journal.section(SectionKind::FutureLog).pages()[0].title(),
        "2027-01"
    );
    assert_eq!(journal.total_pages(), 3);
}

#[test]
fn page_titles_must_be_unique_within_a_section() {
    let mut journal = Journal::new(name("unique"));
    journal.add_page(SectionKind::Journal, "Day").unwrap();

    let error = journal.add_page(SectionKind::Journal, "Day").unwrap_err();
    assert!(matches!(error, JournalError::DuplicatePage(_)));

    assert!(journal.add_page(SectionKind::FutureLog, "Day").is_ok());
}

#[test]
fn page_titles_are_validated_and_trimmed() {
    let mut journal = Journal::new(name("titles"));

    let error = journal.add_page(SectionKind::Journal, "   ").unwrap_err();
    assert!(matches!(error, JournalError::InvalidPage(_)));

    let index = journal.add_page(SectionKind::Journal, "  Day  ").unwrap();
    assert_eq!(
        journal.page(SectionKind::Journal, index).unwrap().title(),
        "Day"
    );
}

#[test]
fn removing_a_page_shifts_the_remaining_pages() {
    let mut journal = Journal::new(name("remove_page"));
    journal.add_page(SectionKind::Journal, "one").unwrap();
    journal.add_page(SectionKind::Journal, "two").unwrap();

    let removed = journal.remove_page(SectionKind::Journal, 0).unwrap();
    assert_eq!(removed.title(), "one");
    assert_eq!(
        journal.section(SectionKind::Journal).pages()[0].title(),
        "two"
    );
    assert!(journal.remove_page(SectionKind::Journal, 5).is_none());
}

#[test]
fn legacy_bullets_load_into_general_page() {
    let source = "- [ ] pending\n- a note\n";
    let journal = Journal::from_text(name("legacy"), source);

    let section = journal.section(SectionKind::Journal);
    assert_eq!(section.len(), 1);
    assert_eq!(section.pages()[0].title(), "General");
    assert_eq!(
        section.pages()[0].bullets(),
        &[Bullet::task("pending"), Bullet::note("a note")]
    );
}

#[test]
fn structured_text_round_trips() {
    let mut journal = Journal::new(name("roundtrip"));

    let future = journal.add_page(SectionKind::FutureLog, "2027-01").unwrap();
    journal
        .page_mut(SectionKind::FutureLog, future)
        .unwrap()
        .push(Bullet::event("trip"));

    let monthly = journal
        .add_page(SectionKind::MonthlyLog, "2026-11")
        .unwrap();
    journal
        .page_mut(SectionKind::MonthlyLog, monthly)
        .unwrap()
        .push(Bullet::task("review goals"));

    let daily = journal
        .add_page(SectionKind::Journal, "2026-11-08")
        .unwrap();
    let page = journal.page_mut(SectionKind::Journal, daily).unwrap();
    page.push(Bullet::task("buy bread"));
    page.push(Bullet::note("call mom"));

    let text = journal.to_text();
    let loaded = Journal::from_text(name("roundtrip"), &text);

    assert_eq!(loaded, journal);
}

#[test]
fn marker_like_content_is_escaped() {
    let mut journal = Journal::new(name("escape"));
    let index = journal.add_page(SectionKind::Journal, "Day").unwrap();
    let page = journal.page_mut(SectionKind::Journal, index).unwrap();
    page.push(Bullet::Raw("@@section future_log".to_owned()));
    page.push(Bullet::Raw("@@page fake".to_owned()));
    page.push(Bullet::Raw("\\@@page escaped".to_owned()));

    let text = journal.to_text();
    assert!(text.contains("\\@@section future_log"));

    let loaded = Journal::from_text(name("escape"), &text);
    assert_eq!(loaded, journal);
}

#[test]
fn empty_journal_still_serializes_all_sections() {
    let journal = Journal::new(name("markers"));
    let text = journal.to_text();

    assert!(text.contains("@@section future_log\n"));
    assert!(text.contains("@@section monthly_log\n"));
    assert!(text.contains("@@section journal\n"));
    assert_eq!(Journal::from_text(name("markers"), &text), journal);
}
