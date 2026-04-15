use super::super::*;
use crate::types::Value;

#[test]
fn quotient_both_negative() {
    assert_eq!(
        quotient_fn(&[Value::Number(-7.0), Value::Number(-2.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn quotient_less_than_one() {
    assert_eq!(
        quotient_fn(&[Value::Number(1.0), Value::Number(5.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn quotient_fractional_numerator() {
    // 7.9 / 2 → trunc(3.95) = 3
    assert_eq!(
        quotient_fn(&[Value::Number(7.9), Value::Number(2.0)]),
        Value::Number(3.0)
    );
}
