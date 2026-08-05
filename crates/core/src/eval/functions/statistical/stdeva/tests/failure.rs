use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdeva_no_args_returns_na() {
    assert_eq!(stdeva_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdeva_single_value_returns_div_zero() {
    // Sample stddev: n < 2 → DivByZero
    assert_eq!(stdeva_fn(&[Value::Number(5.0)]), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn stdeva_only_empty_returns_div_zero() {
    // Empty is skipped; n < 2 → DivByZero
    assert_eq!(stdeva_fn(&[Value::Empty]), Value::Error(ErrorKind::DivByZero));
}
