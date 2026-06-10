use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn quartile_q5_returns_num_error() {
    // quart=5 out of range → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_fn(&[arr, Value::Number(5.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_negative_quart_returns_num_error() {
    // quart=-1 → #NUM!
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_fn(&[arr, Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_empty_array_returns_num_error() {
    let arr = Value::Array(vec![]);
    assert_eq!(quartile_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_fractional_quart_truncated() {
    // quart=1.5 truncated to 1 → QUARTILE([1,2], 1) = 1.25
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_fn(&[arr, Value::Number(1.5)]), Value::Number(1.25));
}
