use super::super::lcm_fn;
use crate::types::Value;

#[test]
fn lcm_with_zero() {
    // LCM(0, 5) = 0
    assert_eq!(lcm_fn(&[Value::Number(0.0), Value::Number(5.0)]), Value::Number(0.0));
}

#[test]
fn lcm_both_zero() {
    // LCM(0, 0) = 0
    assert_eq!(lcm_fn(&[Value::Number(0.0), Value::Number(0.0)]), Value::Number(0.0));
}
