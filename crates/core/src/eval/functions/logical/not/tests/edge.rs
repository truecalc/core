use super::super::not_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn not_zero_returns_true() {
    assert_eq!(not_fn(&[Value::Number(0.0)]), Value::Bool(true));
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(not_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn array_returns_value_error() {
    assert_eq!(not_fn(&[Value::Array(vec![Value::Bool(true)])]), Value::Error(ErrorKind::Value));
}
