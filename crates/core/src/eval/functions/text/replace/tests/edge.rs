use super::super::*;
use crate::types::Value;

#[test]
fn start_beyond_end() {
    assert_eq!(
        replace_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(10.0),
            Value::Number(2.0),
            Value::Text("X".to_string()),
        ]),
        Value::Text("HelloX".to_string())
    );
}

#[test]
fn clamp_num_chars_beyond_end() {
    assert_eq!(
        replace_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(3.0),
            Value::Number(100.0),
            Value::Text("X".to_string()),
        ]),
        Value::Text("HeX".to_string())
    );
}

#[test]
fn empty_new_text_deletes() {
    assert_eq!(
        replace_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Text("".to_string()),
        ]),
        Value::Text("Ho".to_string())
    );
}
