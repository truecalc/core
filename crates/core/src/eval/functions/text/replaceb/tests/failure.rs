use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(replaceb_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn negative_num_bytes() {
    assert_eq!(
        replaceb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(1.0),
            Value::Number(-1.0),
            Value::Text("X".to_string())
        ]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn start_below_one() {
    assert_eq!(
        replaceb_fn(&[
            Value::Text("Hello".to_string()),
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Text("X".to_string())
        ]),
        Value::Error(ErrorKind::Value)
    );
}
