use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(to_date_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let args = [Value::Number(1.0), Value::Number(2.0)];
    assert_eq!(to_date_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn error_propagates() {
    let args = [Value::Error(ErrorKind::Value)];
    assert_eq!(to_date_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn bool_returns_value_error() {
    let args = [Value::Bool(true)];
    assert_eq!(to_date_fn(&args), Value::Error(ErrorKind::Value));
}
