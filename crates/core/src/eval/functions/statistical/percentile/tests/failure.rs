use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn percentile_k_above_1_returns_num_error() {
    // k=1.1 → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_fn(&[arr, Value::Number(1.1)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_k_negative_returns_num_error() {
    // k=-0.1 → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_fn(&[arr, Value::Number(-0.1)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_empty_array_returns_num_error() {
    let arr = Value::Array(vec![]);
    assert_eq!(percentile_fn(&[arr, Value::Number(0.5)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_non_number_k_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        percentile_fn(&[arr, Value::Text("x".to_string())]),
        Value::Error(ErrorKind::Num)
    );
}
