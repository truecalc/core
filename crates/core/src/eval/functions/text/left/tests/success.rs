use super::super::*;
use crate::types::Value;

#[test]
fn basic_left() {
    assert_eq!(
        left_fn(&[Value::Text("Hello".to_string()), Value::Number(3.0)]),
        Value::Text("Hel".to_string())
    );
}

#[test]
fn default_one_char() {
    assert_eq!(
        left_fn(&[Value::Text("Hello".to_string())]),
        Value::Text("H".to_string())
    );
}

#[test]
fn number_coerced_to_text() {
    assert_eq!(
        left_fn(&[Value::Number(123.0), Value::Number(2.0)]),
        Value::Text("12".to_string())
    );
}
