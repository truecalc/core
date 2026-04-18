use super::super::*;
use crate::types::Value;

#[test]
fn covariance_s_all_same_x_returns_zero() {
    let arr1 = Value::Array(vec![Value::Number(3.0), Value::Number(3.0), Value::Number(3.0)]);
    let arr2 = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    assert_eq!(covariance_s_fn(&[arr1, arr2]), Value::Number(0.0));
}

#[test]
fn covariance_s_all_same_both_returns_zero() {
    let arr1 = Value::Array(vec![Value::Number(4.0), Value::Number(4.0)]);
    let arr2 = Value::Array(vec![Value::Number(7.0), Value::Number(7.0)]);
    assert_eq!(covariance_s_fn(&[arr1, arr2]), Value::Number(0.0));
}

#[test]
fn covariance_s_plain_number_args_single_each_returns_num() {
    // Two plain numbers → n=1 → Num (Google Sheets oracle)
    assert_eq!(
        covariance_s_fn(&[Value::Number(5.0), Value::Number(10.0)]),
        Value::Error(crate::types::ErrorKind::Num)
    );
}
