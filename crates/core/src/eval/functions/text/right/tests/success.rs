use super::super::*;
use crate::types::Value;

#[test]
fn basic_right() {
    assert_eq!(
        right_fn(&[Value::Text("Hello".to_string()), Value::Number(3.0)]),
        Value::Text("llo".to_string())
    );
}

#[test]
fn default_one_char() {
    assert_eq!(
        right_fn(&[Value::Text("Hello".to_string())]),
        Value::Text("o".to_string())
    );
}

#[test]
fn number_coerced_to_text() {
    assert_eq!(
        right_fn(&[Value::Number(123.0), Value::Number(2.0)]),
        Value::Text("23".to_string())
    );
}
