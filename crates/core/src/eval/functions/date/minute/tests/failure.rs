use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(minute_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    assert_eq!(minute_fn(&[Value::Number(0.5), Value::Number(0.5)]), Value::Error(ErrorKind::NA));
}

#[test]
fn text_arg() {
    assert_eq!(minute_fn(&[Value::Text("abc".to_string())]), Value::Error(ErrorKind::Value));
}
