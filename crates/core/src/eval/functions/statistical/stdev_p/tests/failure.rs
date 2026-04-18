use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdev_p_no_args_returns_na() {
    assert_eq!(stdev_p_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdev_p_no_numeric_values_returns_div_zero() {
    assert_eq!(
        stdev_p_fn(&[Value::Text("a".to_string()), Value::Bool(false)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn stdev_p_empty_only_returns_div_zero() {
    assert_eq!(stdev_p_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
