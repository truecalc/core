use super::super::workday_intl_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args() {
    assert_eq!(workday_intl_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg() {
    assert_eq!(workday_intl_fn(&[Value::Number(45292.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [
        Value::Number(45292.0),
        Value::Number(5.0),
        Value::Number(1.0),
        Value::Number(0.0),
        Value::Number(0.0),
    ];
    assert_eq!(workday_intl_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_start() {
    let args = [Value::Text("bad".to_string()), Value::Number(5.0)];
    assert_eq!(workday_intl_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn invalid_weekend_code() {
    let args = [Value::Number(45292.0), Value::Number(5.0), Value::Number(99.0)];
    assert_eq!(workday_intl_fn(&args), Value::Error(ErrorKind::Value));
}
