use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn skew_p_no_values_returns_div0() {
    assert_eq!(skew_p_fn(&[]), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn skew_p_all_same_returns_div0() {
    assert_eq!(
        skew_p_fn(&[Value::Number(5.0), Value::Number(5.0), Value::Number(5.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}
