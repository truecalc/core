use super::super::*;
use crate::types::Value;

#[test]
fn zero_bytes() {
    assert_eq!(
        rightb_fn(&[Value::Text("Hello".to_string()), Value::Number(0.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn exceeds_length() {
    assert_eq!(
        rightb_fn(&[Value::Text("Hi".to_string()), Value::Number(10.0)]),
        Value::Text("Hi".to_string())
    );
}

#[test]
fn empty_string() {
    assert_eq!(
        rightb_fn(&[Value::Text("".to_string()), Value::Number(3.0)]),
        Value::Text("".to_string())
    );
}
