use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(timevalue_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [Value::Text("12:00:00".to_string()), Value::Text("extra".to_string())];
    assert_eq!(timevalue_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_string() {
    let args = [Value::Text("abc".to_string())];
    assert_eq!(timevalue_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_string_arg() {
    let args = [Value::Number(0.5)];
    assert_eq!(timevalue_fn(&args), Value::Error(ErrorKind::Value));
}
