use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(edate_fn(&[Value::Number(45306.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args() {
    let args = [Value::Number(45306.0), Value::Number(1.0), Value::Number(0.0)];
    assert_eq!(edate_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn non_numeric_start() {
    let args = [Value::Text("bad".to_string()), Value::Number(1.0)];
    assert_eq!(edate_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_months() {
    let args = [Value::Number(45306.0), Value::Text("bad".to_string())];
    assert_eq!(edate_fn(&args), Value::Error(ErrorKind::Value));
}
