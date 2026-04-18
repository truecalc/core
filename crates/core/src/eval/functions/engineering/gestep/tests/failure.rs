use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(gestep_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn three_args_returns_na() {
    assert_eq!(
        gestep_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn non_numeric_first_arg_returns_value() {
    assert_eq!(
        gestep_fn(&[Value::Text("abc".to_string()), Value::Number(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn non_numeric_second_arg_returns_value() {
    assert_eq!(
        gestep_fn(&[Value::Number(1.0), Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
