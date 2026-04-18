use super::super::*;
use crate::types::Value;

#[test]
fn zero_chars() {
    assert_eq!(
        left_fn(&[Value::Text("Hello".to_string()), Value::Number(0.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn clamp_beyond_length() {
    assert_eq!(
        left_fn(&[Value::Text("Hello".to_string()), Value::Number(100.0)]),
        Value::Text("Hello".to_string())
    );
}

#[test]
fn empty_string() {
    assert_eq!(
        left_fn(&[Value::Text("".to_string()), Value::Number(3.0)]),
        Value::Text("".to_string())
    );
}
