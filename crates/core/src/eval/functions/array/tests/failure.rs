use super::super::{
    array_constrain_fn, flatten_fn, rows_fn, columns_fn,
    sort_fn, sumproduct_fn, transpose_fn, unique_fn,
};
use crate::types::{ErrorKind, Value};

fn num(n: f64) -> Value { Value::Number(n) }
fn flat(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| num(n)).collect())
}

#[test]
fn rows_wrong_arity() {
    assert_eq!(rows_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(rows_fn(&[num(1.0), num(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn columns_wrong_arity() {
    assert_eq!(columns_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(columns_fn(&[num(1.0), num(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn transpose_wrong_arity() {
    assert_eq!(transpose_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(transpose_fn(&[num(1.0), num(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn sort_wrong_arity() {
    assert_eq!(sort_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn unique_wrong_arity() {
    assert_eq!(unique_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn array_constrain_wrong_arity() {
    assert_eq!(array_constrain_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(array_constrain_fn(&[num(1.0), num(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn array_constrain_zero_rows() {
    let arr = Value::Array(vec![Value::Array(vec![num(1.0), num(2.0)])]);
    assert_eq!(
        array_constrain_fn(&[arr, num(0.0), num(1.0)]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn sumproduct_wrong_arity() {
    assert_eq!(sumproduct_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn sumproduct_mismatched_lengths() {
    assert_eq!(
        sumproduct_fn(&[flat(&[1.0, 2.0]), flat(&[1.0, 2.0, 3.0])]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn flatten_wrong_arity() {
    assert_eq!(flatten_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(flatten_fn(&[num(1.0), num(2.0)]), Value::Error(ErrorKind::NA));
}
