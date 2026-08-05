use super::super::*;
use crate::types::Value;

#[test]
fn empty_strings() {
    assert_eq!(
        concatenate_fn(&[Value::Text("".to_string()), Value::Text("".to_string())]),
        Value::Text("".to_string())
    );
}

#[test]
fn bool_coerced() {
    assert_eq!(
        concatenate_fn(&[Value::Bool(true), Value::Text("!".to_string())]),
        Value::Text("TRUE!".to_string())
    );
}

#[test]
fn empty_value_coerced() {
    assert_eq!(
        concatenate_fn(&[Value::Text("a".to_string()), Value::Empty, Value::Text("b".to_string())]),
        Value::Text("ab".to_string())
    );
}
