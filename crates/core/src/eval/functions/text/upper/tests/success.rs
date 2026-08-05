use super::super::*;
use crate::types::Value;

#[test]
fn basic_upper() {
    assert_eq!(
        upper_fn(&[Value::Text("hello".to_string())]),
        Value::Text("HELLO".to_string())
    );
}

#[test]
fn mixed_case() {
    assert_eq!(
        upper_fn(&[Value::Text("HeLLo WoRLd".to_string())]),
        Value::Text("HELLO WORLD".to_string())
    );
}

#[test]
fn already_uppercase() {
    assert_eq!(
        upper_fn(&[Value::Text("HELLO".to_string())]),
        Value::Text("HELLO".to_string())
    );
}
