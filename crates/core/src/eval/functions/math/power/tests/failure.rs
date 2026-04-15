use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_value_error() {
    assert_eq!(power_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn one_arg_returns_value_error() {
    assert_eq!(power_fn(&[Value::Number(2.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_base_returns_value_error() {
    assert_eq!(
        power_fn(&[Value::Text("abc".to_string()), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}
