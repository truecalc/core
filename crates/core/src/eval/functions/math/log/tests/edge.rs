use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn log_of_one_is_zero() {
    assert_eq!(log_fn(&[Value::Number(1.0)]), Value::Number(0.0));
}

#[test]
fn log_base_one_returns_div_by_zero() {
    // log base 1: ln(b)=0, division by zero → #DIV/0! (GS behavior)
    assert_eq!(
        log_fn(&[Value::Number(10.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn ln_of_one_is_zero() {
    assert_eq!(ln_fn(&[Value::Number(1.0)]), Value::Number(0.0));
}
