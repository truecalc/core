use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn pi_with_args_returns_value_error() {
    assert_eq!(pi_fn(&[Value::Number(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn sin_no_args_returns_value_error() {
    assert_eq!(sin_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn cos_non_numeric_returns_value_error() {
    assert_eq!(
        cos_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
