use super::super::sqrtpi_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn negative_returns_num_error() {
    // SQRTPI(-1) = #NUM!
    assert_eq!(sqrtpi_fn(&[Value::Number(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(sqrtpi_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_two_args() {
    assert_eq!(
        sqrtpi_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::NA)
    );
}
