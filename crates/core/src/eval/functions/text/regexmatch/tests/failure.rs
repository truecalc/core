use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(regexmatch_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let r = regexmatch_fn(&[
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_pattern_returns_value_error() {
    assert_eq!(
        regexmatch_fn(&[Value::Text("hello".into()), Value::Text("[invalid".into())]),
        Value::Error(ErrorKind::Value)
    );
}
