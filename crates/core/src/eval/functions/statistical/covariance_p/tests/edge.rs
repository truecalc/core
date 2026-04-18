use super::super::*;
use crate::types::Value;

#[test]
fn covariance_p_single_point_returns_zero() {
    // Single point: cov = ((x-mean_x)(y-mean_y))/1 = 0
    let arr1 = Value::Array(vec![Value::Number(5.0)]);
    let arr2 = Value::Array(vec![Value::Number(10.0)]);
    assert_eq!(covariance_p_fn(&[arr1, arr2]), Value::Number(0.0));
}

#[test]
fn covariance_p_all_same_values_returns_zero() {
    let arr1 = Value::Array(vec![Value::Number(3.0), Value::Number(3.0), Value::Number(3.0)]);
    let arr2 = Value::Array(vec![Value::Number(7.0), Value::Number(7.0), Value::Number(7.0)]);
    assert_eq!(covariance_p_fn(&[arr1, arr2]), Value::Number(0.0));
}

#[test]
fn covariance_p_plain_number_args() {
    // Two plain numbers — collect_nums treats them as single-element collections
    assert_eq!(covariance_p_fn(&[Value::Number(5.0), Value::Number(10.0)]), Value::Number(0.0));
}
