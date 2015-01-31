#![feature(collections)]

extern crate theca;

use theca::utils::{cmp_last_touched, format_field};
use std::cmp::Ordering;

#[test]
fn test_format_field() {
    assert_eq!(
        format_field(
            &"this is some stuff yo".to_string(),
            12,
            false
        ),
         "this is some".to_string()
    );
    assert_eq!(
        format_field(
            &"this is some stuff yo".to_string(),
            11,
            true
        ),
        "this is ...".to_string()
    );
}

#[test]
fn test_cmp_last_touched() {
    let old = "2015-01-22 19:43:24 -0800";
    let new = "2015-01-26 20:18:18 -0800";

    assert!(cmp_last_touched(old, new).is_ok());
    assert!(cmp_last_touched(new, old).is_ok());

    assert_eq!(cmp_last_touched(old, new).ok().unwrap(), Ordering::Less);
    assert_eq!(cmp_last_touched(new, old).ok().unwrap(), Ordering::Greater);
}
