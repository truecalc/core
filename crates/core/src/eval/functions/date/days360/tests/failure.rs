use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(days360_fn(&[Value::Number(45292.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [Value::Number(45292.0), Value::Number(45657.0), Value::Bool(false), Value::Number(0.0)];
    assert_eq!(days360_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_start() {
    let args = [Value::Text("bad".to_string()), Value::Number(45657.0)];
    assert_eq!(days360_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_end() {
    let args = [Value::Number(45292.0), Value::Text("bad".to_string())];
    assert_eq!(days360_fn(&args), Value::Error(ErrorKind::Value));
}
