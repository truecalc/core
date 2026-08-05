use super::super::split_fn;
use crate::types::{ErrorKind, Value};

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
    let r = split_fn(&[
        Value::Text("a,b".into()),
        Value::Text(",".into()),
        Value::Bool(true),
        Value::Bool(true),
        Value::Text("extra".into()),
    ]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}
