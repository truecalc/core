use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn max_propagates_error_values() {
    // Direct errors propagate through max_fn (eager dispatch would handle real formulas)
    assert_eq!(
        max_fn(&[Value::Number(3.0), Value::Error(ErrorKind::Value), Value::Number(5.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn max_text_returns_value_error() {
    // Text in direct args → #VALUE!
    assert_eq!(
        max_fn(&[Value::Number(1.0), Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn max_no_args_returns_na() {
    assert_eq!(max_fn(&[]), Value::Error(ErrorKind::NA));
}
