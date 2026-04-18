use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(year_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    assert_eq!(year_fn(&[Value::Number(1.0), Value::Number(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn text_arg() {
    assert_eq!(year_fn(&[Value::Text("abc".to_string())]), Value::Error(ErrorKind::Value));
}
