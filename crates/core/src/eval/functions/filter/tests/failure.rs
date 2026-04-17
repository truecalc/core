use super::super::{columns_fn, filter_fn, index_fn, rows_fn, sort_fn};
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

// ---------------------------------------------------------------------------
// SORT arity
// ---------------------------------------------------------------------------

#[test]
fn sort_zero_args_returns_error() {
    assert_eq!(sort_fn(&[]), Value::Error(ErrorKind::NA));
}

// ---------------------------------------------------------------------------
// ROWS arity
// ---------------------------------------------------------------------------

#[test]
fn rows_zero_args_returns_error() {
    assert_eq!(rows_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn rows_two_args_returns_error() {
    assert_eq!(
        rows_fn(&[Value::Number(1.0), Value::Number(2.0)]),
        Value::Error(ErrorKind::NA)
    );
}

// ---------------------------------------------------------------------------
// COLUMNS arity
// ---------------------------------------------------------------------------

#[test]
fn columns_zero_args_returns_error() {
    assert_eq!(columns_fn(&[]), Value::Error(ErrorKind::NA));
}

// ---------------------------------------------------------------------------
// INDEX out of bounds → #REF!
// ---------------------------------------------------------------------------

#[test]
fn index_out_of_bounds_returns_ref_error() {
    let array = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(
        index_fn(&[array, Value::Number(10.0)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn index_zero_row_returns_value_error() {
    let array = Value::Array(vec![Value::Number(1.0)]);
    assert_eq!(
        index_fn(&[array, Value::Number(0.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn index_wrong_arg_count_returns_error() {
    assert_eq!(index_fn(&[Value::Number(1.0)]), Value::Error(ErrorKind::NA));
}
