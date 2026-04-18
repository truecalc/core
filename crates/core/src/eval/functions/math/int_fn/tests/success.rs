use super::super::*;
use crate::types::Value;

#[test]
fn int_positive_fraction() {
    assert_eq!(int_fn(&[Value::Number(3.7)]), Value::Number(3.0));
}

#[test]
fn int_positive_exact() {
    assert_eq!(int_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn int_negative_fraction() {
    // INT(-1.5) → -2 (floor toward negative infinity)
    assert_eq!(int_fn(&[Value::Number(-1.5)]), Value::Number(-2.0));
}
