use super::super::*;
use crate::types::Value;

#[test]
fn covariance_p_basic() {
    // xs=[1,2,3], ys=[4,5,6]
    // mean_x=2, mean_y=5
    // cov = ((1-2)(4-5) + (2-2)(5-5) + (3-2)(6-5)) / 3 = (1+0+1)/3 = 2/3
    let arr1 = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    let arr2 = Value::Array(vec![Value::Number(4.0), Value::Number(5.0), Value::Number(6.0)]);
    let result = covariance_p_fn(&[arr1, arr2]);
    if let Value::Number(v) = result {
        assert!((v - 2.0 / 3.0).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn covariance_p_two_points() {
    // xs=[1,3], ys=[2,4], mean_x=2, mean_y=3
    // cov = ((1-2)(2-3) + (3-2)(4-3)) / 2 = (1+1)/2 = 1.0
    let arr1 = Value::Array(vec![Value::Number(1.0), Value::Number(3.0)]);
    let arr2 = Value::Array(vec![Value::Number(2.0), Value::Number(4.0)]);
    assert_eq!(covariance_p_fn(&[arr1, arr2]), Value::Number(1.0));
}

#[test]
fn covariance_p_negative_covariance() {
    // xs=[1,3], ys=[4,2], mean_x=2, mean_y=3
    // cov = ((1-2)(4-3) + (3-2)(2-3)) / 2 = (-1-1)/2 = -1.0
    let arr1 = Value::Array(vec![Value::Number(1.0), Value::Number(3.0)]);
    let arr2 = Value::Array(vec![Value::Number(4.0), Value::Number(2.0)]);
    assert_eq!(covariance_p_fn(&[arr1, arr2]), Value::Number(-1.0));
}
