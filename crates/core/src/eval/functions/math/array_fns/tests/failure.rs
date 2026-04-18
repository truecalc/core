use super::super::{columns_fn, index_fn, rows_fn};
use crate::types::{ErrorKind, Value};

fn n(v: f64) -> Value {
    Value::Number(v)
}
fn arr2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(|r| Value::Array(r)).collect())
}

#[test]
fn rows_no_args_returns_na() {
    assert_eq!(rows_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn rows_too_many_args_returns_na() {
    assert_eq!(rows_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn columns_no_args_returns_na() {
    assert_eq!(columns_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn columns_too_many_args_returns_na() {
    assert_eq!(columns_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn index_one_arg_returns_na() {
    assert_eq!(index_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn index_four_args_returns_na() {
    assert_eq!(
        index_fn(&[n(1.0), n(1.0), n(1.0), n(1.0)]),
        Value::Error(ErrorKind::NA)
    );
}

#[test]
fn index_row_zero_returns_value_error() {
    let arr = arr2d(vec![vec![n(1.0), n(2.0)]]);
    assert_eq!(
        index_fn(&[arr, n(0.0), n(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn index_col_zero_returns_value_error() {
    let arr = arr2d(vec![vec![n(1.0), n(2.0)]]);
    assert_eq!(
        index_fn(&[arr, n(1.0), n(0.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn index_row_out_of_bounds_returns_ref_error() {
    let arr = arr2d(vec![vec![n(1.0), n(2.0)]]);
    assert_eq!(
        index_fn(&[arr, n(5.0), n(1.0)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn index_col_out_of_bounds_returns_ref_error() {
    let arr = arr2d(vec![vec![n(1.0), n(2.0)]]);
    assert_eq!(
        index_fn(&[arr, n(1.0), n(5.0)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn index_1d_row_not_1_with_col_returns_ref() {
    let arr = Value::Array(vec![n(1.0), n(2.0)]);
    assert_eq!(
        index_fn(&[arr, n(2.0), n(1.0)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn index_scalar_row_not_1_returns_ref() {
    assert_eq!(index_fn(&[n(42.0), n(2.0)]), Value::Error(ErrorKind::Ref));
}

#[test]
fn index_scalar_col_not_1_returns_ref() {
    assert_eq!(
        index_fn(&[n(42.0), n(1.0), n(2.0)]),
        Value::Error(ErrorKind::Ref)
    );
}
