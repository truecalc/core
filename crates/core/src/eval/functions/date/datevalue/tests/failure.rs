use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(datevalue_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [Value::Text("2024-01-15".to_string()), Value::Text("extra".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_string() {
    let args = [Value::Text("abc".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn empty_string() {
    let args = [Value::Text("".to_string())];
    assert_eq!(datevalue_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_string_arg() {
    let args = [Value::Number(45306.0)];
    assert_eq!(datevalue_fn(&args), Value::Error(ErrorKind::Value));
}
