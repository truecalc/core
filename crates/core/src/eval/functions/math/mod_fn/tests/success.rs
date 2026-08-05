use super::super::*;
use crate::types::Value;

#[test]
fn mod_basic() {
    assert_eq!(
        mod_fn(&[Value::Number(7.0), Value::Number(3.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn mod_exact_divisible() {
    assert_eq!(
        mod_fn(&[Value::Number(6.0), Value::Number(3.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn mod_negative_n_positive_divisor() {
    // MOD(-7, 3) → 2 (sign follows divisor)
    assert_eq!(
        mod_fn(&[Value::Number(-7.0), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}
