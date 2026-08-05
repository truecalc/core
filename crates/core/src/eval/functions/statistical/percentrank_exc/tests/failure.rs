use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn percentrank_exc_x_below_min_returns_na() {
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn percentrank_exc_x_above_max_returns_na() {
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(4.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn percentrank_exc_empty_array_returns_na() {
    let arr = Value::Array(vec![]);
    assert_eq!(percentrank_exc_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn percentrank_exc_non_number_x_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        percentrank_exc_fn(&[arr, Value::Text("x".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}
