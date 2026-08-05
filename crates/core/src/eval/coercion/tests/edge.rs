use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn zero_number_to_bool_is_false() {
    assert_eq!(to_bool(Value::Number(0.0)), Ok(false));
}

#[test]
fn negative_number_to_bool_is_true() {
    assert_eq!(to_bool(Value::Number(-1.0)), Ok(true));
}

#[test]
fn array_to_number_fails() {
    assert_eq!(
        to_number(Value::Array(vec![])),
        Err(Value::Error(ErrorKind::Value))
    );
}

#[test]
fn array_to_string_val_fails() {
    assert_eq!(
        to_string_val(Value::Array(vec![])),
        Err(Value::Error(ErrorKind::Value))
    );
}

#[test]
fn array_to_bool_fails() {
    assert_eq!(
        to_bool(Value::Array(vec![])),
        Err(Value::Error(ErrorKind::Value))
    );
}
