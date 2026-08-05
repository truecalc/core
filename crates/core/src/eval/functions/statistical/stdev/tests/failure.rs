use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn stdev_no_args_returns_na() {
    assert_eq!(stdev_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn stdev_one_value_returns_div_zero() {
    assert_eq!(stdev_fn(&[Value::Number(5.0)]), Value::Error(ErrorKind::DivByZero));
}
