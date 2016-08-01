extern crate theca;

use theca::{Status, ThecaProfile};

#[test]
fn test_add_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::NoStatus);
    assert_eq!(p.notes[0].body, "".to_string());
}

#[test]
fn test_add_started_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], true, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::Started);
    assert_eq!(p.notes[0].body, "".to_string());
}

#[test]
fn test_add_urgent_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, true, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::Urgent);
    assert_eq!(p.notes[0].body, "".to_string());
}

#[test]
fn test_add_basic_body_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec!["and what?".to_string()], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::NoStatus);
    assert_eq!(p.notes[0].body, "and what?".to_string());
}

#[test]
fn test_add_full_basic_body_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec!["and what?".to_string()], false, true, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::Urgent);
    assert_eq!(p.notes[0].body, "and what?".to_string());
}

#[test]
fn test_edit_note_title() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert!(p.edit_note(1, &"this is a new title".to_string(), &vec![], false, false, false, false, false, false, false).is_ok());
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a new title".to_string());
    assert_eq!(p.notes[0].status, Status::NoStatus);
    assert_eq!(p.notes[0].body, "".to_string());
}

#[test]
fn test_edit_note_status() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert!(p.edit_note(1, &"".to_string(), &vec![], true, false, false, false, false, false, false).is_ok());
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::Started);
    assert_eq!(p.notes[0].body, "".to_string());
    assert!(p.edit_note(1, &"".to_string(), &vec![], false, true, false, false, false, false, false).is_ok());
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::Urgent);
    assert_eq!(p.notes[0].body, "".to_string());
    assert!(p.edit_note(1, &"".to_string(), &vec![], false, false, true, false, false, false, false).is_ok());
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::NoStatus);
    assert_eq!(p.notes[0].body, "".to_string());
}

#[test]
fn test_edit_note_body_basic() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert!(p.edit_note(1, &"".to_string(), &vec!["woo body".to_string()], false, false, false, false, false, false, false).is_ok());
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::NoStatus);
    assert_eq!(p.notes[0].body, "woo body".to_string());
}

#[test]
fn test_edit_full_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert!(p.edit_note(1, &"this is a new title".to_string(), &vec!["woo body".to_string()], true, false, false, false, false, false, false).is_ok());
    assert_eq!(p.notes[0].id, 1);
    assert_eq!(p.notes[0].title, "this is a new title".to_string());
    assert_eq!(p.notes[0].status, Status::Started);
    assert_eq!(p.notes[0].body, "woo body".to_string());
}

#[test]
fn test_delete_single_note() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    p.delete_note(&vec![1]);
    assert_eq!(p.notes.len(), 0);
}

#[test]
fn test_delete_some_notes() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 2);
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 3);
    p.delete_note(&vec![1,3]);
    assert_eq!(p.notes.len(), 1);
    assert_eq!(p.notes[0].id, 2);
    assert_eq!(p.notes[0].title, "this is a title".to_string());
    assert_eq!(p.notes[0].status, Status::NoStatus);
    assert_eq!(p.notes[0].body, "".to_string());
}

#[test]
fn test_clear_notes() {
    let mut p = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 1);
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 2);
    assert!(p.add_note(&"this is a title".to_string(), &vec![], false, false, false, false, false).is_ok());
    assert_eq!(p.notes.len(), 3);

    assert!(p.clear(true).is_ok());
    assert_eq!(p.notes.len(), 0);
}
