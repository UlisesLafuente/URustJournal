use urust_journal::journal::{AnnotationColor, Bullet, BulletMarker};

#[test]
fn parses_every_bullet_kind() {
    let cases = [
        ("• pending", Bullet::task("pending")),
        ("· pending", Bullet::task("pending")),
        ("x done", completed("done")),
        ("X done", completed("done")),
        ("× done", completed("done")),
        ("> later", Bullet::migrated("later")),
        ("< someday", Bullet::scheduled("someday")),
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
fn parses_legacy_checkbox_syntax() {
    let cases = [
        ("- [ ] pending", Bullet::task("pending")),
        ("- [x] done", completed("done")),
        ("- [>] later", Bullet::migrated("later")),
        ("- [<] someday", Bullet::scheduled("someday")),
    ];

    for (line, expected) in cases {
        assert_eq!(Bullet::parse(line), expected, "line: {line:?}");
    }
}

#[test]
fn renders_bullets_without_losing_content() {
    let bullets = [
        Bullet::task("pending"),
        completed("done"),
        Bullet::migrated("later"),
        Bullet::scheduled("someday"),
        Bullet::event("meeting"),
        Bullet::note("a note"),
        Bullet::Raw("# Heading".to_owned()),
    ];

    let expected = [
        "• pending",
        "x done",
        "> later",
        "< someday",
        "o meeting",
        "- a note",
        "# Heading",
    ];

    for (bullet, line) in bullets.iter().zip(expected) {
        assert_eq!(bullet.render(), line);
    }
}

#[test]
fn markers_require_a_separating_space() {
    assert_eq!(Bullet::parse("•task"), Bullet::Raw("•task".to_owned()));
    assert_eq!(Bullet::parse("- [ ]task"), Bullet::note("[ ]task"));
    assert_eq!(Bullet::parse("other"), Bullet::Raw("other".to_owned()));
}

#[test]
fn changing_marker_preserves_text() {
    let mut bullet = Bullet::task("buy bread");
    assert_eq!(bullet.marker(), Some(BulletMarker::Task));

    bullet.set_marker(BulletMarker::Migrated);
    assert_eq!(bullet, Bullet::migrated("buy bread"));

    bullet.set_marker(BulletMarker::Completed);
    assert_eq!(bullet, completed("buy bread"));

    bullet.set_marker(BulletMarker::Scheduled);
    assert_eq!(bullet, Bullet::scheduled("buy bread"));

    bullet.set_marker(BulletMarker::Note);
    assert_eq!(bullet, Bullet::note("buy bread"));
}

#[test]
fn raw_lines_ignore_marker_changes() {
    let mut bullet = Bullet::Raw("# Heading".to_owned());
    assert_eq!(bullet.marker(), None);

    bullet.set_marker(BulletMarker::Task);
    assert_eq!(bullet, Bullet::Raw("# Heading".to_owned()));
}

#[test]
fn parses_renders_and_preserves_annotation_colors() {
    let bullet = Bullet::parse("• {c:blue} texto");
    assert_eq!(bullet.color(), AnnotationColor::DesaturatedBlue);
    assert_eq!(bullet.text(), "texto");
    assert_eq!(bullet.render(), "• {c:blue} texto");

    let mut bullet = Bullet::note("nota");
    bullet.set_color(AnnotationColor::EmeraldGreen);
    assert_eq!(bullet.render(), "- {c:emerald} nota");
    bullet.set_marker(BulletMarker::Completed);
    assert_eq!(bullet.color(), AnnotationColor::EmeraldGreen);
    assert_eq!(bullet.render(), "x {c:emerald} nota");
}

#[test]
fn unknown_color_tag_kept_as_plain_text() {
    let bullet = Bullet::parse("• {c:purple} texto");
    assert_eq!(bullet.color(), AnnotationColor::Black);
    assert_eq!(bullet.text(), "{c:purple} texto");
}

fn completed(text: &str) -> Bullet {
    Bullet::Task {
        text: text.to_owned(),
        completed: true,
        color: AnnotationColor::Black,
    }
}
