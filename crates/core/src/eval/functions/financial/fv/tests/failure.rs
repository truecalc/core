use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(fv_fn(&[Value::Number(0.1), Value::Number(5.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args() {
    let args = vec![Value::Number(0.1), Value::Number(0.1), Value::Number(0.1),
                    Value::Number(0.1), Value::Number(0.1), Value::Number(0.1)];
    assert_eq!(fv_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_pmt() {
    let args = [Value::Number(0.05), Value::Number(10.0), Value::Text("bad".to_string())];
    assert_eq!(fv_fn(&args), Value::Error(ErrorKind::Value));
}
