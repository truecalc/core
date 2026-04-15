use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args_one() {
    assert_eq!(npv_fn(&[Value::Number(0.1)]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_few_args_zero() {
    assert_eq!(npv_fn(&[]), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_rate() {
    let args = [Value::Text("bad".to_string()), Value::Number(100.0)];
    assert_eq!(npv_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn non_numeric_cash_flow() {
    let args = [Value::Number(0.1), Value::Text("bad".to_string())];
    assert_eq!(npv_fn(&args), Value::Error(ErrorKind::Value));
}
