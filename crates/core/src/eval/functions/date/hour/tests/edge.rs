use super::super::*;
use crate::types::Value;

#[test]
fn serial_zero_midnight() {
    // oracle: HOUR(0) = 0
    assert_eq!(hour_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn serial_half_noon() {
    // oracle: HOUR(0.5) = 12
    assert_eq!(hour_fn(&[Value::Number(0.5)]), Value::Number(12.0));
}

#[test]
fn integer_serial_midnight() {
    // Integer serials have zero fractional part → hour 0
    assert_eq!(hour_fn(&[Value::Number(45306.0)]), Value::Number(0.0));
}
