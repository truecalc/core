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

#[test]
fn occurrence_zero_replaces_all() {
    // GS: instance_num=0 means replace all occurrences (same as omitting)
    assert_eq!(
        substitute_fn(&[
            Value::Text("hello".to_string()),
            Value::Text("l".to_string()),
            Value::Text("r".to_string()),
            Value::Number(0.0),
        ]),
        Value::Text("herro".to_string())
    );
}
