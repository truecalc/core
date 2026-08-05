use super::super::*;
use crate::types::Value;

#[test]
fn basic_lower() {
    assert_eq!(
        lower_fn(&[Value::Text("HELLO".to_string())]),
        Value::Text("hello".to_string())
    );
}

#[test]
fn mixed_case() {
    assert_eq!(
        lower_fn(&[Value::Text("HeLLo WoRLd".to_string())]),
        Value::Text("hello world".to_string())
    );
}

#[test]
fn already_lowercase() {
    assert_eq!(
        lower_fn(&[Value::Text("hello".to_string())]),
        Value::Text("hello".to_string())
    );
}
