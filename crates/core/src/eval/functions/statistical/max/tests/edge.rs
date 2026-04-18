use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn max_no_args_returns_na() {
    assert_eq!(max_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn max_text_in_args_returns_value_error() {
    assert_eq!(
        max_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn max_negative_numbers() {
    assert_eq!(
        max_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-1.0)
    );
}
