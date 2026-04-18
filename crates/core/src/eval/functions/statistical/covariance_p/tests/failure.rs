use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn covariance_p_no_args_returns_na() {
    assert_eq!(covariance_p_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_p_one_arg_returns_na() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(covariance_p_fn(&[arr]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_p_three_args_returns_na() {
    let arr1 = Value::Array(vec![Value::Number(1.0)]);
    let arr2 = Value::Array(vec![Value::Number(2.0)]);
    let arr3 = Value::Array(vec![Value::Number(3.0)]);
    assert_eq!(covariance_p_fn(&[arr1, arr2, arr3]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_p_unequal_lengths_returns_na() {
    let arr1 = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let arr2 = Value::Array(vec![Value::Number(3.0)]);
    assert_eq!(covariance_p_fn(&[arr1, arr2]), Value::Error(ErrorKind::NA));
}

#[test]
fn covariance_p_empty_arrays_returns_div_zero() {
    let arr1 = Value::Array(vec![]);
    let arr2 = Value::Array(vec![]);
    assert_eq!(covariance_p_fn(&[arr1, arr2]), Value::Error(ErrorKind::DivByZero));
}
