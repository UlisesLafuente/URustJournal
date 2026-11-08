use urust_journal::journal::{Bullet, Journal, JournalName};

fn name(raw: &str) -> JournalName {
    JournalName::new(raw).expect("valid name")
}

#[test]
fn parses_every_bullet_kind() {
    let cases = [
        ("- [ ] pending", Bullet::task("pending")),
        (
            "- [x] done",
            Bullet::Task {
                text: "done".to_owned(),
                completed: true,
            },
        ),
        ("- [>] later", Bullet::migrated("later")),
        ("- [<] someday", Bullet::scheduled("someday")),
        ("o meeting", Bullet::event("meeting")),
        ("- a note", Bullet::note("a note")),
        ("# Heading", Bullet::Raw("# Heading".to_owned())),
        ("", Bullet::Raw(String::new())),
    ];

    for (line, expected) in cases {
        assert_eq!(Bullet::parse(line), expected, "line: {line:?}");
    }
}

#[test]
fn renders_bullets_without_losing_content() {
    let bullets = [
        Bullet::task("pending"),
        Bullet::Task {
            text: "done".to_owned(),
            completed: true,
        },
        Bullet::migrated("later"),
        Bullet::scheduled("someday"),
        Bullet::event("meeting"),
        Bullet::note("a note"),
        Bullet::Raw("# Heading".to_owned()),
    ];

    let expected = [
        "- [ ] pending",
        "- [x] done",
        "- [>] later",
        "- [<] someday",
        "o meeting",
        "- a note",
        "# Heading",
    ];

    for (bullet, line) in bullets.iter().zip(expected) {
        assert_eq!(bullet.render(), line);
    }
}

#[test]
fn text_round_trips_through_journal() {
    let source = "- [ ] pending\n- [x] done\no meeting\n- a note\n# Heading\n";
    let journal = Journal::from_text(name("roundtrip"), source);

    assert_eq!(journal.to_text(), source);
    assert_eq!(journal.bullets().len(), 5);
}

#[test]
fn toggling_tasks_only_affects_tasks() {
    let mut journal = Journal::new(name("toggle"));
    journal.push(Bullet::task("pending"));
    journal.push(Bullet::note("note"));

    assert!(journal.toggle(0));
    assert_eq!(
        journal.bullets()[0],
        Bullet::Task {
            text: "pending".to_owned(),
            completed: true,
        }
    );

    assert!(journal.toggle(1));
    assert_eq!(journal.bullets()[1], Bullet::note("note"));
}

#[test]
fn remove_returns_none_out_of_bounds() {
    let mut journal = Journal::new(name("remove"));
    journal.push(Bullet::note("only"));

    assert!(journal.remove(0).is_some());
    assert!(journal.remove(0).is_none());
    assert!(journal.is_empty());
}
