use super::super::*;
use crate::types::Value;

#[test]
fn power_basic() {
    assert_eq!(
        power_fn(&[Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(8.0)
    );
}

#[test]
fn power_of_one() {
    assert_eq!(
        power_fn(&[Value::Number(5.0), Value::Number(1.0)]),
        Value::Number(5.0)
    );
}

#[test]
fn power_zero_exponent() {
    assert_eq!(
        power_fn(&[Value::Number(7.0), Value::Number(0.0)]),
        Value::Number(1.0)
    );
}
