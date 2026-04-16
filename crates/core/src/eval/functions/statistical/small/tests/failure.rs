use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn small_k0_returns_num_error() {
    // k=0 is invalid → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(small_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn small_k_exceeds_count_returns_num_error() {
    // k > n → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(small_fn(&[arr, Value::Number(3.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn small_empty_array_returns_num_error() {
    // No values → #NUM!
    let arr = Value::Array(vec![]);
    assert_eq!(small_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn small_fractional_k_returns_num_error() {
    // k=1.5 is not an integer → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(small_fn(&[arr, Value::Number(1.5)]), Value::Error(ErrorKind::Num));
}
