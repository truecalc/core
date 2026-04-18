use super::super::*;
use crate::types::Value;

#[test]
fn serial_zero_zero_seconds() {
    // oracle: SECOND(0) = 0
    assert_eq!(second_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn serial_half_zero_seconds() {
    // noon serial: no seconds
    assert_eq!(second_fn(&[Value::Number(0.5)]), Value::Number(0.0));
}

#[test]
fn integer_serial_zero_seconds() {
    assert_eq!(second_fn(&[Value::Number(45306.0)]), Value::Number(0.0));
}
