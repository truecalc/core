use super::super::*;
use crate::types::Value;

#[test]
fn int_zero() {
    assert_eq!(int_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn int_negative_exact() {
    assert_eq!(int_fn(&[Value::Number(-3.0)]), Value::Number(-3.0));
}

#[test]
fn int_very_small_positive() {
    assert_eq!(int_fn(&[Value::Number(0.9999)]), Value::Number(0.0));
}
