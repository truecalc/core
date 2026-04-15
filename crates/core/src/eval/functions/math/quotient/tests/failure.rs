use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn denominator_zero_returns_div_by_zero() {
    assert_eq!(
        quotient_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(quotient_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_arg_returns_value_error() {
    assert_eq!(
        quotient_fn(&[Value::Text("abc".to_string()), Value::Number(2.0)]),
        Value::Error(ErrorKind::Value)
    );
}
