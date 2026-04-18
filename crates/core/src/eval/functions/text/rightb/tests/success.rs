use super::super::*;
use crate::types::Value;

#[test]
fn basic_rightb() {
    assert_eq!(
        rightb_fn(&[Value::Text("Hello".to_string()), Value::Number(3.0)]),
        Value::Text("llo".to_string())
    );
}

#[test]
fn full_string() {
    assert_eq!(
        rightb_fn(&[Value::Text("Hello".to_string()), Value::Number(5.0)]),
        Value::Text("Hello".to_string())
    );
}
