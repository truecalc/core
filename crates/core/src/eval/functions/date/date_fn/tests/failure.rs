use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    let args = [Value::Number(2024.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn no_args() {
    assert_eq!(date_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [Value::Number(2024.0), Value::Number(1.0), Value::Number(15.0), Value::Number(0.0)];
    assert_eq!(date_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn text_year_value_error() {
    let args = [Value::Text("abc".to_string()), Value::Number(1.0), Value::Number(1.0)];
    assert_eq!(date_fn(&args), Value::Error(ErrorKind::Value));
}
