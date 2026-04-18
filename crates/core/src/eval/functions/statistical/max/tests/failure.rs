use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn max_ignores_error_values() {
    // The eager dispatcher strips errors; max_fn itself skips them too
    assert_eq!(
        max_fn(&[Value::Number(3.0), Value::Error(ErrorKind::Value), Value::Number(5.0)]),
        Value::Number(5.0)
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
