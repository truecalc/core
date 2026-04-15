use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(pmt_fn(&[Value::Number(0.1), Value::Number(5.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args() {
    let args = vec![Value::Number(0.1), Value::Number(0.1), Value::Number(0.1),
                    Value::Number(0.1), Value::Number(0.1), Value::Number(0.1)];
    assert_eq!(pmt_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn nper_zero_returns_num() {
    let args = [Value::Number(0.1), Value::Number(0.0), Value::Number(1000.0)];
    assert_eq!(pmt_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn non_numeric_rate_returns_value_error() {
    let args = [Value::Text("bad".to_string()), Value::Number(5.0), Value::Number(1000.0)];
    assert_eq!(pmt_fn(&args), Value::Error(ErrorKind::Value));
}
