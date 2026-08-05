use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn quartile_exc_q0_returns_num_error() {
    // quart=0 not valid for EXC → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_exc_q4_returns_num_error() {
    // quart=4 not valid for EXC → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(4.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_exc_empty_array_returns_num_error() {
    let arr = Value::Array(vec![]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_exc_too_small_dataset_returns_num_error() {
    // With only 1 element, EXC formula idx would be out of range for q1
    // k=0.25, n=1, idx=0.25*2-1=-0.5 < 0 → None → #NUM!
    assert_eq!(
        quartile_exc_fn(&[Value::Number(5.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::Num)
    );
}
