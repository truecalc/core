use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_value_error() {
    assert_eq!(average_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_text_returns_value_error() {
    assert_eq!(
        average_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn propagates_error_arg() {
    assert_eq!(
        average_fn(&[Value::Error(ErrorKind::Ref)]),
        Value::Error(ErrorKind::Ref)
    );
}
