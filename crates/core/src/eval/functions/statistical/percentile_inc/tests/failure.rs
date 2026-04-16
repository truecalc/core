use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn percentile_inc_k_above_1_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(1.1)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_inc_k_negative_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(-0.1)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_inc_empty_array_returns_num_error() {
    let arr = Value::Array(vec![]);
    assert_eq!(percentile_inc_fn(&[arr, Value::Number(0.5)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_inc_non_number_k_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        percentile_inc_fn(&[arr, Value::Text("k".to_string())]),
        Value::Error(ErrorKind::Num)
    );
}
