use super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn basic_split() {
    let result = split_fn(&[Value::Text("a,b,c".into()), Value::Text(",".into())]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Text("a".into()),
            Value::Text("b".into()),
            Value::Text("c".into()),
        ])
    );
}

#[test]
fn no_args_returns_na() {
    assert_eq!(split_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    assert_eq!(split_fn(&[Value::Text("a,b".into())]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    // SPLIT accepts up to 4 args; 5 is too many
    let r = split_fn(&[
        Value::Text("a,b".into()),
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Bool(true),
        Value::Text("extra".into()),
    ]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}
