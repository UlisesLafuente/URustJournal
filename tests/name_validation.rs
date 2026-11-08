use urust_journal::JournalError;
use urust_journal::journal::{JournalName, MAX_NAME_LEN};

#[test]
fn accepts_valid_names() {
    for name in ["journal1", "my_journal", "a_z", "_start", "end_", "ABC123"] {
        assert!(
            JournalName::new(name).is_ok(),
            "expected valid journal name: {name}"
        );
    }
}

#[test]
fn rejects_empty_name() {
    assert!(matches!(
        JournalName::new(""),
        Err(JournalError::InvalidName(_))
    ));
}

#[test]
fn rejects_special_characters() {
    for name in [
        "journal?",
        "journal@",
        "journal!",
        "journal-",
        "journal++",
        "journal..",
        "journal@123",
        "with space",
        "slash/name",
        "back\\slash",
    ] {
        assert!(
            JournalName::new(name).is_err(),
            "expected invalid journal name: {name}"
        );
    }
}

#[test]
fn rejects_overlong_names() {
    let name = "a".repeat(MAX_NAME_LEN + 1);
    assert!(JournalName::new(&name).is_err());

    let name = "a".repeat(MAX_NAME_LEN);
    assert!(JournalName::new(&name).is_ok());
}

#[test]
fn rejects_windows_reserved_names() {
    for name in ["CON", "con", "NUL", "nul", "COM1", "lpt9"] {
        assert!(
            JournalName::new(name).is_err(),
            "expected reserved name to be rejected: {name}"
        );
    }
}
