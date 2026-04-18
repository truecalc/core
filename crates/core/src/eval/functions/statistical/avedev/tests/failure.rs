use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn avedev_no_args_returns_na() {
    assert_eq!(avedev_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn avedev_no_numeric_values_returns_div_zero() {
    // Only text/bool — no numeric values
    assert_eq!(
        avedev_fn(&[Value::Text("a".to_string()), Value::Bool(false)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn avedev_empty_only_returns_div_zero() {
    assert_eq!(
        avedev_fn(&[Value::Empty]),
        Value::Error(ErrorKind::DivByZero)
    );
}
