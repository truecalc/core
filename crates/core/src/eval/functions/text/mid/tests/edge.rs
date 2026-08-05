use super::super::*;
use crate::types::Value;

#[test]
fn start_beyond_end() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(6.0), Value::Number(3.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn zero_num_chars() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(2.0), Value::Number(0.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn clamp_beyond_length() {
    assert_eq!(
        mid_fn(&[Value::Text("Hello".to_string()), Value::Number(3.0), Value::Number(100.0)]),
        Value::Text("llo".to_string())
    );
}
