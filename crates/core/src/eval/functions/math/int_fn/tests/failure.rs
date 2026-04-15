use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_value_error() {
    assert_eq!(int_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args_returns_value_error() {
    assert_eq!(
        int_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn non_numeric_text_returns_value_error() {
    assert_eq!(
        int_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
