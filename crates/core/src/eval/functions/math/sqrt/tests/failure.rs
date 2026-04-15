use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn sqrt_negative_returns_num_error() {
    assert_eq!(
        sqrt_fn(&[Value::Number(-1.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(sqrt_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_text_returns_value_error() {
    assert_eq!(
        sqrt_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}
