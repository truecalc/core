use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn large_k0_returns_num_error() {
    // k=0 is invalid → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(large_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn large_k_exceeds_count_returns_num_error() {
    // k > n → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(large_fn(&[arr, Value::Number(3.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn large_empty_array_returns_num_error() {
    // No values → #NUM!
    let arr = Value::Array(vec![]);
    assert_eq!(large_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn large_fractional_k_truncated() {
    // k=1.5 is truncated to 1 → 1st largest of [1.0, 2.0] = 2.0
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(large_fn(&[arr, Value::Number(1.5)]), Value::Number(2.0));
}
