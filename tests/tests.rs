#![allow(unstable)]

extern crate theca;
use theca::{ThecaProfile, Args};

#[test]
fn test_add_note() {
    let mut profile = ThecaProfile {
        encrypted: false,
        notes: vec![]
    };
    let test_args = Args {
        cmd_add: true,
        cmd_clear: false,
        cmd_del: false,
        cmd_edit: false,
        cmd_info: false,
        cmd_new_profile: false,
        cmd_search: false,
        cmd_transfer: false,
        cmd__: false,
        arg_id: vec![],
        arg_name: vec!["".to_string()],
        arg_pattern: "".to_string(),
        arg_title: "the title".to_string(),
        flag_append: "".to_string(),
        flag_body: "".to_string(),
        flag_condensed: false,
        flag_datesort: false,
        flag_editor: false,
        flag_encrypted: false,
        flag_key: "".to_string(),
        flag_limit: 0,
        flag_none: false,
        flag_prepend: "".to_string(),
        flag_profile: "".to_string(),
        flag_profile_folder: "".to_string(),
        flag_regex: false,
        flag_reverse: false,
        flag_search_body: false,
        flag_started: false,
        flag_urgent: false,
        flag_version: false,
        flag_yes: false
    };
    match profile.add_item(&test_args) {
        Ok(_) => {},
        Err(_) => assert!(false) // ?
    };
    assert_eq!(profile.notes[0].id, 1);
    assert_eq!(profile.notes[0].title, "the title".to_string());
    assert_eq!(profile.notes[0].status, "".to_string());
    assert_eq!(profile.notes[0].body, "".to_string());
}

// #[test]
// fn test_add_full_note() {
//     let mut profile = ThecaProfile {
//         encrypted: false,
//         notes: vec![]
//     };
//     let test_args = Args {
//         flag_profiles_folder: "".to_string(),
//         flag_p: "".to_string(),
//         cmd_new_profile: false,
//         cmd_search: false,
//         flag_body: false,
//         flag_reverse: false,
//         cmd_add: true,
//         cmd_edit: false,
//         cmd_del: false,
//         arg_name: "".to_string(),
//         arg_pattern: "".to_string(),
//         flag_encrypted: false,
//         flag_key: "".to_string(),
//         flag_c: false,
//         flag_l: 0,
//         arg_title: "the title".to_string(),
//         flag_started: true,
//         flag_urgent: false,
//         flag_none: false,
//         flag_b: "this is a body".to_string(),
//         flag_editor: false,
//         cmd__: false,
//         arg_id: vec![],
//         flag_h: false,
//         flag_v: false,
//         cmd_info: false
//     };
//     profile.add_item(&test_args);
//     assert_eq!(profile.notes[0].id, 1);
//     assert_eq!(profile.notes[0].title, "the title".to_string());
//     assert_eq!(profile.notes[0].status, "Started".to_string());
//     assert_eq!(profile.notes[0].body, "this is a body".to_string());
// }

// #[test]
// fn test_add_del_note() {
//     let mut profile = ThecaProfile {
//         encrypted: false,
//         notes: vec![]
//     };
//     let add_args = Args {
//         flag_profiles_folder: "".to_string(),
//         flag_p: "".to_string(),
//         cmd_new_profile: false,
//         cmd_search: false,
//         flag_body: false,
//         flag_reverse: false,
//         cmd_add: true,
//         cmd_edit: false,
//         cmd_del: false,
//         arg_name: "".to_string(),
//         arg_pattern: "".to_string(),
//         flag_encrypted: false,
//         flag_key: "".to_string(),
//         flag_c: false,
//         flag_l: 0,
//         arg_title: "the title".to_string(),
//         flag_started: true,
//         flag_urgent: false,
//         flag_none: false,
//         flag_b: "this is a body".to_string(),
//         flag_editor: false,
//         cmd__: false,
//         arg_id: vec![],
//         flag_h: false,
//         flag_v: false,
//         cmd_info: false
//     };

//     profile.add_item(&add_args);
//     profile.delete_item(1);

//     assert_eq!(profile.notes.len(), 0);
// }

// #[test]
// fn test_add_add_del_add_note() {
//     let mut profile = ThecaProfile {
//         encrypted: false,
//         notes: vec![]
//     };
//     let add_args = Args {
//         flag_profiles_folder: "".to_string(),
//         flag_p: "".to_string(),
//         cmd_new_profile: false,
//         cmd_search: false,
//         flag_body: false,
//         flag_reverse: false,
//         cmd_add: true,
//         cmd_edit: false,
//         cmd_del: false,
//         arg_name: "".to_string(),
//         arg_pattern: "".to_string(),
//         flag_encrypted: false,
//         flag_key: "".to_string(),
//         flag_c: false,
//         flag_l: 0,
//         arg_title: "the title".to_string(),
//         flag_started: true,
//         flag_urgent: false,
//         flag_none: false,
//         flag_b: "this is a body".to_string(),
//         flag_editor: false,
//         cmd__: false,
//         arg_id: vec![],
//         flag_h: false,
//         flag_v: false,
//         cmd_info: false
//     };

//     profile.add_item(&add_args);
//     profile.add_item(&add_args);
//     profile.add_item(&add_args);
//     profile.delete_item(2);
//     profile.add_item(&add_args);

//     assert_eq!(profile.notes[0].id, 1);
//     assert_eq!(profile.notes[0].title, "the title".to_string());
//     assert_eq!(profile.notes[0].status, "Started".to_string());
//     assert_eq!(profile.notes[0].body, "this is a body".to_string());

//     assert_eq!(profile.notes[2].id, 4);
//     assert_eq!(profile.notes[2].title, "the title".to_string());
//     assert_eq!(profile.notes[2].status, "Started".to_string());
//     assert_eq!(profile.notes[2].body, "this is a body".to_string());
// }
