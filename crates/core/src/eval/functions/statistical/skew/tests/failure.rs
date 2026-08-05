use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn skew_less_than_3_returns_div0() {
    assert_eq!(
        skew_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn skew_all_same_returns_div0() {
    assert_eq!(
        skew_fn(&[Value::Number(5.0), Value::Number(5.0), Value::Number(5.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}
