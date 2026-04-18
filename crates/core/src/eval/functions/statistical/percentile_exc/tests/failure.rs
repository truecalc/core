use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn percentile_exc_k_zero_returns_num_error() {
    // k=0 not allowed for EXC → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_exc_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_exc_k_one_returns_num_error() {
    // k=1 not allowed for EXC → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(percentile_exc_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_exc_empty_array_returns_num_error() {
    let arr = Value::Array(vec![]);
    assert_eq!(percentile_exc_fn(&[arr, Value::Number(0.5)]), Value::Error(ErrorKind::Num));
}

#[test]
fn percentile_exc_out_of_range_for_dataset_returns_num_error() {
    // With only 1 element, idx = 0.5*2-1=0.0, lo=0, hi=0: actually returns value!
    // But with k near 1: idx=0.9*2-1=0.8, lo=0, hi=0 → returns value[0].
    // Test a case where idx < 0: k=0.1, n=1, idx=0.1*2-1=-0.8 < 0 → #NUM!
    assert_eq!(
        percentile_exc_fn(&[Value::Number(5.0), Value::Number(0.1)]),
        Value::Error(ErrorKind::Num)
    );
}
