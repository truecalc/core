use super::super::*;
use crate::types::Value;

#[test]
fn replace_world_with_earth() {
    assert_eq!(
        replaceb_fn(&[
            Value::Text("Hello World".to_string()),
            Value::Number(7.0),
            Value::Number(5.0),
            Value::Text("Earth".to_string())
        ]),
        Value::Text("Hello Earth".to_string())
    );
}

#[test]
fn replace_single_byte() {
    assert_eq!(
        replaceb_fn(&[
            Value::Text("abc".to_string()),
            Value::Number(2.0),
            Value::Number(1.0),
            Value::Text("X".to_string())
        ]),
        Value::Text("aXc".to_string())
    );
}

#[test]
fn replace_first_byte() {
    assert_eq!(
        replaceb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Text("J".to_string())
        ]),
        Value::Text("Jello".to_string())
    );
}
