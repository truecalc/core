use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn small_ignores_text_and_bool_in_array() {
    // Only numbers counted; text/bool ignored
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Text("abc".to_string()),
        Value::Bool(false),
        Value::Number(5.0),
    ]);
    // SMALL([10, "abc", false, 5], 1) = 5
    assert_eq!(small_fn(&[arr, Value::Number(1.0)]), Value::Number(5.0));
}

#[test]
fn small_duplicate_values() {
    // SMALL([3,3,3], 2) = 3
    let arr = Value::Array(vec![
        Value::Number(3.0), Value::Number(3.0), Value::Number(3.0),
    ]);
    assert_eq!(small_fn(&[arr, Value::Number(2.0)]), Value::Number(3.0));
}

#[test]
fn small_negative_values() {
    // SMALL([-5,-3,-1], 1) = -5
    let arr = Value::Array(vec![
        Value::Number(-5.0), Value::Number(-3.0), Value::Number(-1.0),
    ]);
    assert_eq!(small_fn(&[arr, Value::Number(1.0)]), Value::Number(-5.0));
}

#[test]
fn small_negative_k_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(small_fn(&[arr, Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}
