use super::super::*;
use crate::types::Value;

#[test]
fn empty_string() {
    assert_eq!(
        lower_fn(&[Value::Text("".to_string())]),
        Value::Text("".to_string())
    );
}

#[test]
fn numbers_unchanged() {
    assert_eq!(
        lower_fn(&[Value::Text("Hello123".to_string())]),
        Value::Text("hello123".to_string())
    );
}

#[test]
fn bool_coerced() {
    assert_eq!(
        lower_fn(&[Value::Bool(true)]),
        Value::Text("true".to_string())
    );
}
