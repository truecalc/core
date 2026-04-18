use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn covariance_s_no_args_returns_na() {
    assert_eq!(covariance_s_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_s_one_arg_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(covariance_s_fn(&[arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_s_three_args_returns_na() {
    let arr1 = Value::Array(vec![Value::Number(1.0)]);
    let arr2 = Value::Array(vec![Value::Number(2.0)]);
    let arr3 = Value::Array(vec![Value::Number(3.0)]);
    assert_eq!(covariance_s_fn(&[arr1, arr2, arr3]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_s_unequal_lengths_returns_na() {
    let arr1 = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let arr2 = Value::Array(vec![Value::Number(3.0)]);
    assert_eq!(covariance_s_fn(&[arr1, arr2]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_s_single_point_returns_num() {
    // n < 2 → Num (Google Sheets oracle)
    let arr1 = Value::Array(vec![Value::Number(5.0)]);
    let arr2 = Value::Array(vec![Value::Number(10.0)]);
    assert_eq!(covariance_s_fn(&[arr1, arr2]), Value::Error(ErrorKind::Num));
}

#[test]
fn covariance_s_empty_arrays_returns_num() {
    let arr1 = Value::Array(vec![]);
    let arr2 = Value::Array(vec![]);
    assert_eq!(covariance_s_fn(&[arr1, arr2]), Value::Error(ErrorKind::Num));
}
