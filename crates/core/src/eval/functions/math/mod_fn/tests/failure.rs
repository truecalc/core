use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn divisor_zero_returns_div_by_zero() {
    assert_eq!(
        mod_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(mod_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_first_arg() {
    assert_eq!(
        mod_fn(&[Value::Text("abc".to_string()), Value::Number(3.0)]),
        Value::Error(ErrorKind::Value)
    );
}
