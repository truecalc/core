use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn percentile_exc_k_negative_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_exc_fn(&[arr, Value::Number(-0.1)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_exc_k_above_1_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_exc_fn(&[arr, Value::Number(1.1)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_exc_interpolation_two_elements() {
    // PERCENTILE.EXC([1,3], 0.5): idx=0.5*3-1=0.5, lo=0(1.0), hi=1(3.0), frac=0.5 → 2.0
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(percentile_exc_fn(&[arr, Value::Number(0.5)]), Value::Number(2.0));
}
