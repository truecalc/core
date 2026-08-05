use super::super::*;
use crate::types::Value;

#[test]
fn round_to_two_decimals() {
    assert_eq!(
        round_fn(&[Value::Number(3.14159), Value::Number(2.0)]),
        Value::Number(3.14)
    );
}

#[test]
fn roundup_basic() {
    assert_eq!(
        roundup_fn(&[Value::Number(3.1), Value::Number(0.0)]),
        Value::Number(4.0)
    );
}

#[test]
fn rounddown_basic() {
    assert_eq!(
        rounddown_fn(&[Value::Number(3.9), Value::Number(0.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn round_half_point_five_away_from_zero() {
    assert_eq!(
        round_fn(&[Value::Number(2.5), Value::Number(0.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn roundup_negative() {
    // away from zero → more negative
    assert_eq!(
        roundup_fn(&[Value::Number(-3.1), Value::Number(0.0)]),
        Value::Number(-4.0)
    );
}

#[test]
fn rounddown_negative() {
    // toward zero → less negative
    assert_eq!(
        rounddown_fn(&[Value::Number(-3.9), Value::Number(0.0)]),
        Value::Number(-3.0)
    );
}
