use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(nper_fn(&[Value::Number(0.1), Value::Number(-100.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn too_many_args() {
    let args = vec![Value::Number(0.1), Value::Number(0.1), Value::Number(0.1),
                    Value::Number(0.1), Value::Number(0.1), Value::Number(0.1)];
    assert_eq!(nper_fn(&args), Value::Error(ErrorKind::Value));
}

#[test]
fn zero_rate_zero_pmt_div_by_zero() {
    let args = [Value::Number(0.0), Value::Number(0.0), Value::Number(1000.0)];
    assert_eq!(nper_fn(&args), Value::Error(ErrorKind::DivByZero));
}

#[test]
fn non_numeric_arg() {
    let args = [Value::Text("bad".to_string()), Value::Number(-100.0), Value::Number(1000.0)];
    assert_eq!(nper_fn(&args), Value::Error(ErrorKind::Value));
}
