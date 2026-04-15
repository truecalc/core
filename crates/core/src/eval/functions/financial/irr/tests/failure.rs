use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_few_args() {
    assert_eq!(irr_fn(&[Value::Number(-100.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn all_positive_returns_num() {
    let args = [Value::Number(100.0), Value::Number(200.0)];
    assert_eq!(irr_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn all_negative_returns_num() {
    let args = [Value::Number(-100.0), Value::Number(-200.0)];
    assert_eq!(irr_fn(&args), Value::Error(ErrorKind::Num));
}

#[test]
fn non_numeric_arg() {
    let args = [Value::Number(-100.0), Value::Text("bad".to_string())];
    assert_eq!(irr_fn(&args), Value::Error(ErrorKind::Value));
}
