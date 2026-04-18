use super::super::countif_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn wrong_arity_zero_args() {
    assert_eq!(countif_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn wrong_arity_one_arg() {
    assert_eq!(
        countif_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn wrong_arity_three_args() {
    assert_eq!(
        countif_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}
