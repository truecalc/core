use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn ceiling_negative_both_negative_sig() {
    // CEILING(-4.5, -1) → -5
    assert_eq!(
        ceiling_fn(&[Value::Number(-4.5), Value::Number(-1.0)]),
        Value::Number(-5.0)
    );
}

#[test]
fn floor_negative_both_negative_sig() {
    // FLOOR(-4.5, -1) → -4
    assert_eq!(
        floor_fn(&[Value::Number(-4.5), Value::Number(-1.0)]),
        Value::Number(-4.0)
    );
}

#[test]
fn ceiling_significance_zero_returns_div_by_zero() {
    assert_eq!(
        ceiling_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn floor_significance_zero_returns_div_by_zero() {
    assert_eq!(
        floor_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}
