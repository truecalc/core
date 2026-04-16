use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn quartile_exc_fractional_quart_returns_num_error() {
    // quart=1.5 not integer → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(1.5)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_exc_negative_quart_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_exc_larger_dataset_q2() {
    // QUARTILE.EXC([2,4,6,8,10], 2) k=0.5, idx=0.5*6-1=2.0 → 6.0
    let arr = Value::Array(vec![
        Value::Number(2.0), Value::Number(4.0), Value::Number(6.0),
        Value::Number(8.0), Value::Number(10.0),
    ]);
    assert_eq!(quartile_exc_fn(&[arr, Value::Number(2.0)]), Value::Number(6.0));
}
