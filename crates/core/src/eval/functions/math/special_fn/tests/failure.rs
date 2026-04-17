use super::super::{erf_fn, gammaln_fn};
use crate::types::{ErrorKind, Value};

#[test]
fn erf_non_numeric() {
    assert_eq!(
        erf_fn(&[Value::Text("abc".to_string())]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn erf_no_args() {
    assert_eq!(erf_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn gammaln_zero() {
    assert_eq!(
        gammaln_fn(&[Value::Number(0.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn gammaln_negative() {
    assert_eq!(
        gammaln_fn(&[Value::Number(-1.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn gammaln_no_args() {
    assert_eq!(gammaln_fn(&[]), Value::Error(ErrorKind::NA));
}
