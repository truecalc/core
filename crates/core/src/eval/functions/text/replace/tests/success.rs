use super::super::*;
use crate::types::Value;

#[test]
fn basic_replace() {
    assert_eq!(
        replace_fn(&[
            Value::Text("Hello World".to_string()),
            Value::Number(7.0),
            Value::Number(5.0),
            Value::Text("Rust".to_string()),
        ]),
        Value::Text("Hello Rust".to_string())
    );
}

#[test]
fn replace_at_start() {
    assert_eq!(
        replace_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Text("XY".to_string()),
        ]),
        Value::Text("XYllo".to_string())
    );
}

#[test]
fn insert_without_removing() {
    assert_eq!(
        replace_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(3.0),
            Value::Number(0.0),
            Value::Text("XX".to_string()),
        ]),
        Value::Text("HeXXllo".to_string())
    );
}
