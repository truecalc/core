use super::super::*;
use crate::types::Value;

// CEILING.MATH tests
#[test]
fn ceiling_math_basic() {
    assert_eq!(
        ceiling_math_fn(&[Value::Number(5.5)]),
        Value::Number(6.0)
    );
}

#[test]
fn ceiling_math_with_sig() {
    assert_eq!(
        ceiling_math_fn(&[Value::Number(5.5), Value::Number(2.0)]),
        Value::Number(6.0)
    );
}

#[test]
fn ceiling_math_exact_multiple() {
    assert_eq!(
        ceiling_math_fn(&[Value::Number(4.0), Value::Number(2.0)]),
        Value::Number(4.0)
    );
}

#[test]
fn ceiling_math_zero() {
    assert_eq!(
        ceiling_math_fn(&[Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn ceiling_math_negative_mode0() {
    // mode=0: negative rounds toward zero (positive infinity)
    assert_eq!(
        ceiling_math_fn(&[Value::Number(-5.5), Value::Number(1.0), Value::Number(0.0)]),
        Value::Number(-5.0)
    );
}

#[test]
fn ceiling_math_negative_mode1() {
    // mode≠0: negative rounds away from zero (negative infinity)
    assert_eq!(
        ceiling_math_fn(&[Value::Number(-5.5), Value::Number(1.0), Value::Number(1.0)]),
        Value::Number(-6.0)
    );
}

// CEILING.PRECISE tests
#[test]
fn ceiling_precise_basic() {
    assert_eq!(
        ceiling_precise_fn(&[Value::Number(5.5)]),
        Value::Number(6.0)
    );
}

#[test]
fn ceiling_precise_negative() {
    // rounds toward zero
    assert_eq!(
        ceiling_precise_fn(&[Value::Number(-5.5)]),
        Value::Number(-5.0)
    );
}

#[test]
fn ceiling_precise_neg_sig_ignored() {
    // sign of significance is ignored
    assert_eq!(
        ceiling_precise_fn(&[Value::Number(5.5), Value::Number(-2.0)]),
        Value::Number(6.0)
    );
}

// FLOOR.MATH tests
#[test]
fn floor_math_basic() {
    assert_eq!(
        floor_math_fn(&[Value::Number(5.5)]),
        Value::Number(5.0)
    );
}

#[test]
fn floor_math_negative_mode0() {
    // mode=0: negative rounds toward negative infinity
    assert_eq!(
        floor_math_fn(&[Value::Number(-5.5), Value::Number(1.0), Value::Number(0.0)]),
        Value::Number(-6.0)
    );
}

#[test]
fn floor_math_negative_mode1() {
    // mode≠0: negative rounds toward zero
    assert_eq!(
        floor_math_fn(&[Value::Number(-5.5), Value::Number(1.0), Value::Number(1.0)]),
        Value::Number(-5.0)
    );
}

// FLOOR.PRECISE tests
#[test]
fn floor_precise_basic() {
    assert_eq!(
        floor_precise_fn(&[Value::Number(5.5)]),
        Value::Number(5.0)
    );
}

#[test]
fn floor_precise_negative() {
    // rounds toward negative infinity
    assert_eq!(
        floor_precise_fn(&[Value::Number(-5.5)]),
        Value::Number(-6.0)
    );
}

#[test]
fn floor_precise_neg_sig_ignored() {
    // sign of significance is ignored
    assert_eq!(
        floor_precise_fn(&[Value::Number(5.5), Value::Number(-2.0)]),
        Value::Number(4.0)
    );
}

// ISO.CEILING tests
#[test]
fn iso_ceiling_basic() {
    assert_eq!(
        iso_ceiling_fn(&[Value::Number(5.5)]),
        Value::Number(6.0)
    );
}

#[test]
fn iso_ceiling_negative() {
    assert_eq!(
        iso_ceiling_fn(&[Value::Number(-5.5)]),
        Value::Number(-5.0)
    );
}

#[test]
fn iso_ceiling_neg_sig_ignored() {
    assert_eq!(
        iso_ceiling_fn(&[Value::Number(-5.5), Value::Number(2.0)]),
        Value::Number(-4.0)
    );
}
