use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn percentrank_x_below_min_returns_na() {
    // x=0 < min(1) → #N/A
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn percentrank_x_above_max_returns_na() {
    // x=6 > max(5) → #N/A
    let arr = Value::Array(vec![
        Value::Number(1.0), Value::Number(2.0), Value::Number(3.0),
        Value::Number(4.0), Value::Number(5.0),
    ]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(6.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn percentrank_empty_array_returns_na() {
    let arr = Value::Array(vec![]);
    assert_eq!(percentrank_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn percentrank_non_number_x_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        percentrank_fn(&[arr, Value::Text("x".to_string())]),
        Value::Error(ErrorKind::NA)
    );
}
