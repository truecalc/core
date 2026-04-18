use super::super::*;
use crate::types::Value;

#[test]
fn ceiling_math_sig_zero_returns_zero() {
    assert_eq!(
        ceiling_math_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn floor_math_sig_zero_returns_zero() {
    assert_eq!(
        floor_math_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn ceiling_precise_sig_zero_returns_zero() {
    assert_eq!(
        ceiling_precise_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn floor_precise_sig_zero_returns_zero() {
    assert_eq!(
        floor_precise_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn ceiling_math_fractional_sig() {
    // 3.1 with sig=0.5 -> 3.5
    assert_eq!(
        ceiling_math_fn(&[Value::Number(3.1), Value::Number(0.5)]),
        Value::Number(3.5)
    );
}

#[test]
fn floor_math_fractional_sig() {
    // 3.9 with sig=0.5 -> 3.5
    assert_eq!(
        floor_math_fn(&[Value::Number(3.9), Value::Number(0.5)]),
        Value::Number(3.5)
    );
}

#[test]
fn ceiling_math_negative_sig_ignored() {
    // negative significance is converted to abs
    assert_eq!(
        ceiling_math_fn(&[Value::Number(5.5), Value::Number(-2.0)]),
        Value::Number(6.0)
    );
}

#[test]
fn ceiling_precise_negative_small() {
    // -0.1 rounds up to 0
    assert_eq!(
        ceiling_precise_fn(&[Value::Number(-0.1), Value::Number(1.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn floor_precise_negative_small() {
    // -0.1 rounds down to -1
    assert_eq!(
        floor_precise_fn(&[Value::Number(-0.1)]),
        Value::Number(-1.0)
    );
}

#[test]
fn ceiling_math_negative_sig2_mode0() {
    // -4.1 with sig=2, mode=0 -> -4
    assert_eq!(
        ceiling_math_fn(&[Value::Number(-4.1), Value::Number(2.0), Value::Number(0.0)]),
        Value::Number(-4.0)
    );
}

#[test]
fn floor_math_negative_sig2_mode0() {
    // -4.1 with sig=2, mode=0 -> -6
    assert_eq!(
        floor_math_fn(&[Value::Number(-4.1), Value::Number(2.0), Value::Number(0.0)]),
        Value::Number(-6.0)
    );
}

#[test]
fn iso_ceiling_sig_zero_returns_zero() {
    assert_eq!(
        iso_ceiling_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}
