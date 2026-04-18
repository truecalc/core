use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdev_s_no_args_returns_na() {
    assert_eq!(stdev_s_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdev_s_one_value_returns_div_zero() {
    assert_eq!(stdev_s_fn(&[Value::Number(5.0)]), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn stdev_s_no_numeric_values_returns_div_zero() {
    assert_eq!(
        stdev_s_fn(&[Value::Text("a".to_string()), Value::Bool(false)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn stdev_s_empty_only_returns_div_zero() {
    assert_eq!(stdev_s_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
