use super::super::*;
use crate::types::Value;

#[test]
fn two_strings() {
    assert_eq!(
        concatenate_fn(&[Value::Text("Hello".to_string()), Value::Text(" World".to_string())]),
        Value::Text("Hello World".to_string())
    );
}

#[test]
fn single_string() {
    assert_eq!(
        concatenate_fn(&[Value::Text("Hello".to_string())]),
        Value::Text("Hello".to_string())
    );
}

#[test]
fn number_coerced() {
    assert_eq!(
        concatenate_fn(&[Value::Text("Value: ".to_string()), Value::Number(42.0)]),
        Value::Text("Value: 42".to_string())
    );
}
