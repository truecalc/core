use super::super::*;
use crate::types::Value;

#[test]
fn mod_positive_n_negative_divisor() {
    // MOD(7, -3) → -2 (sign follows divisor)
    assert_eq!(
        mod_fn(&[Value::Number(7.0), Value::Number(-3.0)]),
        Value::Number(-2.0)
    );
}

#[test]
fn mod_both_negative() {
    // MOD(-7, -3) → -1
    assert_eq!(
        mod_fn(&[Value::Number(-7.0), Value::Number(-3.0)]),
        Value::Number(-1.0)
    );
}

#[test]
fn mod_fractional() {
    // MOD(5.5, 2.0) → 1.5
    assert_eq!(
        mod_fn(&[Value::Number(5.5), Value::Number(2.0)]),
        Value::Number(1.5)
    );
}
