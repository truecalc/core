use super::super::*;
use crate::types::Value;

#[test]
fn empty_string() {
    assert_eq!(
        upper_fn(&[Value::Text("".to_string())]),
        Value::Text("".to_string())
    );
}

#[test]
fn numbers_unchanged() {
    assert_eq!(
        upper_fn(&[Value::Text("Hello123".to_string())]),
        Value::Text("HELLO123".to_string())
    );
}

#[test]
fn bool_coerced() {
    assert_eq!(
        upper_fn(&[Value::Bool(false)]),
        Value::Text("FALSE".to_string())
    );
}
