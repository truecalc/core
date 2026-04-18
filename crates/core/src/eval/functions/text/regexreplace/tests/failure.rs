use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(regexreplace_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_few_args_returns_na() {
    let r = regexreplace_fn(&[Value::Text("a".into()), Value::Text("b".into())]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let r = regexreplace_fn(&[
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("c".into()),
        Value::Text("d".into()),
    ]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_pattern_returns_value_error() {
    assert_eq!(
        regexreplace_fn(&[
            Value::Text("hello".into()),
            Value::Text("[invalid".into()),
            Value::Text("X".into()),
        ]),
        Value::Error(ErrorKind::Value)
    );
}
