use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(time_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_few_args() {
    let args = [Value::Number(12.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [Value::Number(12.0), Value::Number(0.0), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn negative_hour_num_error() {
    let args = [Value::Number(-1.0), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn non_numeric_arg() {
    let args = [Value::Text("bad".to_string()), Value::Number(0.0), Value::Number(0.0)];
    assert_eq!(time_fn(&args), Value::Error(ErrorKind::Value));
}
