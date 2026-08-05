use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn negative_num_bytes() {
    assert_eq!(
        leftb_fn(&[Value::Text("Hello".to_string()), Value::Number(-1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(leftb_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    // one arg is valid: defaults to 1 byte (GS behaviour)
    assert_eq!(
        leftb_fn(&[Value::Text("Hello".to_string())]),
        Value::Text("H".to_string())
    );
}
