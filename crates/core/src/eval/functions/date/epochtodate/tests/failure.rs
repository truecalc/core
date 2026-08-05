use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(epochtodate_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let args = [Value::Number(0.0), Value::Number(1.0), Value::Number(0.0)];
    assert_eq!(epochtodate_fn(&args), Value::Error(ErrorKind::NA));
}

#[test]
fn invalid_unit_returns_num() {
    let args = [Value::Number(0.0), Value::Number(4.0)];
    assert_eq!(epochtodate_fn(&args), Value::Error(ErrorKind::Num));
}
