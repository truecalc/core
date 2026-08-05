use super::super::*;
use crate::types::Value;

#[test]
fn old_not_found() {
    assert_eq!(
        substitute_fn(&[
            Value::Text("Hello".to_string()),
            Value::Text("xyz".to_string()),
            Value::Text("abc".to_string()),
        ]),
        Value::Text("Hello".to_string())
    );
}

#[test]
fn instance_beyond_count() {
    // Only 2 occurrences, instance 3 — no replacement
    assert_eq!(
        substitute_fn(&[
            Value::Text("aa".to_string()),
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
            Value::Number(3.0),
        ]),
        Value::Text("aa".to_string())
    );
}

#[test]
fn empty_old_text_no_change() {
    assert_eq!(
        substitute_fn(&[
            Value::Text("Hello".to_string()),
            Value::Text("".to_string()),
            Value::Text("x".to_string()),
        ]),
        Value::Text("Hello".to_string())
    );
}
