use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(regexextract_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let r = regexextract_fn(&[
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}

#[test]
fn no_match_returns_na() {
    assert_eq!(
        regexextract_fn(&[Value::Text("hello".into()), Value::Text("[0-9]+".into())]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn capture_group_returns_first_group() {
    // With capture groups, REGEXEXTRACT returns the first group's content.
    assert_eq!(
        regexextract_fn(&[
            Value::Text("2024-01-15".into()),
            Value::Text("([0-9]{4})-([0-9]{2})".into())
        ]),
        Value::Text("2024".into())
    );
}
