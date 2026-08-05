use super::super::filter_fn;
use crate::types::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// FILTER arity
// ---------------------------------------------------------------------------

#[test]
fn filter_zero_args_returns_error() {
    assert_eq!(filter_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn filter_one_arg_returns_error() {
    assert_eq!(
        filter_fn(&[Value::Number(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn filter_four_args_returns_error() {
    assert_eq!(
        filter_fn(&[
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ]),
        Value::Error(ErrorKind::NA)
    );
}

// ---------------------------------------------------------------------------
// FILTER no matches → #N/A when no if_empty
// ---------------------------------------------------------------------------

#[test]
fn filter_no_matches_returns_na() {
    let array = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    let mask = Value::Array(vec![Value::Bool(false), Value::Bool(false)]);
    assert_eq!(filter_fn(&[array, mask]), Value::Error(ErrorKind::NA));
}
