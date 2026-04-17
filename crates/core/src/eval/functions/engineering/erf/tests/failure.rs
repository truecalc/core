use super::super::{erf_fn, erf_precise_fn, erfc_fn};
use crate::types::{ErrorKind, Value};

#[test]
fn erf_no_args_returns_na() {
    assert_eq!(erf_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn erf_three_args_returns_na() {
    assert_eq!(
        erf_fn(&[Value::Number(0.0), Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn erf_text_arg_returns_value_error() {
    assert_eq!(
        erf_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn erf_precise_no_args_returns_na() {
    assert_eq!(erf_precise_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn erfc_no_args_returns_na() {
    assert_eq!(erfc_fn(&[]), Value::Error(ErrorKind::NA));
}
