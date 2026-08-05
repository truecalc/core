use super::super::*;
use crate::types::Value;

#[test]
fn abs_positive() {
    assert_eq!(abs_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn abs_negative() {
    assert_eq!(abs_fn(&[Value::Number(-5.0)]), Value::Number(5.0));
}

#[test]
fn abs_zero() {
    assert_eq!(abs_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}
