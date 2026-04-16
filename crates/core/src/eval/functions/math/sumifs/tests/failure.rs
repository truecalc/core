use super::super::sumifs_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn zero_args_returns_na() {
    assert_eq!(sumifs_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn one_arg_returns_na() {
    assert_eq!(
        sumifs_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn two_args_returns_na() {
    // Only sum_range + range, missing criterion
    assert_eq!(
        sumifs_fn(&[Value::Number(1.0), Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn even_args_after_sum_range_returns_na() {
    // sum_range + 3 more (odd number after sum_range, i.e. incomplete pair)
    assert_eq!(
        sumifs_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ]),
        Value::Error(ErrorKind::NA)
    );
}
