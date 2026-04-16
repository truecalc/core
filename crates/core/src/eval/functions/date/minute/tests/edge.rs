use super::super::*;
use crate::types::Value;

#[test]
fn serial_zero_zero_minutes() {
    assert_eq!(minute_fn(&[Value::Number(0.0)]), Value::Number(0.0));
}

#[test]
fn serial_half_zero_minutes() {
    // oracle: MINUTE(0.5) = 0 (noon has 0 minutes)
    assert_eq!(minute_fn(&[Value::Number(0.5)]), Value::Number(0.0));
}

#[test]
fn integer_serial_zero_minutes() {
    assert_eq!(minute_fn(&[Value::Number(45306.0)]), Value::Number(0.0));
}
