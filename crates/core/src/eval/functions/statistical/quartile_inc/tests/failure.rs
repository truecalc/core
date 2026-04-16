use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn quartile_inc_q5_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(5.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_inc_negative_quart_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_inc_empty_array_returns_num_error() {
    let arr = Value::Array(vec![]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn quartile_inc_fractional_quart_returns_num_error() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(quartile_inc_fn(&[arr, Value::Number(1.5)]), Value::Error(ErrorKind::Num));
}
