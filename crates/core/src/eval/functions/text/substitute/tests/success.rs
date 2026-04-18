use super::super::*;
use crate::types::Value;

#[test]
fn replace_all() {
    assert_eq!(
        substitute_fn(&[
            Value::Text("aaa".to_string()),
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ]),
        Value::Text("bbb".to_string())
    );
}

#[test]
fn replace_specific_instance() {
    assert_eq!(
        substitute_fn(&[
            Value::Text("aaa".to_string()),
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
            Value::Number(2.0),
        ]),
        Value::Text("aba".to_string())
    );
}

#[test]
fn replace_word() {
    assert_eq!(
        substitute_fn(&[
            Value::Text("Hello World".to_string()),
            Value::Text("World".to_string()),
            Value::Text("Rust".to_string()),
        ]),
        Value::Text("Hello Rust".to_string())
    );
}
