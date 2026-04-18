use super::super::countifs_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn zero_args_returns_na() {
    assert_eq!(countifs_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    assert_eq!(
        countifs_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn odd_args_returns_na() {
    // 3 args — not an even number of range/criterion pairs
    assert_eq!(
        countifs_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
