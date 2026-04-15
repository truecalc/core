use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(pv_fn(&[Value::Number(0.1), Value::Number(5.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args() {
    let args = vec![Value::Number(0.1), Value::Number(0.1), Value::Number(0.1),
                    Value::Number(0.1), Value::Number(0.1), Value::Number(0.1)];
    assert_eq!(pv_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_rate() {
    let args = [Value::Text("bad".to_string()), Value::Number(5.0), Value::Number(100.0)];
    assert_eq!(pv_fn(&args), Value::Error(ErrorKind::Value));
}
