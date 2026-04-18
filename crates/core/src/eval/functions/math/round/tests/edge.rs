use super::super::*;
use crate::types::Value;

#[test]
fn round_negative_digits_rounds_to_tens() {
    // ROUND(1234, -2) → 1200
    assert_eq!(
        round_fn(&[Value::Number(1234.0), Value::Number(-2.0)]),
        Value::Number(1200.0)
    );
}

#[test]
fn round_negative_half_away() {
    // ROUND(-2.5, 0) → -3 (away from zero)
    assert_eq!(
        round_fn(&[Value::Number(-2.5), Value::Number(0.0)]),
        Value::Number(-3.0)
    );
}

#[test]
fn roundup_already_integer() {
    // ROUNDUP(3.0, 0) → 3 (no change needed)
    assert_eq!(
        roundup_fn(&[Value::Number(3.0), Value::Number(0.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn rounddown_fractional_negative_digits() {
    // ROUNDDOWN(1567, -2) → 1500
    assert_eq!(
        rounddown_fn(&[Value::Number(1567.0), Value::Number(-2.0)]),
        Value::Number(1500.0)
    );
}
