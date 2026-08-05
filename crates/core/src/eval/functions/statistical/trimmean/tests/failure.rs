use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn trimmean_negative_percent_returns_num() {
    assert_eq!(
        trimmean_fn(&[Value::Number(1.0), Value::Number(-0.1)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn trimmean_percent_one_returns_num() {
    assert_eq!(
        trimmean_fn(&[Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::Num)
    );
}
