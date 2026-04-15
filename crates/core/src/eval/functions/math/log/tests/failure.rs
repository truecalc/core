use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn log_of_zero_returns_num_error() {
    assert_eq!(log_fn(&[Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn log_of_negative_returns_num_error() {
    assert_eq!(log_fn(&[Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn ln_of_zero_returns_num_error() {
    assert_eq!(ln_fn(&[Value::Number(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn log10_of_negative_returns_num_error() {
    assert_eq!(
        log10_fn(&[Value::Number(-5.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn no_args_returns_value_error() {
    assert_eq!(log_fn(&[]), Value::Error(ErrorKind::Value));
}
