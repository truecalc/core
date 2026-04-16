use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn large_ignores_text_and_bool_in_array() {
    // Only numbers counted; text/bool ignored
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Text("abc".to_string()),
        Value::Bool(true),
        Value::Number(5.0),
    ]);
    // LARGE([10, "abc", true, 5], 1) = 10
    assert_eq!(large_fn(&[arr, Value::Number(1.0)]), Value::Number(10.0));
}

#[test]
fn large_duplicate_values() {
    // LARGE([3,3,3], 2) = 3
    let arr = Value::Array(vec![
        Value::Number(3.0), Value::Number(3.0), Value::Number(3.0),
    ]);
    assert_eq!(large_fn(&[arr, Value::Number(2.0)]), Value::Number(3.0));
}

#[test]
fn large_negative_values() {
    // LARGE([-5,-3,-1], 1) = -1
    let arr = Value::Array(vec![
        Value::Number(-5.0), Value::Number(-3.0), Value::Number(-1.0),
    ]);
    assert_eq!(large_fn(&[arr, Value::Number(1.0)]), Value::Number(-1.0));
}

#[test]
fn large_negative_k_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(large_fn(&[arr, Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}
