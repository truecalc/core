use super::super::networkdays_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    let args = [Value::Number(45292.0)];
    assert_eq!(networkdays_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [
        Value::Number(45292.0),
        Value::Number(45296.0),
        Value::Number(0.0),
        Value::Number(0.0),
    ];
    assert_eq!(networkdays_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_start() {
    let args = [Value::Text("bad".to_string()), Value::Number(45296.0)];
    assert_eq!(networkdays_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn no_args() {
    assert_eq!(networkdays_fn(&[]), Value::Error(ErrorKind::NA));
}
