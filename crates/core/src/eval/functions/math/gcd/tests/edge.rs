use super::super::gcd_fn;
use crate::types::Value;

#[test]
fn gcd_with_zero() {
    // GCD(0, 5) = 5
    assert_eq!(gcd_fn(&[Value::Number(0.0), Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn gcd_both_zero() {
    // GCD(0, 0) = 0
    assert_eq!(gcd_fn(&[Value::Number(0.0), Value::Number(0.0)]), Value::Number(0.0));
}
