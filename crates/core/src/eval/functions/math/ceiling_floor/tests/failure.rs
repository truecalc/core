use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn ceiling_no_args_returns_value_error() {
    assert_eq!(ceiling_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn floor_no_args_returns_value_error() {
    assert_eq!(floor_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn ceiling_non_numeric_returns_value_error() {
    assert_eq!(
        ceiling_fn(&[Value::Text("abc".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}
