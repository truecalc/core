use super::super::not_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn text_returns_value_error() {
    assert_eq!(not_fn(&[Value::Text("hello".to_string())]), Value::Error(ErrorKind::Value));
}

#[test]
fn empty_returns_value_error() {
    assert_eq!(not_fn(&[Value::Empty]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args_returns_value_error() {
    assert_eq!(not_fn(&[Value::Bool(true), Value::Bool(false)]), Value::Error(ErrorKind::NA));
}
