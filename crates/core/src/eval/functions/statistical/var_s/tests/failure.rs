use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn var_s_no_args_returns_na() {
    assert_eq!(var_s_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn var_s_one_value_returns_div_zero() {
    assert_eq!(var_s_fn(&[Value::Number(5.0)]), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn var_s_non_numeric_text_returns_value_error() {
    assert_eq!(
        var_s_fn(&[Value::Text("a".to_string()), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn var_s_empty_only_returns_div_zero() {
    assert_eq!(var_s_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
